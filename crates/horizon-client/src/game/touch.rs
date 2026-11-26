//! Android on-screen controls. Layout coordinates are normalized so they survive rotation
//! and resolution changes; the layout is kept in the app-private game directory.
use std::{collections::HashMap, fs, path::PathBuf};

use horizon_client::hud::{HudAtlas, HudBatch, rgba};
use winit::{
    event::{Touch, TouchPhase},
    event::MouseButton,
    keyboard::KeyCode,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Control {
    Stick,
    Jump,
    Attack,
    Use,
    Sneak,
    Inventory,
}

impl Control {
    const ALL: [Self; 6] = [
        Self::Stick, Self::Jump, Self::Attack, Self::Use, Self::Sneak, Self::Inventory,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Stick => "stick",
            Self::Jump => "jump",
            Self::Attack => "attack",
            Self::Use => "use",
            Self::Sneak => "sneak",
            Self::Inventory => "inventory",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Stick => "MOVE",
            Self::Jump => "JUMP",
            Self::Attack => "HIT",
            Self::Use => "USE",
            Self::Sneak => "SNEAK",
            Self::Inventory => "INV",
        }
    }

    fn default_position(self) -> [f32; 2] {
        match self {
            Self::Stick => [0.15, 0.78],
            Self::Jump => [0.88, 0.78],
            Self::Attack => [0.75, 0.78],
            Self::Use => [0.88, 0.61],
            Self::Sneak => [0.75, 0.61],
            Self::Inventory => [0.94, 0.43],
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Contact {
    Button(Control),
    Drag(Control),
    Look([f64; 2]),
    Stick(Vec<KeyCode>),
}

#[derive(Debug)]
pub(super) enum Action {
    Key(KeyCode, bool),
    Mouse(MouseButton, bool),
    Look(f64, f64),
}

#[derive(Default)]
pub(super) struct TouchControls {
    positions: HashMap<Control, [f32; 2]>,
    contacts: HashMap<u64, Contact>,
    pub(super) editing: bool,
}

impl TouchControls {
    pub(super) fn load() -> Self {
        let mut this = Self::default();
        let path = Self::layout_path();
        if let Ok(text) = fs::read_to_string(path)
            && let Ok(value) = serde_json::from_str::<serde_json::Value>(&text)
        {
            for control in Control::ALL {
                if let Some(position) = value.get(control.name()).and_then(|v| v.as_array())
                    && position.len() == 2
                    && let (Some(x), Some(y)) = (position[0].as_f64(), position[1].as_f64())
                {
                    this.positions.insert(
                        control,
                        [x.clamp(0.04, 0.96) as f32, y.clamp(0.08, 0.94) as f32],
                    );
                }
            }
        }
        this
    }

    fn layout_path() -> PathBuf {
        std::env::current_dir()
            .unwrap_or_default()
            .join("horizon")
            .join("touch-controls.json")
    }

    fn position(&self, control: Control) -> [f32; 2] {
        self.positions
            .get(&control)
            .copied()
            .unwrap_or_else(|| control.default_position())
    }

    fn save(&self) {
        let mut value = serde_json::Map::new();
        for control in Control::ALL {
            let [x, y] = self.position(control);
            value.insert(control.name().into(), serde_json::json!([x, y]));
        }
        let path = Self::layout_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, serde_json::Value::Object(value).to_string());
    }

    fn hit(&self, control: Control, point: [f64; 2], width: f32, height: f32) -> bool {
        let [nx, ny] = self.position(control);
        let x = nx * width;
        let y = ny * height;
        let radius = if control == Control::Stick { 64.0 } else { 28.0 };
        (point[0] - f64::from(x)).powi(2) + (point[1] - f64::from(y)).powi(2)
            <= radius * radius
    }

    fn button_action(control: Control, pressed: bool) -> Option<Action> {
        match control {
            Control::Jump => Some(Action::Key(KeyCode::Space, pressed)),
            Control::Sneak => Some(Action::Key(KeyCode::ShiftLeft, pressed)),
            Control::Attack => Some(Action::Mouse(MouseButton::Left, pressed)),
            Control::Use => Some(Action::Mouse(MouseButton::Right, pressed)),
            Control::Inventory if pressed => Some(Action::Key(KeyCode::KeyE, true)),
            Control::Inventory => Some(Action::Key(KeyCode::KeyE, false)),
            Control::Stick => None,
        }
    }

    fn stick_keys(&self, point: [f64; 2], width: f32, height: f32) -> Vec<KeyCode> {
        let [nx, ny] = self.position(Control::Stick);
        let cx = f64::from(nx * width);
        let cy = f64::from(ny * height);
        let radius = 52.0_f64;
        let dx = ((point[0] - cx) / radius).clamp(-1.0, 1.0);
        let dy = ((point[1] - cy) / radius).clamp(-1.0, 1.0);
        let mut keys = Vec::with_capacity(2);
        if dy < -0.28 { keys.push(KeyCode::KeyW); }
        if dy > 0.28 { keys.push(KeyCode::KeyS); }
        if dx < -0.28 { keys.push(KeyCode::KeyA); }
        if dx > 0.28 { keys.push(KeyCode::KeyD); }
        keys
    }

    fn key_diff(old: &[KeyCode], new: &[KeyCode], actions: &mut Vec<Action>) {
        for key in old.iter().filter(|key| !new.contains(key)) {
            actions.push(Action::Key(*key, false));
        }
        for key in new.iter().filter(|key| !old.contains(key)) {
            actions.push(Action::Key(*key, true));
        }
    }

    pub(super) fn handle(
        &mut self,
        touch: Touch,
        width: f32,
        height: f32,
    ) -> Vec<Action> {
        let point = [touch.location.x, touch.location.y];
        let mut actions = Vec::new();
        match touch.phase {
            TouchPhase::Started => {
                let edit_button = [f64::from(width * 0.96), f64::from(height * 0.08)];
                if (point[0] - edit_button[0]).powi(2)
                    + (point[1] - edit_button[1]).powi(2)
                    < 28.0_f64.powi(2)
                {
                    self.editing = !self.editing;
                    return actions;
                }
                if self.editing {
                    if let Some(control) = Control::ALL
                        .into_iter()
                        .find(|&c| self.hit(c, point, width, height))
                    {
                        self.contacts.insert(touch.id, Contact::Drag(control));
                    }
                    return actions;
                }
                if let Some(control) = [
                    Control::Jump,
                    Control::Attack,
                    Control::Use,
                    Control::Sneak,
                    Control::Inventory,
                ]
                    .into_iter()
                    .find(|&c| self.hit(c, point, width, height))
                {
                    self.contacts.insert(touch.id, Contact::Button(control));
                    if let Some(action) = Self::button_action(control, true) {
                        actions.push(action);
                    }
                } else if self.hit(Control::Stick, point, width, height) {
                    let keys = self.stick_keys(point, width, height);
                    self.contacts.insert(touch.id, Contact::Stick(keys.clone()));
                    actions.extend(keys.into_iter().map(|key| Action::Key(key, true)));
                } else if point[0] > f64::from(width * 0.38) {
                    self.contacts.insert(touch.id, Contact::Look(point));
                }
            }
            TouchPhase::Moved => {
                let next_stick = self.stick_keys(point, width, height);
                match self.contacts.get_mut(&touch.id) {
                    Some(Contact::Drag(control)) => {
                        let control = *control;
                        self.positions.insert(
                            control,
                            [
                                (point[0] / f64::from(width)).clamp(0.04, 0.96) as f32,
                                (point[1] / f64::from(height)).clamp(0.08, 0.94) as f32,
                            ],
                        );
                    }
                    Some(Contact::Look(previous)) => {
                        let dx = point[0] - previous[0];
                        let dy = point[1] - previous[1];
                        *previous = point;
                        actions.push(Action::Look(dx, dy));
                    }
                    Some(Contact::Stick(previous)) => {
                        Self::key_diff(previous, &next_stick, &mut actions);
                        *previous = next_stick;
                    }
                    _ => {}
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                if let Some(contact) = self.contacts.remove(&touch.id) {
                    match contact {
                        Contact::Button(control) => {
                            if let Some(action) = Self::button_action(control, false) {
                                actions.push(action);
                            }
                        }
                        Contact::Stick(keys) => {
                            actions.extend(keys.into_iter().map(|key| Action::Key(key, false)));
                        }
                        Contact::Drag(_) => self.save(),
                        Contact::Look(_) => {}
                    }
                }
            }
        }
        actions
    }

    pub(super) fn cancel_all(&mut self) -> Vec<Action> {
        let mut actions = Vec::new();
        for (_, contact) in self.contacts.drain() {
            match contact {
                Contact::Button(control) => {
                    if let Some(action) = Self::button_action(control, false) {
                        actions.push(action);
                    }
                }
                Contact::Stick(keys) => {
                    actions.extend(keys.into_iter().map(|key| Action::Key(key, false)));
                }
                Contact::Drag(_) | Contact::Look(_) => {}
            }
        }
        actions
    }

    pub(super) fn clear_contacts(&mut self) {
        self.contacts.clear();
    }

    pub(super) fn draw(&self, batch: &mut HudBatch, atlas: &HudAtlas) {
        let (w, h) = (batch.width, batch.height);
        let base = rgba(20, 24, 30, if self.editing { 190 } else { 115 });
        let edge = rgba(220, 230, 240, if self.editing { 220 } else { 135 });
        let [sx, sy] = self.position(Control::Stick);
        let (sx, sy) = (sx * w, sy * h);
        batch.fill(atlas, sx - 58.0, sy - 58.0, 116.0, 116.0, edge);
        batch.fill(atlas, sx - 54.0, sy - 54.0, 108.0, 108.0, base);
        batch.fill(atlas, sx - 20.0, sy - 20.0, 40.0, 40.0, edge);
        batch.fill(atlas, sx - 16.0, sy - 16.0, 32.0, 32.0, base);
        for control in [
            Control::Jump,
            Control::Attack,
            Control::Use,
            Control::Sneak,
            Control::Inventory,
        ] {
            let [nx, ny] = self.position(control);
            let (x, y) = (nx * w, ny * h);
            batch.fill(atlas, x - 27.0, y - 27.0, 54.0, 54.0, edge);
            batch.fill(atlas, x - 23.0, y - 23.0, 46.0, 46.0, base);
            let label = control.label();
            let text_width = label.len() as f32 * 6.0;
            batch.text(atlas, label, x - text_width / 2.0, y - 4.0, u32::MAX, true);
        }
        let (ex, ey) = (w * 0.96, h * 0.08);
        batch.fill(atlas, ex - 18.0, ey - 18.0, 36.0, 36.0, edge);
        batch.fill(atlas, ex - 15.0, ey - 15.0, 30.0, 30.0, base);
        batch.text(
            atlas,
            if self.editing { "OK" } else { "EDIT" },
            ex - if self.editing { 6.0 } else { 12.0 },
            ey - 4.0,
            u32::MAX,
            true,
        );
        if self.editing {
            batch.text(atlas, "DRAG BUTTONS TO MOVE", w * 0.5 - 70.0, h * 0.08, u32::MAX, true);
        }
    }
}

impl super::Game<'_> {
    pub(super) fn handle_touch(&mut self, touch: Touch) -> horizon_client::Result<()> {
        if self.screen != super::Screen::Playing || self.title.is_some() || self.player.is_none() {
            let actions = self.touch.cancel_all();
            for action in actions {
                match action {
                    Action::Key(key, pressed) => self.mapped_key_event(key, pressed, false)?,
                    Action::Mouse(button, pressed) => self.play_mouse(button, pressed)?,
                    Action::Look(_, _) => {}
                }
            }
            return Ok(());
        }
        let Some(renderer) = self.renderer.as_ref() else { return Ok(()); };
        let batch = HudBatch::new([renderer.config.width, renderer.config.height]);
        let mut touch = touch;
        let scale = f64::from(batch.scale);
        touch.location.x /= scale;
        touch.location.y /= scale;
        let actions = self.touch.handle(touch, batch.width, batch.height);
        for action in actions {
            match action {
                Action::Key(key, pressed) => self.mapped_key_event(key, pressed, false)?,
                Action::Mouse(button, pressed) => self.play_mouse(button, pressed)?,
                Action::Look(dx, dy) => {
                    let sensitivity = f64::from(self.settings.sensitivity) * 0.35;
                    let dx = dx * sensitivity * if self.settings.invert_x { -1.0 } else { 1.0 };
                    let dy = dy * sensitivity * if self.settings.invert_y { -1.0 } else { 1.0 };
                    self.yaw -= dx as f32;
                    self.pitch = (self.pitch + dy as f32).clamp(-90.0, 90.0);
                }
            }
        }
        Ok(())
    }
}

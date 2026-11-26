//! Android NativeActivity entry point for the existing Horizon client.
#![cfg(target_os = "android")]

use std::path::PathBuf;
use winit::platform::android::activity::AndroidApp;

#[global_allocator]
static ALLOCATOR: horizon_threads::Allocator = horizon_threads::Allocator;

/// Hide both Android system bars before the native game surface is shown.
#[unsafe(no_mangle)]
fn android_on_create(state: &winit::platform::android::activity::OnCreateState) {
    use jni::{
        JavaVM, jni_sig, jni_str,
        objects::{JObject, JValue},
    };

    let vm = unsafe { JavaVM::from_raw(state.vm_as_ptr().cast()) };
    let _ = vm.attach_current_thread(|env| {
        let activity = unsafe { JObject::from_raw(env, state.activity_as_ptr().cast()) };
        let Ok(window) = env
            .call_method(
                &activity,
                jni_str!("getWindow"),
                jni_sig!("()Landroid/view/Window;"),
                &[],
            )
            .and_then(|value| value.l())
        else {
            return Ok(());
        };
        let Ok(decor) = env
            .call_method(
                &window,
                jni_str!("getDecorView"),
                jni_sig!("()Landroid/view/View;"),
                &[],
            )
            .and_then(|value| value.l())
        else {
            return Ok(());
        };

        // LAYOUT_STABLE | LAYOUT_HIDE_NAVIGATION | LAYOUT_FULLSCREEN |
        // HIDE_NAVIGATION | FULLSCREEN | IMMERSIVE_STICKY.
        let _ = env.call_method(
            &decor,
            jni_str!("setSystemUiVisibility"),
            jni_sig!("(I)V"),
            &[JValue::Int(0x1706)],
        );
        Ok::<(), jni::errors::Error>(())
    });
}

/// Android's NativeActivity loader calls this symbol. Place all mutable game data
/// under the app-private files directory, then hand the activity to winit.
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    // Keep Android's status bar out of the game surface.
    app.set_window_flags(
        winit::platform::android::activity::WindowManagerFlags::FULLSCREEN,
        winit::platform::android::activity::WindowManagerFlags::empty(),
    );

    let data_dir: PathBuf = app
        .internal_data_path()
        .expect("Android did not provide an internal app data directory");
    std::fs::create_dir_all(&data_dir).expect("cannot create Horizon app data directory");
    std::env::set_current_dir(&data_dir).expect("cannot select Horizon app data directory");

    // Set Horizon's own path overrides directly instead of mutating process-wide
    // environment variables after android-activity has started its I/O thread.
    let home = data_dir.join("horizon");
    horizon_client::logging::set_game_home(home.clone());
    horizon_client::logging::set_player_dir(home);

    horizon_client::game::set_android_app(app);
    horizon_client::entry::main();
}

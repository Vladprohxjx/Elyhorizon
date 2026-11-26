//! Android NativeActivity entry point for the existing Horizon client.
#![cfg(target_os = "android")]

use std::path::PathBuf;
use winit::platform::android::activity::AndroidApp;

#[global_allocator]
static ALLOCATOR: horizon_threads::Allocator = horizon_threads::Allocator;

/// Android's NativeActivity loader calls this symbol. Place all mutable game data
/// under the app-private files directory, then hand the activity to winit.
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    // Keep the NativeActivity window non-fullscreen so Android can show
    // the soft keyboard for focused game text fields.

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

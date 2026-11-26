//! Android NativeActivity entry point for the existing Horizon client.
#![cfg(target_os = "android")]

use std::path::PathBuf;

#[global_allocator]
static ALLOCATOR: horizon_threads::Allocator = horizon_threads::Allocator;
use winit::platform::android::activity::AndroidApp;

/// Android's NativeActivity loader calls this symbol. Place all mutable game data
/// under the app-private files directory, then hand the activity to winit.
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    let data_dir: PathBuf = app
        .internal_data_path()
        .expect("Android did not provide an internal app data directory");
    std::fs::create_dir_all(&data_dir).expect("cannot create Horizon app data directory");
    std::env::set_current_dir(&data_dir).expect("cannot select Horizon app data directory");
    // Edition 2024 marks process-wide environment mutation unsafe. This runs
    // once, before Horizon starts worker threads.
    unsafe {
        std::env::set_var("HORIZON_HOME", data_dir.join("horizon"));
        std::env::set_var("HOME", &data_dir);
        std::env::set_var("XDG_DATA_HOME", data_dir.join(".local/share"));
        std::env::set_var("XDG_CONFIG_HOME", data_dir.join(".config"));
    }
    horizon_client::game::set_android_app(app);
    horizon_client::entry::main();
}

//! Android NativeActivity entry point for the existing Horizon client.
#![cfg(target_os = "android")]

use winit::platform::android::activity::AndroidApp;

/// Android's NativeActivity loader calls this symbol. Hand the activity to winit
/// before entering the same startup path used by desktop builds.
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    horizon_client::game::set_android_app(app);
    horizon_client::entry::main();
}

# Android build (initial NativeActivity MVP)

Requirements: Android SDK + NDK, JDK 17+, Rust 1.90+, Android Rust target
`aarch64-linux-android`, Gradle 8.9+, and `cargo-ndk`.

```sh
rustup target add aarch64-linux-android
cargo install cargo-ndk
cd android
gradle :app:assembleDebug
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

The first build targets arm64-v8a. Rust is built as a `cdylib` and loaded by
Android's `NativeActivity`. The app uses its private files directory for the
working directory, logs, settings and worlds.

## Import your own Minecraft resources

The APK deliberately does not bundle Minecraft's copyrighted game files.
Prepare the normal 26.3 resource tree on a desktop checkout with the documented
`horizon-prepare --complete` workflow, then copy that prepared directory to
the app's private storage:

```sh
adb push .local/horizon-client/26.3 /sdcard/horizon-26.3
adb shell run-as org.horizon.client mkdir -p files/.local/horizon-client
adb shell run-as org.horizon.client cp -R /sdcard/horizon-26.3 files/.local/horizon-client/26.3
``

If the device's Android build does not provide `cp` in `run-as`, copy the
directory through Android Studio Device Explorer into
`/data/data/org.horizon.client/files/.local/horizon-client/26.3`.

## Current MVP scope and limitations

- NativeActivity boot and winit event-loop handoff.
- Vulkan/GLES-capable wgpu configuration inherited from the desktop renderer.
- Single-touch pointer emulation for existing title/menu buttons.
- Game data and save paths redirected into app-private storage.
- No bundled Minecraft assets; import your own prepared 26.3 resources first.
- Virtual movement/aim controls, Android audio, controller hotplug, background
  pause/resume stress testing, and on-device performance tuning remain follow-up
  work. The native APK has not yet been built or tested on a physical device.

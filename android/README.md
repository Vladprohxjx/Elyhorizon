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
- Multi-touch gameplay controls: a movable movement pad, right-side look, jump, attack, use, sneak and inventory buttons. Tap EDIT in-game to drag controls into place; positions are saved in app-private storage.
- Game data and save paths redirected into app-private storage.
- No bundled Minecraft assets; import your own prepared 26.3 resources first.
- Android audio, controller hotplug, background pause/resume stress testing and
  device-specific performance validation remain follow-up work. Fresh Android
  installs start with a conservative performance profile (8-chunk view distance,
  6-chunk simulation distance, 60 FPS battery mode and reduced render resolution). The native APK has not yet been built or tested on a physical device.

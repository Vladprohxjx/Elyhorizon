# Android build (initial NativeActivity MVP)

Requirements: Android SDK + NDK, JDK 17+, Rust 1.90+, Android Rust target
`aarch64-linux-android`, and `cargo-ndk`.

```sh
rustup target add aarch64-linux-android
cargo install cargo-ndk
cd android
gradle :app:assembleDebug
```

Install `app/build/outputs/apk/debug/app-debug.apk` with adb.

The initial target is arm64-v8a. This APK embeds the native client library;
Minecraft Java 26.3 assets/resources are not bundled. The client still requires
the repository's normal `horizon-prepare` output. Android-specific storage
and resource import must be completed before gameplay can start; see the
porting notes in the repository's Android documentation.

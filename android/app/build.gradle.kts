plugins {
    id("com.android.application")
}

android {
    namespace = "org.horizon.client"
    compileSdk = 35

    defaultConfig {
        applicationId = "org.horizon.client"
        minSdk = 26
        targetSdk = 28
        versionCode = 1
        versionName = "0.1.0"
        ndk {
            abiFilters += "arm64-v8a"
        }
    }

    sourceSets["main"].jniLibs.srcDir("src/main/jniLibs")
}

val buildRust by tasks.registering(Exec::class) {
    workingDir = rootProject.projectDir.parentFile
    commandLine(
        "cargo", "ndk", "-t", "arm64-v8a",
        "-o", "android/app/src/main/jniLibs",
        "build", "--release", "-p", "horizon-client-android",
    )
    inputs.files(fileTree("../../crates"), file("../../Cargo.toml"), file("../../Cargo.lock"))
    outputs.dir("src/main/jniLibs")
}

tasks.named("preBuild").configure {
    dependsOn(buildRust)
}

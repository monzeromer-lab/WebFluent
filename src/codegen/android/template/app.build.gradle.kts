// Written once by `wf build`, and yours to change: signing, flavours and
// dependencies go here.
import java.util.Properties

plugins {
    id("com.android.application")
}

// The app's id, version and Android levels, from `build.android` in
// webfluent.app.json. `wf build` rewrites this file on every build, so
// change them there.
val webfluent = Properties().apply {
    val written = file("webfluent.properties")
    check(written.exists()) {
        "app/webfluent.properties is missing: run `wf build` in the WebFluent project first"
    }
    written.inputStream().use { load(it) }
}

android {
    namespace = "@@PACKAGE@@"
    compileSdk = webfluent.getProperty("compileSdk").toInt()

    defaultConfig {
        applicationId = webfluent.getProperty("applicationId")
        minSdk = webfluent.getProperty("minSdk").toInt()
        targetSdk = webfluent.getProperty("targetSdk").toInt()
        versionCode = webfluent.getProperty("versionCode").toInt()
        versionName = webfluent.getProperty("versionName")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

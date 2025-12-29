plugins {
    kotlin("multiplatform")
    // kotlin("native.cocoapods") // Только для iOS сборки на Mac
    id("com.android.library")
    kotlin("plugin.serialization") version "1.9.21"
    id("org.jetbrains.compose") version "1.5.11"
}

kotlin {
    androidTarget {
        compilations.all {
            kotlinOptions {
                jvmTarget = "17"
            }
        }
    }
    
    // iOS targets - только для macOS сборки
    // Раскомментируйте для iOS сборки на Mac
    // iosX64()
    // iosArm64()
    // iosSimulatorArm64()

    // cocoapods {
    //     summary = "Rimskiy Shared Module"
    //     homepage = "https://github.com/rimskiy/shared"
    //     version = "1.0"
    //     ios.deploymentTarget = "14.1"
    //     framework {
    //         baseName = "RimskiyShared"
    //         isStatic = true
    //     }
    //     pod("ComposeApp") {
    //         moduleName = "ComposeApp"
    //         version = "1.0"
    //     }
    // }

    sourceSets {
        val commonMain by getting {
            dependencies {
                // Coroutines
                implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")
                
                // Serialization
                implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.6.0")
                
                // Ktor для HTTP
                implementation("io.ktor:ktor-client-core:2.3.5")
                implementation("io.ktor:ktor-client-content-negotiation:2.3.5")
                implementation("io.ktor:ktor-serialization-kotlinx-json:2.3.5")
                implementation("io.ktor:ktor-client-logging:2.3.5")
                
                // Settings (для хранения токенов) - используем общую версию
                implementation("com.russhwolf:multiplatform-settings:1.1.1")
                
                // UUID
                implementation("com.benasher44:uuid:0.8.4")
                
                // Compose Multiplatform
                implementation(compose.runtime)
                implementation(compose.foundation)
                implementation(compose.material3)
                implementation(compose.ui)
                implementation(compose.materialIconsExtended)
            }
        }
        
        val commonTest by getting {
            dependencies {
                implementation(kotlin("test"))
            }
        }
        
        val androidMain by getting {
            dependencies {
                implementation("io.ktor:ktor-client-android:2.3.5")
                // multiplatform-settings-android уже включен через commonMain
            }
        }
        
        // iOS source sets - только для macOS сборки
        // Раскомментируйте для iOS сборки на Mac
        // val iosX64Main by getting
        // val iosArm64Main by getting
        // val iosSimulatorArm64Main by getting
        // val iosMain by creating {
        //     dependsOn(commonMain)
        //     iosX64Main.dependsOn(this)
        //     iosArm64Main.dependsOn(this)
        //     iosSimulatorArm64Main.dependsOn(this)
        //     
        //     dependencies {
        //         implementation("io.ktor:ktor-client-darwin:2.3.5")
        //         implementation("com.russhwolf:multiplatform-settings-ios:1.1.1")
        //     }
        // }
    }
}

android {
    namespace = "com.rimskiy.shared"
    compileSdk = 34
    
    defaultConfig {
        minSdk = 24
    }
    
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}


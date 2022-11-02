# Rust Multi-platform Mobile (RMM) ;-)

This repo is a "hello world" example of using a single shared library, written in rust, in an Android app, an iOS app, and a web application.

For mobile, it uses [`uniffi`](https://github.com/mozilla/uniffi-rs) from Mozilla, which generates all the necessary bindings and marshalls FFI calls between kotlin/swift and rust.

For web it uses [`yew`](https://yew.rs/).

## Architecture

To enable state management and orchestration of I/O and effects, the shared library is built following the
[Elm architecture](https://guide.elm-lang.org/architecture/), with the platform specific shell acting as the 'platform' which facilitates side-effects. The subtle difference is that rendering the user interface is one of the side effects.

![Architecture](./architecture.png)

The shared library defines two types of messages - `Msg` and `Cmd`. The `Msg` messages flow from the
app to the library, in response to user interactions or to relay results of asynchronous work (like I/O). `Cmd`
are messages the library returns to the application to request a side-effect, for example a HTTP call.

A typical cycle through the loop begins with a user interaction which passes a message to the library through its `update` function. The function updates the library's inner state, and responds with a `Cmd`. In a simple case
this might be a `Cmd::Render` requesting a user interface refresh. In more complex cases, the command may
request a network API call, current time, biometric authentication, image from a camera, or any other side-effect. Once the app facilitates the side-effect, it passes the result to the core as another `Msg`, and the exchange
continues in this fashion until a `Cmd::Render` is returned and no more side-effects are in flight.

The library also exposes a view model through a `view` function, which returns information instructing the
application what to present on screen using the native UI toolkit.

This architecture is also similar to Erlang's actors, and Haskell's IO monad. The key benefits of building the
shared library in this way are:

- The library is side-effect free and doesn't require any system APIs, which means it can be compiled to WebAssembly
- The library, which contains the bulk of the application logic, is testable with no mocking or stubbing by
  setting up desired state, and then passing messages to the `update` function and checking the right commands are returned and the right view is presented
- Thanks to UniFFI, the types are shared across the FFI boundary, and when the code is updated
  (e.g. with new variants on the `Msg` type), the type checking in Swift and Kotlin will prevent the apps from
  building until the new messages are handled by them, keeping everything in sync.

## Get the example running

### Rust

1. Make sure you have the following rust targets installed (e.g. `rustup target add aarch64-apple-ios`)

   ```txt
   aarch64-apple-darwin
   aarch64-apple-ios
   aarch64-apple-ios-sim
   aarch64-linux-android
   wasm32-unknown-unknown
   x86_64-apple-ios
   ```

1. Install the `uniffi-bindgen` binary ...

   ```sh
   cargo install uniffi_bindgen
   ```

1. Make sure the core builds

   ```sh
   cd shared
   cargo build
   # => Finished dev [unoptimized + debuginfo] target(s) in 1.40s
   ```

### Yew web app

The web application should now build and run

```
cd web
trunk serve
```

### iOS

You will need XCode, which you can get in the mac AppStore. When XCode starts, open the `iOS` directory
and run a build, the app should start in the simulator.

### Android

You will need [Android Studio](https://developer.android.com/studio/). You may or may not face a few problems:

- Build failing due to a `linker-wrapper.sh` script failure. Make sure you have Python installed and in PATH
- Android studio failing to install git. You can set the path to your git binary (e.g. the homebrew one)
  in the preferences under Version Control > Git

You should be able to build and run the project in the simulator.

## How to start a fresh project of your own

### Rust core

1. Make sure you have the following rust targets installed (e.g. `rustup target add aarch64-apple-ios`)

   ```txt
   aarch64-apple-darwin
   aarch64-apple-ios
   aarch64-apple-ios-sim
   aarch64-linux-android
   wasm32-unknown-unknown
   x86_64-apple-ios
   ```

1. Install the `uniffi-bindgen` binary ...

   ```sh
   cargo install uniffi_bindgen
   ```

1. Create a new rust library ...

   ```sh
   cargo new --lib shared
   ```

1. Edit [`./Cargo.toml`](./Cargo.toml) to add the new library to the Cargo workspace ...

   ```toml
   [workspace]
   members = ["shared"]
   ```

1. Edit [`./shared/Cargo.toml`](./shared/Cargo.toml) ...
   Note that the crate type:

   1. `"lib"` is the default rust library for use when linking into a rust binary, e.g. for WebAssembly in the web variant
   1. `"staticlib"` is a static library (`libshared.a`) for including in the Swift iOS app variant
   1. `"cdylib"` is a c-abi dynamic library (`libshared.so`) for use with JNA when included in the Kotlin Android app variant

   ```toml
   [lib]
   crate-type = ["lib", "staticlib", "cdylib"]
   name = "shared"

   [dependencies]
   uniffi = "0.21.0"
   uniffi_macros = "0.21.0"

   [build-dependencies]
   uniffi_build = { version = "0.21.0", features = ["builtin-bindgen"] }

   ```

1. Create [`./shared/src/shared.udl`](./shared/src/shared.udl) ...

   ```txt
   namespace shared {
     u32 add(u32 left, u32 right);
   };

   ```

1. Create [`./shared/uniffi.toml`](./shared/uniffi.toml) ...

   ```toml
   [bindings.kotlin]
   package_name = "redbadger.rmm.shared"
   cdylib_name = "shared"

   [bindings.swift]
   cdylib_name = "shared_ffi"
   omit_argument_labels = true

   ```

1. Create [`./shared/build.rs`](./shared/build.rs) ...

   ```rust
   fn main() {
        uniffi_build::generate_scaffolding("./src/shared.udl").unwrap();
   }

   ```

1. Include the scaffolding in [`./shared/src/lib.rs`](./shared/src/lib.rs), and change types from `usize` to `u32` ...

   ```rust
   uniffi_macros::include_scaffolding!("shared");

   pub fn add(left: u32, right: u32) -> u32 {
       left + right
   }

   ```

1. Make sure everything builds OK ...
   ```sh
   cargo build
   ```

### Android App

1. Create a Kotlin App in Android Studio (e.g. "Empty Compose Activity (Material UI)" at `/Android`)

1. Add a Kotlin Android Library (`aar`) — you can find more details on how to do this [here](https://developer.android.com/studio/projects/android-library), but in a nutshell ...

   1. go to File -> New -> New Module...
   1. choose "Android Library"
   1. Call it e.g. `shared`
   1. Package name must match that in [`./shared/uniffi.toml`](./shared/uniffi.toml), e.g. `redbadger.rmm.shared`

1. Add the `shared` library as a dependency of `app`

   1. either...
      1. go to File -> Project Structure...
      1. choose "Dependencies"
      1. choose `app` and use the `+` to add a "Module dependency"
      1. select the `shared` library
   1. or...
      1. add this line to the `dependencies` section in [`./Android/app/build.gradle`](./Android/app/build.gradle)
         ```groovy
         implementation project(path: ':shared')
         ```

1. Mozilla has a rust gradle plugin for android [here](https://github.com/mozilla/rust-android-gradle). Add the plugin to `./Android/build.gradle`, and sync ...

   ```groovy
   plugins {
      id "org.mozilla.rust-android-gradle.rust-android" version "0.9.3"
   }
   ```

1. We are also using [Java Native Access (JNA)](https://github.com/java-native-access/jna). Add the following to `./Android/shared/build.gradle`, and sync ...

   ```groovy
   plugins {
      ...
      id 'org.mozilla.rust-android-gradle.rust-android'
   }
   android {
      namespace 'redbadger.rmm.shared'
      ...
      ndkVersion '25.1.8937393'
   }

   dependencies {
      implementation "net.java.dev.jna:jna:5.12.1@aar"
      ...
   }

   apply plugin: 'org.mozilla.rust-android-gradle.rust-android'

   cargo {
      module  = "../.."
      libname = "shared"
      targets = ["arm64"]
      extraCargoBuildArguments = ['--package', 'shared']
   }

   afterEvaluate {
      // The `cargoBuild` task isn't available until after evaluation.
      android.libraryVariants.all { variant ->
         def productFlavor = ""
         variant.productFlavors.each {
               productFlavor += "${it.name.capitalize()}"
         }
         def buildType = "${variant.buildType.name.capitalize()}"
         tasks["cargoBuild"].dependsOn(tasks["bindGen"])
         tasks["generate${productFlavor}${buildType}Assets"].dependsOn(tasks["cargoBuild"])
      }
   }

   task bindGen(type: Exec) {
      def outDir = "${projectDir}/src/main/java"
      workingDir "../../"
      commandLine(
               "sh", "-c",
               """\
               \$HOME/.cargo/bin/uniffi-bindgen generate shared/src/shared.udl \
               --language kotlin \
               --out-dir $outDir
               """
      )
   }

   ```

1. Run "Build -> Make project" to make sure that everything compiles (including the shared rust library) — you should be able to see the library object file ...

   ```sh
   ls Android/shared/build/rustJniLibs/android/arm64-v8a
   libshared.so
   ```

1. Try calling into the rust library from the Android app, for example ...

   1. open `Android/app/src/main/java/com/example/android/MainActivity.kt`
   1. add `import redbadger.rmm.shared.add`
   1. add a `class` for the callback to get Platform details ...
      ```kotlin
      class GetPlatform : Platform {
         override fun get(): String {
            return Build.BRAND + " " + Build.VERSION.RELEASE
         }
      }
      ```
   1. call the `addForPlatform` function, e.g. in a Text UI component ...

      ```kotlin
      Text(text = addForPlatform(1u, 2u, GetPlatform()))
      ```

   1. run the app in a simulator to show that the function in the shared library is called

### iOS App

(adapted, for UniFFI, from [this post](https://blog.mozilla.org/data/2022/01/31/this-week-in-glean-building-and-deploying-a-rust-library-on-ios/) by Jan-Erik Rediger, with thanks.)

1. Open xCode and create a new iOS app (e.g. called `iOS` with organization `com.redbadger`)

1. Add a build rule to process files that match the pattern `*.udl` with the following script.
   This will use Uniffi to create the swift bindings and the C headers in a `generated` directory.
   Uncheck "Run once per architecture" ...

   ```bash
   # Skip during indexing phase in XCode 13+
   if [ $ACTION == "indexbuild" ]; then
      echo "Not building *.udl files during indexing."
      exit 0
   fi

   # Skip for preview builds
   if [ "${ENABLE_PREVIEWS}" = "YES" ]; then
      echo "Not building *.udl files during preview builds."
      exit 0
   fi

   cd "$INPUT_FILE_DIR"/.. && "$HOME"/.cargo/bin/uniffi-bindgen generate src/"$INPUT_FILE_NAME" --language swift --out-dir "$PROJECT_DIR/generated"
   ```

   Also add the following as output files:

   ```txt
   $(PROJECT_DIR)/generated/$(INPUT_FILE_BASE).swift
   $(PROJECT_DIR)/generated/$(INPUT_FILE_BASE)FFI.h
   ```

1. In "Build Settings" ...

   1. add a "User-defined setting" called "`build_variant`", with a value of `debug` for Debug and `release` for Release
   1. search for "bridging header", and add `generated/sharedFFI.h`, for any architecture/SDK, in both Debug and Release. If there isn't already a setting for "bridging header" you can add one (and then delete it) as per [this StackOverflow question](https://stackoverflow.com/questions/41787935/how-to-use-objective-c-bridging-header-in-a-swift-project/41788055#41788055)
   1. search for "library search paths" and add some dummy values for debug and release. This will update the project file so you can search in it for `LIBRARY_SEARCH_PATHS` in the next step.

1. Open `./iOS/iOs.xcodeproj/project.pbxproj` in a code editor and search for "LIBRARY_SEARCH_PATHS" (you should find 2 occurrences), and add the following ...

   1. in the "debug" section

   ```txt
   "LIBRARY_SEARCH_PATHS[sdk=iphoneos*][arch=arm64]" = "$(PROJECT_DIR)/../target/aarch64-apple-ios/debug";
   "LIBRARY_SEARCH_PATHS[sdk=iphonesimulator*][arch=arm64]" = "$(PROJECT_DIR)/../target/aarch64-apple-ios-sim/debug";
   "LIBRARY_SEARCH_PATHS[sdk=iphonesimulator*][arch=x86_64]" = "$(PROJECT_DIR)/../target/x86_64-apple-ios/debug";
   ```

   1. in the "release"" section

   ```txt
   "LIBRARY_SEARCH_PATHS[sdk=iphoneos*][arch=arm64]" = "$(PROJECT_DIR)/../target/aarch64-apple-ios/release";
   "LIBRARY_SEARCH_PATHS[sdk=iphonesimulator*][arch=arm64]" = "$(PROJECT_DIR)/../target/aarch64-apple-ios-sim/release";
   "LIBRARY_SEARCH_PATHS[sdk=iphonesimulator*][arch=x86_64]" = "$(PROJECT_DIR)/../target/x86_64-apple-ios/release";
   ```

1. Create a script to build the rust library (e.g. this script [`./iOS/bin/compile-library.sh`](./iOS/bin/compile-library.sh))
1. Test the build (which will still fail, but should create the `generated` directory)
1. In "Build phases", create or modify the following phases (you can drag them so that they match the order below) ...

   1. add a "New Run Script Phase" with the following script, and uncheck "Based on dependency analysis". You can rename it to something like "Build Rust library" by double clicking on the heading. ...

      ```sh
      cd "${PROJECT_DIR}"/../shared
      bash "${PROJECT_DIR}/bin/compile-library.sh" shared "$build_variant"
      ```

   1. add `./shared/src/shared.udl` to "Compile Sources" (using the "add other" button). Select"Copy items if needed" and "Create folder references"
   1. add a "Headers" section that includes `./iOS/generated/sharedFFI.h` as a "Public" header
   1. add `./target/debug/libshared.a` to the "Link Binary with Libraries" section (this is the wrong target, but the library search paths, which we set above, should resolve this, for more info see the blog post linked above ([this post](https://blog.mozilla.org/data/2022/01/31/this-week-in-glean-building-and-deploying-a-rust-library-on-ios/)))

1. add a `class` for the callback to get Platform details ...

   ```swift
   class GetPlatform: Platform {
      func get() -> String {
         return UIDevice.current.systemName + " " + UIDevice.current.systemVersion
      }
   }

   ```

1. call the `addForPlatform` function, e.g. in a Text UI component ...

   ```swift
   Text(try! addForPlatform(1, 2, GetPlatform()))
   ```

### Yew web app

1. Install [`trunk`](https://github.com/thedodd/trunk)
1. Create a new rust binary ...

   ```sh
   cargo new web
   ```

1. Edit [`./Cargo.toml`](./Cargo.toml) to add the new app to the Cargo workspace ...

   ```toml
   [workspace]
   members = ["shared", "web"]
   ```

1. Add [`yew`](https://yew.rs/), and the shared library, as dependencies in [`./web/Cargo.toml`](./web/Cargo.toml) ...

   ```toml
   [dependencies]
   yew = "0.19.3"
   shared = { path = "../shared" }
   ```

1. Add [`./web/index.html`](./web/index.html) ...

   ```html
   <!DOCTYPE html>
   <html>
     <head>
       <meta charset="utf-8" />
       <title>Yew App</title>
     </head>
   </html>
   ```

1. Edit [`./web/src/main.rs`](./web/src/main.rs), for example ...

   ```rust
   use shared::add;
   use yew::prelude::*;

   struct WebPlatform;
   impl Platform for WebPlatform {
      fn get(&self) -> Result<String, PlatformError> {
            let navigator = window().unwrap().navigator();
            let agent = navigator.user_agent().unwrap_or_default();
            let parser = Parser::new();
            Ok(parser.parse(&agent).unwrap_or_default().name.to_string())
      }
   }

   #[function_component(HelloWorld)]
   fn hello_world() -> Html {
      html! {
         <p>{"1 + 2 = "}{add_for_platform(1, 2, Box::new(WebPlatform {})).unwrap_or_default()}</p>
      }
   }

   fn main() {
      yew::start_app::<HelloWorld>();
   }
   ```

1. Build and serve the web page ...

   ```sh
   cd ./web
   trunk serve
   ```
# hot-lib-reloader


[![Crates.io](https://img.shields.io/crates/v/hot-lib-reloader)](https://crates.io/crates/hot-lib-reloader)
[![](https://docs.rs/structopt/badge.svg)](https://docs.rs/hot-lib-reloader)
[![CI](https://github.com/rksm/hot-lib-reloader-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/rksm/hot-lib-reloader-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/hot-lib-reloader?color=informational&logo=mit)](/LICENSE.md)

![](doc/hot-reload-demo.gif)

`hot-lib-reloader` is a development tool that allows you to reload functions of a running Rust program.
This allows to do "live programming" where you modify code and immediately see the effects in your running program.

This is build around the [libloading crate](https://crates.io/crates/libloading) and will require you to put code you want to hot-reload inside a Rust library (dylib). For a detailed discussion about the idea and implementation see [this blog post](https://robert.kra.hn/posts/hot-reloading-rust).


## Table of contents:

- [Usage](#usage)
    - [Example project setup](#example-project-setup)
        - [Executable](#executable)
        - [Library](#library)
        - [Running it](#running-it)
    - [lib-reload events](#lib-reload-events)

- [Usage tips](#usage-tips)
    - [Know the limitations](#know-the-limitations)
        - [No signature changes](#no-signature-changes)
        - [Type changes require some care](#type-changes-require-some-care)
        - [Hot-reloadable functions cannot be generic](#hot-reloadable-functions-cannot-be-generic)
        - [Global state in reloadable code](#global-state-in-reloadable-code)
        - [Rust nightly](#rust-nightly)
    - [Use feature flags to switch between hot-reload and static code](#use-feature-flags-to-switch-between-hot-reload-and-static-code)
    - [Use serialization or generic values for changing types](#use-serialization-or-generic-values-for-changing-types)
    - [Use a hot-reload friendly app structure](#use-a-hot-reload-friendly-app-structure)
    - [Use multiple libraries](#use-multiple-libraries)
    - [Code-completion with rust-analyzer](#code-completion-with-rust-analyzer)

- [Examples](#examples)

- [Known issues](#known-issues)
    - [tracing crate](#tracing-crate)


## Usage

To quicky generate a new project supporting hot-reload you can use a [cargo generate](https://cargo-generate.github.io/cargo-generate/) template: `cargo generate rksm/rust-hot-reload`.


### Example project setup

Assuming you use a workspace project with the following layout:

```output
├── Cargo.toml
└── src
│   └── main.rs
└── lib
    ├── Cargo.toml
    └── src
        └── lib.rs
```


#### Executable

Setup the workspace with a root project named `bin` in `./Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = ["lib"]

[package]
name = "bin"
version = "0.1.0"
edition = "2021"

[dependencies]
hot-lib-reloader = "^0.5"
lib = { path = "lib" }
```

In `./src/main.rs` define a sub-module using the
[`hot_lib_reloader_macro::hot_module`] attribute macro which wraps the functions
exported by the library:

```rust
// The value of `dylib = "..."` should be the library containing the hot-reloadable functions
// It should normally be the crate name of your sub-crate.
#[hot_lib_reloader::hot_module(dylib = "lib")]
mod hot_lib {
    // Reads public no_mangle functions from lib.rs and  generates hot-reloadable
    // wrapper functions with the same signature inside this module.
    hot_functions_from_file!("../lib/src/lib.rs");

    // Because we generate functions with the exact same signatures,
    // we need to import types used
    pub use lib::State;
}

fn main() {
    let mut state = hot_lib::State { counter: 0 };
    // Running in a loop so you can modify the code and see the effects
    loop {
        hot_lib::step(&mut state);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```

#### Library

The library should expose functions. It should set the crate type `dylib` in `./lib/Cargo.toml`:

```toml
[package]
name = "lib"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib", "dylib"]
```

The functions you want to be reloadable should be public and have the `#[no_mangle]` attribute. Note that you can define other function that are not supposed to change without `no_mangle` and you will be able to use those alongside the other functions.

```rust
pub struct State {
    pub counter: usize,
}

#[no_mangle]
pub fn step(state: &mut State) {
    state.counter += 1;
    println!("doing stuff in iteration {}", state.counter);
}
```

#### Running it

1. Start compilation of the library: `cargo watch -w lib -x 'build -p lib'`
2. In another terminal run the executable: `cargo run`

Now change for example the print statement in `lib/lib.rs` and see the effect on the runtime.


In addition, using a tool like [cargo runcc](https://crates.io/crates/runcc) is recommended. This allows to run both the lib build and the application in one go.



### lib-reload events

You can get notified about two kinds of events using the methods provided by [`LibReloadObserver`]:

- [`wait_for_about_to_reload`](LibReloadObserver::wait_for_about_to_reload) the watched library is about to be reloaded (but the old version is still loaded)
- [`wait_for_reload`](LibReloadObserver::wait_for_reload) a new version of the watched library was just reloaded

This is useful to run code before and / or after library updates. One use case is to serialize and then deserialize state another one is driving the application.

To continue with the example above, let's say instead of running the library function `step` every second we only want to re-run it when the library has changed.
In order to do that, we first need to get hold of the `LibReloadObserver`. For that we can expose a function `subscribe()` that is annotated with the `#[lib_change_subscription]` (that attribute tells the `hot_module` macro to provide an implementation for it):

```rust
#[hot_lib_reloader::hot_module(dylib = "lib")]
mod hot_lib {
    /* code from above */

    // expose a type to subscribe to lib load events
    #[lib_change_subscription]
    pub fn subscribe() -> hot_lib_reloader::LibReloadObserver {}
}
```

And then the main function just waits for reloaded events:

```rust
fn main() {
    let mut state = hot_lib::State { counter: 0 };
    let lib_observer = hot_lib::subscribe();
    loop {
        hot_lib::step(&mut state);
        // blocks until lib was reloaded
        lib_observer.wait_for_reload();
    }
}
```

How to block reload to do serialization / deserialization is shown in the [reload-events example](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples/reload-events).



## Usage tips


### Know the limitations

Reloading code from dynamic libraries comes with a number of caveats which are discussed in some detail [here](https://robert.kra.hn/posts/hot-reloading-rust/#caveats-and-asterisks).


#### No signature changes

When the signature of a hot-reloadable function changes, the parameter and result types the executable expects differ from what the library provides. In that case you'll likely see a crash.


#### Type changes require some care

Types of structs and enums that are used in both the executable and library cannot be freely changed. If the layout of types differs you run into undefined behavior which will likely result in a crash.

See [use serialization](#use-serialization-or-generic-values-for-changing-types) for a way around it.


#### Hot-reloadable functions cannot be generic

Since `#[no_mangle]` does not support generics, generic functions can't be named / found in the library.

#### Global state in reloadable code

If your hot-reload library contains global state (or depends on a library that does), you will need to re-initialize it after reload. This can be a problem with libraries that hide the global state from the user. If you need to use global state, keep it inside the executable and pass it into the reloadable functions if possible.

#### Rust nightly

You currently need to use Rust nightly to run hot-reloadable code. The reason for that is that we currently need [the `proc_macro::Span` feature](https://github.com/rust-lang/rust/issues/54725). We are looking into a solution that works on stable.



### Use feature flags to switch between hot-reload and static code

See the [reload-feature example](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples/reload-feature) for a complete project.

Cargo allows to specify optional dependencies and conditional compilation through feature flags.
When you define a feature like this

```toml
[features]
default = []
reload = ["dep:hot-lib-reloader"]

[dependencies]
hot-lib-reloader = { version = "^0.5", optional = true }
```

and then conditionally use either the normal or the hot module in the code calling the reloadable functions you can seamlessly switch between a static and hot-reloadable version of your application:

```rust
#[cfg(feature = "reload")]
use hot_lib::*;
#[cfg(not(feature = "reload"))]
use lib::*;

#[cfg(feature = "reload")]
#[hot_lib_reloader::hot_module(dylib = "lib")]
mod hot_lib { /*...*/ }
```

To run the static version just use `cargo run` the hot reloadable variant with `cargo run --features reload`.


### Use serialization or generic values for changing types

If you want to iterate on state while developing you have the option to serialize it. If you use a generic value representation such as [serde_json::Value](https://docs.rs/serde_json/latest/serde_json/value/enum.Value.html), you don't need string or binary formats and typically don't even need to clone anything.

Here is an example where we crate a state container that has an inner `serde_json::Value`:

```rust
#[hot_lib_reloader::hot_module(dylib = "lib")]
mod hot_lib {
    pub use lib::State;
    hot_functions_from_file!("../lib/src/lib.rs");
}

fn main() {
    let mut state = hot_lib::State {
        inner: serde_json::json!(null),
    };

    loop {
        state = hot_lib::step(state);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```

In the library we are now able to change the value and type layout of `InnerState` as we wish:


```rust
#[derive(Debug)]
pub struct State {
    pub inner: serde_json::Value,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct InnerState {}

#[no_mangle]
pub fn step(state: State) -> State {
    let inner: InnerState = serde_json::from_value(state.inner).unwrap_or(InnerState {});

    // You can modify the InnerState layout freely and state.inner value here freely!

    State {
        inner: serde_json::to_value(inner).unwrap(),
    }
}
```


Alternatively you can also do the serialization just before the lib is to be reloaded and deserialize immediately thereafter. This is shown in the [reload-events example](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples/reload-events).



### Use a hot-reload friendly app structure

Whether or not hot-reload is easy to use depends on how you architect your app. In particular, the ["functional core, imparative shell" pattern](https://www.destroyallsoftware.com/screencasts/catalog/functional-core-imperative-shell) makes it easy to split state and behavior and works well with `hot-lib-reloader`

For example, for a simple game where you have the main loop in your control, setting up the outer state in the main function and then passing it into a `fn update(state: &mut State)` and a `fn render(state: &State)` is a straightforward way to get two hot-reloadable functions.

But even when using a framework that takes control, chances are that there are ways to have it call hot-reloadable code. The [bevy example](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples/bevy) where system functions can be made hot-reloadable, shows how this can work.

You can also wait for lib changes and run code afterwards.
This is useful if you want to iterate over a program that only produces output once, for example work on a data analysis or visualization.
This snippet shows how:

```rust
#[hot_module(dylib = "lib")]
mod hot_lib {
    /*...*/
    #[lib_change_subscription]
    pub fn lib_reload_rx() -> mpsc::Receiver<ChangedEvent> {}
}

loop {
    hot_lib::step();
    // waits for a lib reload:
    let event = rx.recv()?;
}
```


### Code-completion with rust-analyzer

Functions that get injected with automatic code generation that happens with `hot_functions_from_file!("path/to/file.rs");` won't be picked up by rust-analyzer and thus you don't have auto-completion for them.

There is a different syntax available that allows you to define reloadable functions inline so that they get picked up by rust-analyzer:

```rust
#[hot_lib_reloader::hot_module(dylib = "lib")]
mod hot_lib {
    #[hot_functions]
    extern "Rust" {
        pub fn step();
    }
}
```


### Debugging

If your `hot_module` gives you a strange compilation error, try `cargo expand` to see what code is generated.



## Examples

Examples can be found at [rksm/hot-lib-reloader-rs/examples](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples).

- [minimal](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples/minimal): Bare-bones setup.
- [reload-feature](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples/reload-feature): Use a feature to switch between dynamic and static version.
- [serialized-state](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples/serialized-state): Shows an option to allow to modify types and state freely.
- [reload-events](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples/reload-events): How to block reload to do serialization / deserialization.
- [bevy](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples/bevy): Shows how to hot-reload bevy systems.



## Known issues

### tracing crate

When used with the `tracing` crate multiple issues can occur:
- When `tracing` is used in the library that is reloaded the app sometimes crashes with `Attempted to register a DefaultCallsite that already exists!`
- When used in combination with bevy, `commands.insert(component)` operations stop to work after a reload, likely because of internal state getting messed up.

If you can, don't use `hot-lib-reloader` in combination with `tracing`.



## License

[MIT](https://github.com/rksm/hot-lib-reloader-rs/blob/hot-module/LICENSE)


License: MIT

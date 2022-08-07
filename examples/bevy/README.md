This example shows how to use hot-reloading with bevy.

# Usage

To run without hot reloading just use `cargo run`.

To run the example with hot-reload enabled run these two commands in parallel:

## Linux and macOS


```shell
$ cargo watch -w systems -w components -x "build -p systems --features dynamic"
$ cargo run --features reload
```

Alternatively with a tool like [runcc](https://crates.io/crates/runcc) you can run this as a single commands: `cargo runcc -c`

## Windows

```shell
$ cargo watch -w systems -w components -x "build -p systems --features dynamic"
$ env CARGO_TARGET_DIR="target-bin" cargo run --features reload
```

Alternatively with [runcc](https://crates.io/crates/runcc): `cargo runcc -c runcc-windows.yml`

[Why is this different from Linux / MacOS?](#library-files-on-windows-get-locked-while-the-app-is-running-and-there-is-a-permission-error-when-they-change)


# Automatically generating reloadable system functions

The `define_lib_reloader!` macro allows for a property `generate_bevy_systems` that does automatically generate systems in the context of the main app that make those systems easily reloadable and avoids you writing boilerplate funcions. For the detailed explanation about that see ["How to use it with Bevy"](https://robert.kra.hn/posts/hot-reloading-rust/#how-to-use-it-with-bevy).

The short version of it is here:

Assuming you want to hot-reload bevy systems, place those into a separate library such as [`systems/src/lib.rs`](./systems/src/lib.rs). When [defining the lib-reloader](./src/main.rs), set `generate_bevy_systems: true` similar to:

```rust
hot_lib_reloader::define_lib_reloader! {
    unsafe SystemsReloader {
        lib_name: "systems",
        source_files: ["../systems/src/lib.rs"],
        generate_bevy_systems: true,
    }
}
```

Assuming `systems/src/lib.rs` defines a system like

```rust
#[no_mangle]
pub fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) { /*...*/ }
```

`generate_bevy_systems` will

This will generate a proxying function like

```
pub fn player_movement_system(
    loader: Res<SystemsReloader>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    loader.player_movement_system(keyboard_input, query, time);
}
```

in the same file. This means that the injected `SystemsReloader` will simply forward the call to the library.

In order for the `SystemsReloader` to be available as a resource, define and use a startup system like


```rust
app.add_startup_system(setup_hot_reload)

// ...

pub fn setup_hot_reload(mut commands: Commands) {
    let lib = SystemsReloader::new().expect("init lib loader");
    commands.insert_resource(lib);
    commands.insert_resource(LibLoaderUpdateTimer(Timer::from_seconds(1.0, true)));
}
```

And add a system to repeatedly update the library that uses those resources:

```rust
app.add_system(update_lib)

// ...

fn update_lib(
    time: Res<Time>,
    mut lib: ResMut<SystemsReloader>,
    mut timer: ResMut<LibLoaderUpdateTimer>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        lib.update().expect("update lib");
    }
}
```


# Known issues

See the [list of known issues in the main readme](https://github.com/rksm/hot-lib-reloader-rs#known-issues).

Bevy specific issues:

## Define your components and state outside of the reloadable systems crate

To make changes to the systems not break the type ids of components, making a `components` sub-crate is recommended. This way, they are a separate compilation unit. Otherwise component queries might suddenly be empty after code changes.


## library files on Windows get locked while the app is running and there is a permission error when they change

On Windows, dll files like `systems.dll` will get locked when they are in use by a program. This is a problem for the hot-reloader as it expects those files to change. The setup provided here is careful to avoid these issues. In particular, the following can create a bevy-specific problem:

When Bevy is used with the `dynamic` feature (`bevy = { version = "0.8.0", features = ["dynamic"] }`) there will also be (in addition to `systems.dll` produced `hot-lib-reloader`) a `bevy_dylib.dll`. With `dynamic` enabled the bevy executable now loads all dlls in the target directory — even though the hot-reloader

There are two solutions:

1. Do not use bevy's `dynamic` feature. This makes it work like on Linux and macOS. But the longer compile times ared reducing the usefulness of hot-reload.
2. Keep using `dynamic` but with two different target directories for the lib and executable. This is what `env CARGO_TARGET_DIR=target-bin` (see [usage](#usage)) does and the recommended solution.

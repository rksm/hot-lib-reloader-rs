This example shows how to use hot-reloading with bevy.

# Usage

To run without hot reloading just use `cargo run`.

To run the example with hot-reload enabled run these two commands in parallel:

## Linux and macOS

```shell
$ cargo watch -w systems -x 'build -p systems'
$ cargo watch -i systems -x 'run --features reload'
```

## Windows

```shell
$ cargo watch -w systems -x 'build -p systems'
$ env CARGO_TARGET_DIR=target-bin cargo watch -i systems -x 'run --features reload'
```

The reason why Windows usage differs: In `examples/bevy/systems/Cargo.toml` the bevy dependency uses the `dynamic` feature: `bevy = { version = "0.8.0", features = ["dynamic"] }`. This is to speed up recompilation when the systems library changed. 

This means that in addition to `systems.dll` (produced `hot-lib-reloader`) there is `bevy_dylib.dll`. With `dynamic` enabled the bevy executable now loads all dlls in the target directory. Windows will lock all used library files, they can't be modified (or deleted) while still in use. This is different on other operating systems. `hot-lib-reloader` actually creates a copy of `systems.dll` to avoid this exact issue. But with bevy loading it, `systems.dll` can't be replaced.

There are two solutions:

1. Do not use bevy's `dynamic` feature. This makes it work like on Linux and macOS. But the longer compile times ared reducing the usefulness of hot-reload.
2. Keep using `dynamic` but with two different target directories for the lib and executable. This is what `env CARGO_TARGET_DIR=target-bin` does and the recommended solution.



# `generate_bevy_systems` flag of `define_lib_reloader!`

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

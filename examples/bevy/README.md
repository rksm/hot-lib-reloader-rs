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
$ cargo run --features reload --target-dir "target-bin"
```

Alternatively with [runcc](https://crates.io/crates/runcc): `cargo runcc -c runcc-windows.yml`

[Why is this different from Linux / MacOS?](#library-files-on-windows-get-locked-while-the-app-is-running-and-there-is-a-permission-error-when-they-change)



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
2. Keep using `dynamic` but with two different target directories for the lib and executable. This is the the recommended solution (see [usage](#usage)).

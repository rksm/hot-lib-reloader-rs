#![allow(clippy::needless_doctest_main)]

/*!

[![Crates.io](https://img.shields.io/crates/v/hot-lib-reloader)](https://crates.io/crates/hot-lib-reloader)
[![](https://docs.rs/structopt/badge.svg)](https://docs.rs/hot-lib-reloader)
[![License](https://img.shields.io/crates/l/hot-lib-reloader?color=informational&logo=mit)](/LICENSE.md)

<!--
[![Crates.io](https://img.shields.io/crates/d/hot-lib-reloader)](https://crates.io/crates/hot-lib-reloader)
 -->

A simple crate around [libloading](https://crates.io/crates/libloading) that can be used to watch Rust libraries (dylibs) and will reload them again when they have changed.
Useful for changing code and seeing the effects without having to restart the app.

![](doc/hot-reload-demo.gif)

Note: This is meant to be used for development! Don't use it in production!

Also currently [`proc_macro::Span`](https://github.com/rust-lang/rust/issues/54725) is required and you will need to run hot-reloadable code with Rust nightly.

# What it does

1. Watch a dynamically loadable library you specify, reload it when it changes.

2. Generates a type that provides methods to dynamically call the functions exposed by that library.
You specify Rust source files that contain functions exported in the library above.
`hot-lib-reloader` will parse those, find those functions and their signatures and use it to create methods you can call (instead of manually having to query for a library symbol).

For a detailed discussion see https://robert.kra.hn/posts/hot-reloading-rust/.

# Usage

Assuming you use a workspace with the following layout:

```
├── Cargo.toml
└── src
│   └── main.rs
└── lib
    ├── Cargo.toml
    └── src
        └── lib.rs
```

## lib

The library should expose functions and state. It should have specify `dylib` as crate type. The `./lib/Cargo.toml`:

```toml
[package]
name = "lib"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib", "dylib"]
```

And `./lib/lib.rs`

```
#[no_mangle]
pub fn do_stuff() {
    println!("doing stuff");
}
```

## bin

In the binary, use the lib and lot `hot-lib-reloader`. `./Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = ["lib"]

[package]
name = "bin"
version = "0.1.0"
edition = "2021"

[dependencies]
hot-lib-reloader = "^0.4"
lib = { path = "lib" }
```

You can then define and use the lib reloader like so (`./src/main.rs`):

```no_run
hot_lib_reloader::define_lib_reloader! {
    unsafe MyLibLoader {
        // Will look for "liblib.so" (Linux), "lib.dll" (Windows), ...
        lib_name: "lib",
        // Where to load the reloadable functions from,
        // relative to current file:
        source_files: ["../../lib/src/lib.rs"]
        // You can optionally specify manually:
        // functions: {
        //     fn do_stuff();
        // }
    }
}

fn main() {
    let mut lib = MyLibLoader::new().expect("init lib loader");

    loop {
        lib.update().expect("lib update"); // will reload lib on change

        lib.do_stuff();

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

```

Above is the part that matters: A new type `MyLibLoader` is created that provides a method `MyLibLoader::do_stuff(&self)`.
The method is automatically generated from `lib::do_stuff()`.
Indeed, if we were to add a method `lib::add_numbers(a: i32, b: i32) -> i32`, a method `MyLibLoader::add_numbers(&self, a: i32, b: i32) -> i32` would be generated. Etc.

Note: If you prefer to not use macros, the macro-free version of the code above is:

```no_run
use hot_lib_reloader::LibReloader;

fn main() {
    let mut lib = LibReloader::new("target/debug", "lib").expect("initial load the lib");

    loop {
        lib.update().expect("lib update"); // will reload lib on change

        unsafe {
            lib.get_symbol::<fn()>(b"do_stuff\0")
                .expect("Load do_stuff()")();
        };

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```

## Running it

To start compilation of the library:

```shell
cargo watch -w lib -x 'build -p lib'
```

And in addition to that start compilation of the binary with reload enabled:

```shell
cargo watch -w bin -x run
```

A change that you now make to `lib/lib.rs` will have an immediate effect on the app.



# More examples

Examples can be found at [rksm/hot-lib-reloader-rs/examples](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples).

- [minimal](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples/minimal): Bare-bones setup.
- [reload-feature](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples/reload-feature): Use a feature to switch between dynamic and static version.
- [bevy](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples/bevy): Shows how to hot-reload bevy systems.


# Known issues

## tracing crate

When used with the `tracing` crate multiple issues can occur:
- When `tracing` is used in the library that is reloaded the app sometimes crashes with `Attempted to register a DefaultCallsite that already exists!`
- When used in combination with bevy, `commands.insert(component)` operations stop to work after a reload, likely because of internal state getting messed up.

If you can, don't use `hot-lib-reloader` in combination with `tracing`.



# License

*/

mod error;
mod lib_reloader;

pub use error::HotReloaderError;
pub use hot_lib_reloader_macro::{define_lib_reloader, hot_module};
pub use lib_reloader::LibReloader;

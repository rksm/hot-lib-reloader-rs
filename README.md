# hot-lib-reloader


A simple crate around [libloading](https://crates.io/crates/libloading) that can be used to watch Rust libraries (dylibs) and will reload them again when they have changed.
Useful for changing code and seeing the effects without having to restart the app.

Note: This is meant to be used for development! Don't use it in production!

## Usage

Assuming you use a workspace with the following layout:

- Cargo.toml
- lib/Cargo.toml
- lib/src/lib.rs
- bin/Cargo.toml
- bin/src/main.rs

### lib

The library should expose functions and state. It should have specify `dylib` as crate type. The `lib/Cargo.toml`:

```toml
[package]
name = "lib"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib", "dylib"]
```

And `lib/lib.rs`

```rust
#[no_mangle]
pub fn do_stuff() {
    println!("doing stuff");
}
```

### bin

In the binary, use the lib and lot `hot-lib-reloader`. The `bin/Cargo.toml`:

```toml
[package]
name = "bin"
version = "0.1.0"
edition = "2021"

[dependencies]
lib = { path = "../lib" }
hot-lib-reloader = "0.3.0"
```

You can then define and use the lib reloader like so:

```rust
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

Note: If you prefer to not use macros, the macro-free version of the code above is:

```rust
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

### Running it

To start compilation of the library:

```shell
cargo watch -w lib -x build
```

And in addition to that start compilation of the binary with reload enabled:

```shell
cargo watch -w bin -x run
```

A change that you now make to `lib/lib.rs` will have an immediate effect on the app.


## More examples

Examples can be found at [rksm/hot-lib-reloader-rs/examples](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples).

- [minimal](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples/minimal): Bare-bones setup.
- [reload-feature](https://github.com/rksm/hot-lib-reloader-rs/tree/master/examples/reload-feature): Use a feature to switch between dynamic and static version.


License: MIT

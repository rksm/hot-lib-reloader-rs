# Rust hot lib-reloader

A simple crate around [libloading](https://crates.io/crates/libloading) that can be used to watch Rust libraries (dylibs) and will reload them again when they have changed.
Useful for changing code and seeing the effects without having to restart the app.

Note: This is meant to be used for development! Don't use it in production!

## Usage

Assuming you use a workspace with the following layout:

- Cargo.toml
- crates/lib/Cargo.toml
- crates/lib/src/lib.rs
- crates/bin/Cargo.toml
- crates/bin/src/main.rs

### lib

The library should expose functions and state. It should have specify `dylib` as crate type. The `crates/lib/Cargo.toml`:

```toml
[package]
name = "lib"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib", "dylib"]
```

And `crates/lib/lib.rs`

```rust
#[no_mangle]
pub extern "C" fn do_stuff() {
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
hot-lib-reloader = { path = "../../../hot-lib-reloader", optional = true }

[features]
default = []
reload = ["dep:hot-lib-reloader"]
```

In the main function check if the lib has changed and call the exported function:

```rust
#[cfg(feature = "reload")]
use hot_lib_reloader::LibReloader;

fn main() {
    #[cfg(feature = "reload")]
    let mut lib_loader = LibReloader::new("target/debug", "liblib").expect("initial load the lib");

    loop {
        #[cfg(feature = "reload")]
        lib_loader.update().expect("lib update"); # will reload lib on change

        #[cfg(feature = "reload")]
        unsafe {
            lib_loader
                .get_symbol::<fn()>(b"do_stuff\0")
                .expect("Load do_stuff()")();
        };
        #[cfg(not(feature = "reload"))]
        lib::do_stuff();

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```

### Running it

To start compilation of the library:

```shell
cargo watch -w crates/lib -x build
```

And in addition to that start compilation of the binary with reload enabled:

```shell
cargo watch -w crates/bin -x 'run --features reload'
```

A change that you now make to `crates/lib/lib.rs` will have an immediate effect on the app.

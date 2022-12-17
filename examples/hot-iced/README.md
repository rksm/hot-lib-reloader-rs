# hot-iced

Using hot-lib-reloader with [iced](https://github.com/iced-rs/iced/).

Usage:

1. Start the application using `cargo runcc -c` (you can install `cargo-runcc` with `cargo install runcc`).
2. Modify the update function at `lib/src/lib.rs`, e.g. insert a `println!()`. You should see the the library reloading and the update now triggering an output.

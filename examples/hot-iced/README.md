# hot-iced

__NOTE: This currently only works on macOS. See [this issue](https://github.com/rksm/hot-lib-reloader-rs/issues/25) for more details. Contributions to fix this on other platforms are very welcome!__

Using hot-lib-reloader with [iced](https://github.com/iced-rs/iced/).

Usage:

1. Start the application using `just run`.
2. Modify the update function at `lib/src/lib.rs`, e.g. insert a `println!()`. You should see the the library reloading and the update now triggering an output.

Will add `#[no_mangle]` to the item it is applied but only in debug mode.

This is useful for use with [hot-lib-reloader](https://crates.io/crates/hot-lib-reloader) to conditionally expose library functions to the lib reloader only in debug mode.
In release mode where a build is to be expected fully static, no additional penalty is paid.

```rust
#[no_mangle_if_debug]
fn func() {}
```

will expand to

```rust
#[cfg(debug_assertions)]
#[no_mangle]
fn func() {}

#[cfg(not(debug_assertions))]
fn func() {}
```

### License

MIT

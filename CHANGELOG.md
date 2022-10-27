# Changelog

This package tries to adhere to [semver](https://semver.org/).

## [0.6.4]
### Change the default `lib_dir` and allow expressions instead of just string literals for `lib_dir` and `dylib`
This changes the defaults of `lib_dir` and `dylib` properties of the `hot_module` macro.
Previously, specifying `#[hot_module(dylib = "lib")]` would expand into `#[hot_module(dylib = "lib", lib_dir = "target/debug")]` (debug build) or `#[hot_module(dylib = "lib", lib_dir = "target/release")]`.
Now the `lib_dir` value is by defaults are:
- `#[hot_module(dylib = "lib", lib_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/target/debug"))]` and
- `#[hot_module(dylib = "lib", lib_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/target/release"))]`

Also, both `dylib` and `lib_dir` can now be expressions (that should evaluate to something string like) instead of just literal strings.

Changing the defaults actually constitutes a breaking change.
I'm not bumping the minor version as I _assume_ no one depended on that.
If this is not correct and that change breaks something I hereby sincerly apologies.
You can get the old behavior back with

```rust
#[hot_module(
    dylib = "lib",
    lib_dir = if cfg!(debug_assertions) { "target/debug" } else { "target/release" },
    file_watch_debounce = 500
)]
/* ... */
```

This change should fix the case when a program changes the current working directory and library loading doesn't work anymore thereafter.
This was first reported in https://github.com/rksm/hot-lib-reloader-rs/issues/22.

## [0.6.3]
### fix `wait_for_about_to_reload` and `wait_for_reload` when no hot function was called.
As [reported](https://github.com/rksm/hot-lib-reloader-rs/issues/21), when using the wait functions but not calling a hot-reloadable library function, the wait functions would continue to block even if the library was changed.
This release fixes that, calling `wait_for_about_to_reload` and `wait_for_reload` should always return once a library change was made.

## [0.6.2]
### codesign libraries on macos
On macos [spurious crashes](https://github.com/rksm/hot-lib-reloader-rs/issues/15) can happen after reloading the library. In order to avoid this, we will codesign the library when the `codesign` binary is available.


## [0.6.1]
### expose simple update check
It is now possible to use a simple update check to test if the library was reloaded:

```rust
#[hot_lib_reloader::hot_module(dylib = "lib")]
mod hot_lib {
    /* ... */
    #[lib_updated]
    pub fn was_updated() -> bool {}
}
```

This can simplify code that wants to just figure out if a change happened and would have had to use the version counter for that.
Note that this function only returns `true` _once` after a reload. The next time you call that function, if no reload has occurred again, it will return `false`.


## [0.6.0]
### Breaking change
`hot_functions_from_file!("path/to/file.rs")` and `define_lib_reloader!(...)` expect file path to be __relative to the project root__, not relative to the file they appear in.

See https://github.com/rksm/hot-lib-reloader-rs/issues/13 for the background.

### Other changes
- The hot-lib-reloader won't log to stdout/stderr anymore in the running app. It now fully uses the `log` crate. Use `RUST_LOG=hot_lib_reloader=trace` for debugging.
- Fix macro expansion and code completion with rust-analyzer
- No more requirement to use Rust nightly!
- Version counter can be optionally exposed from `hot_module`:
```rust
#[hot_module(dylib = "lib")]
mod hot_lib {
    /* ... */

    #[lib_version]
    pub fn version() -> usize {}
}
```

- Allow to specify the debounce duration for file changes in milliseconds. This is 500ms by default. If you see multiple updates triggerd for one recompile (can happen the library is very large), increase that value. You can try to decrease it for faster reloads. With small libraries / fast hardware 50ms or 20ms should work fine.
```rust
#[hot_module(dylib = "lib", file_watch_debounce = 50)]
/* ... */
```

`hot_lib::version()` will then return a monotonically increasing number, starting with 0.
Each library reload will increase the counter.

## [0.5.6]
Make the logging about attempted lib-loader write locks less verbose.

## [0.5.5]
### Fix locking of lib loader
We used a convoluted half-baked ref counting scheme for access to symbols while
not needing to mutex lock the lib loader during a call (so that recursive calls
work)

This has been cleaned up with the use of `RwLock`s instead. This should also fix
spurious crashes during hot updates that were likely caused by symbols actually
being used (b/c the prev solution wasn't really thread safe).

## [0.5.4]
### `#[no-mangle-if-debug]`
Also add a [no-mangle-if-debug crate](https://github.com/rksm/hot-lib-reloader-rs/tree/master/macro-no-mangle-if-debug) that allows to `#[no_mangle]` functions but only in debug mode. The use of this is optional and nothing about hot-lib-reloader itself changes. This addresses https://github.com/rksm/hot-lib-reloader-rs/issues/10.


## [0.5.3]
`#[lib_change_subscription]` now returns the `LibReloadObserver` type that wraps the mpsc channel.
```rust
#[lib_change_subscription]
pub fn subscribe() -> hot_lib_reloader::LibReloadObserver {}
```
It provides multiple methods to wait for about-to-reload and reloaded events.

Also add lots of documentation, a test, and a example around that.


## [0.5.2]
Added support for getting lib reload events.
Inside a `hot_module`, the following creates a function that can be used to subscribe:
```rust
#[lib_change_subscription]
pub fn subscribe() -> std::sync::mpsc::Receiver<hot_lib_reloader::ChangedEvent> {}
```

## [0.5.1]
### Fixes file observation issues
Using file hashes to figure out when a file actually changed, `notify` alone isn't enough.
The file change strategies on the different OSes seem to be quite different.
The lib might be removed or re-linked or simply overwritten.
The file change events in each case are quite different and in the case of removal, `notify` does not always seem to get events when the file is recreated.
In addition, on macOS copying `lib*.dylib` to `lib*-hot.dylib` seems to trigger a file change event for `lib*.dylib`...
So using a hash is the simplest way to figure out if the dylib actually changed and provides a reliable way to trigger a recompile.

## [0.5.0]
### Added
- added the `#[hot_module]` attribute macro
- manage the reloader internally
- provide hot reloadable functions as part of the hot_module with the identical interface they normally have
- tests & CI setup
- more documentation and examples
- Subscribe to lib changes with `__lib_loader_subscription()`
### Changed
- deprecated `define_lib_reloader`
- lib reloader updates via events


## [0.4.4]
### Changed
- Fix how library files get renamed
- Add note to bevy example for how to use it on Windows

## [0.4.3]
### Changed
- Fix reloads on macOS M1
- Markdown fixes

## [0.4.2]
### Added
- demo gif in readme

## [0.4.0]
### Changed
- Reimplemented the `define_lib_reloader` macro as proc macro. It has a somewhat different syntax now (breaking change)
### Added
- `define_lib_reloader` now allows to load function signatures from source files
- support for hot-reloading bevy systems


## [0.3.0]
### Added
- Convenience macro `define_lib_reloader!`

## [0.2.0]
### Changed
- Windows support
- Don't load from original lib, only reload from single lib file

## [0.1.0]
### Added
- `LibReloader`, providing `new()`, `update()`, `get_symbol()`.

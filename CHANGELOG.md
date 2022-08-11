# Changelog

This package tries to adhere to [semver](https://semver.org/).

## [0.5.5]
### Fix locking of lib loader
We used a convoluted half-baked ref counting scheme for access to symbols while
not needing to mutex lock the lib loader during a call (so that recursive calls
work)

This has been cleaned up with the use of `RwLock`s instead. This should also fix
spurious crashes during hot updates that were likely caused by symbols actually
being used (b/c the prev solution wasn't really thread safe).


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

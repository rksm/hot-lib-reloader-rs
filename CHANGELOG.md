# Changelog

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

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

hot-lib-reloader is a Rust development tool that enables hot-reloading of functions in running programs. It works by loading code from dynamic libraries (dylibs) and reloading them when changes are detected, allowing for live programming workflows.

## Core Architecture

### Main Components

1. **LibReloader** (`src/lib_reloader.rs`): Core library loading logic using libloading crate to manage dynamic library lifecycle
2. **Hot Module Macro** (`macro/src/hot_module/`): Procedural macro that generates wrapper modules with hot-reloadable functions
3. **Code Signing** (`src/codesign.rs`): macOS-specific code signing for reloaded libraries
4. **File Watching**: Uses notify crate with debouncing to detect library changes

### Key Design Patterns

- **Shadow Libraries**: Creates copies of dylibs with unique names to avoid file locking issues
- **Function Wrapping**: Generated wrapper functions handle library reloading transparently
- **Event System**: LibReloadObserver provides hooks for serialization around reloads

## Development Commands

```bash
# Enter nix development shell (if using nix)
nix develop

# Run all checks (format, lint, test)
just check

# Run tests
just test
# Or with nextest
cargo nextest run
cargo test --doc

# Run linting
just lint
# Or directly
cargo clippy --all-features -- -D warnings

# Format code
just fmt

# Check formatting
just fmt-check
```

## Testing Approach

- Unit tests in `tests/lib-loader-test.rs`
- Integration examples in `examples/` directory
- Test library in `tests/lib_for_testing/`
- Use `cargo nextest` for parallel test execution when available

## Important Implementation Details

### Hot Module Macro Usage
The `#[hot_module]` macro requires:
- `dylib` parameter specifying the library name
- `hot_functions_from_file!()` to import functions
- Functions must be `#[unsafe(no_mangle)]` and public in the library

### Platform-Specific Considerations
- **macOS**: Requires codesigning via XCode command line tools
- **File paths**: Shadow libraries created in temp directory or alongside original
- **Debouncing**: Default 500ms, configurable via `file_watch_debounce`

### Common Pitfalls to Avoid
- Function signatures cannot change without restart
- Type layouts must remain compatible between reloads
- Global state in libraries requires re-initialization
- Generic functions cannot be hot-reloaded

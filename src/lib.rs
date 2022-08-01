#![allow(clippy::needless_doctest_main)]

/*!

A simple crate around [libloading](https://crates.io/crates/libloading) that can be used to watch Rust libraries (dylibs) and will reload them again when they have changed.
Useful for changing code and seeing the effects without having to restart the app.

Note: This is meant to be used for development! Don't use it in production!

# Usage

Assuming you use a workspace with the following layout:

- Cargo.toml
- lib/Cargo.toml
- lib/src/lib.rs
- bin/Cargo.toml
- bin/src/main.rs

## lib

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

```
#[no_mangle]
pub extern "C" fn do_stuff() {
    println!("doing stuff");
}
```

## bin

In the binary, use the lib and lot `hot-lib-reloader`. The `bin/Cargo.toml`:

```toml
[package]
name = "bin"
version = "0.1.0"
edition = "2021"

[dependencies]
lib = { path = "../lib" }
hot-lib-reloader = { version = "*", optional = true }
```

You can then define and use the lib reloader like so:

```no_run
hot_lib_reloader::define_lib_reloader!(
    MyLibLoader("target/debug", "lib") {
        fn do_stuff() -> ();
    }
);

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
cargo watch -w lib -x build
```

And in addition to that start compilation of the binary with reload enabled:

```shell
cargo watch -w bin -x run
```

A change that you now make to `lib/lib.rs` will have an immediate effect on the app.

*/

mod macros;

use libloading::Library;
use libloading::Symbol;
use notify::watcher;
use notify::DebouncedEvent;
use notify::RecursiveMode;
use notify::Watcher;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::sync::{atomic, mpsc, Arc};
use std::thread;
use std::time::Duration;
use std::time::Instant;

pub struct LibReloader {
    changed: Arc<atomic::AtomicBool>,
    changed_at: Arc<Mutex<Option<Instant>>>,
    lib: Option<Library>,
    watched_lib_file: PathBuf,
    loaded_lib_file: PathBuf,
}

impl LibReloader {
    pub fn new(
        lib_dir: impl AsRef<Path>,
        lib_name: impl AsRef<str>,
    ) -> Result<Self, Box<dyn Error>> {
        #[cfg(target_os = "macos")]
        let (prefix, ext) = ("lib", "dylib");
        #[cfg(target_os = "linux")]
        let (prefix, ext) = ("lib", "so");
        #[cfg(target_os = "windows")]
        let (prefix, ext) = ("", "dll");
        let lib_name = format!("{prefix}{}", lib_name.as_ref());
        let watched_lib_file = lib_dir.as_ref().join(&lib_name).with_extension(ext);
        let loaded_lib_file = lib_dir
            .as_ref()
            .join(format!("{lib_name}-hot"))
            .with_extension("dll");
        // We don't load the actual lib because this can get problems e.g. on Windows
        // where a file lock would be held, preventing the lib from changing later.
        std::fs::copy(&watched_lib_file, &loaded_lib_file)?;
        let lib = Some(unsafe { Library::new(&loaded_lib_file) }?);

        let lib_loader = Self {
            watched_lib_file: watched_lib_file.clone(),
            loaded_lib_file,
            lib,
            changed: Arc::new(atomic::AtomicBool::new(false)),
            changed_at: Arc::new(Mutex::new(None)),
        };

        lib_loader.watch(watched_lib_file)?;

        Ok(lib_loader)
    }

    /// Checks if the watched library has changed. If it has, reload it and return
    /// true. Otherwise return false.
    pub fn update(&mut self) -> Result<bool, Box<dyn Error>> {
        if !self.changed.load(Ordering::Relaxed) {
            return Ok(false);
        }
        self.changed.store(false, Ordering::Relaxed);
        self.reload()?;
        let mut changed_at = self.changed_at.lock().unwrap();
        *changed_at = Some(Instant::now());
        Ok(true)
    }

    /// Reload library `self.lib_file`.
    fn reload(&mut self) -> Result<(), Box<dyn Error>> {
        let Self {
            lib,
            watched_lib_file,
            loaded_lib_file,
            ..
        } = self;

        println!("[hot-lib-reloader] reloading lib {watched_lib_file:?}",);

        // Close the loaded lib, copy the new lib to a file we can load, then load it.
        if let Some(lib) = lib.take() {
            lib.close()?;
        }
        std::fs::copy(watched_lib_file, loaded_lib_file)?;
        self.lib = Some(unsafe { Library::new(&self.loaded_lib_file) }?);

        Ok(())
    }

    /// Watch for changes of `lib_file`.
    fn watch(&self, lib_file: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let lib_file = lib_file.as_ref().to_path_buf();
        log::info!(
            "[hot-lib-reloader] start watching changes of file {}",
            lib_file.display()
        );

        let changed = self.changed.clone();
        let last_changed = self.changed_at.clone();

        // File watcher thread. We watch `self.lib_file`, when it changes and we haven't
        // a pending change still waiting to be loaded, set `self.changed` to true. This
        // then gets picked up by `self.update`.
        thread::spawn(move || {
            use DebouncedEvent::*;

            let (tx, rx) = mpsc::channel();
            let mut watcher = watcher(tx, Duration::from_millis(50)).unwrap();
            watcher
                .watch(&lib_file, RecursiveMode::NonRecursive)
                .expect("watch lib file");

            let debounce = Duration::from_millis(500);
            let mut last_change = Instant::now() - debounce;
            let mut signal_change = || {
                let now = Instant::now();

                if last_changed
                    .try_lock()
                    .ok()
                    .and_then(|t| *t)
                    .map(|t| now - t < debounce)
                    .unwrap_or(false)
                {
                    return false;
                }

                log::debug!("[hot-lib-reloader] {} changed", lib_file.display());

                last_change = now;
                changed.store(true, Ordering::Relaxed);
                last_changed.lock().unwrap().replace(Instant::now());
                true
            };

            loop {
                match rx.recv() {
                    Ok(Chmod(_) | Create(_) | Write(_)) => {
                        signal_change();
                    }
                    Ok(Remove(_)) => {
                        // just one hard link removed?
                        if !lib_file.exists() {
                            log::debug!(
                                "[hot-lib-reloader] {} was removed, trying to watch it again...",
                                lib_file.display()
                            );
                        }
                        loop {
                            if watcher
                                .watch(&lib_file, RecursiveMode::NonRecursive)
                                .is_ok()
                            {
                                log::info!("[hot-lib-reloader] watching {}", lib_file.display());
                                signal_change();
                                break;
                            }
                            thread::sleep(debounce);
                        }
                    }
                    Ok(_change) => {
                        // dbg!(change);
                    }
                    Err(err) => {
                        log::error!(
                            "[hot-lib-reloader] file watcher error, stopping reload loop: {err}"
                        );
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Get a pointer to a function or static variable by symbol name. Just a
    /// wrapper around [libloading::Library::get].
    ///
    /// The `symbol` may not contain any null bytes, with the exception of the
    /// last byte. Providing a null-terminated `symbol` may help to avoid an
    /// allocation. The symbol is interpreted as is, no mangling.
    ///
    /// # Safety
    ///
    /// Users of this API must specify the correct type of the function or variable loaded.
    pub unsafe fn get_symbol<T>(&self, name: &[u8]) -> Result<Symbol<T>, Box<dyn Error>> {
        Ok(self.lib.as_ref().unwrap().get(name)?)
    }
}

impl Drop for LibReloader {
    fn drop(&mut self) {
        if self.loaded_lib_file.exists() {
            let _ = std::fs::remove_file(&self.loaded_lib_file);
        }
    }
}

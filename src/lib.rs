//! A simple crate around [libloading](https://crates.io/crates/libloading) that can be used to watch Rust libraries (dylibs) and will reload them again when they have changed.
//! Useful for changing code and seeing the effects without having to restart the app.
//!
//! Note: This is meant to be used for development! Don't use it in production!
//!
//! # Usage
//!
//! Assuming you use a workspace with the following layout:
//!
//! - Cargo.toml
//! - crates/lib/Cargo.toml
//! - crates/lib/src/lib.rs
//! - crates/bin/Cargo.toml
//! - crates/bin/src/main.rs
//!
//! ## lib
//!
//! The library should expose functions and state. It should have specify `dylib` as crate type. The `crates/lib/Cargo.toml`:
//!
//! ```toml
//! [package]
//! name = "lib"
//! version = "0.1.0"
//! edition = "2021"
//!
//! [lib]
//! crate-type = ["rlib", "dylib"]
//! ```
//!
//! And `crates/lib/lib.rs`
//!
//! ```
//! #[no_mangle]
//! pub extern "C" fn do_stuff() {
//!     println!("doing stuff");
//! }
//! ```
//!
//! ## bin
//!
//! In the binary, use the lib and lot `hot-lib-reloader`. The `bin/Cargo.toml`:
//!
//! ```toml
//! [package]
//! name = "bin"
//! version = "0.1.0"
//! edition = "2021"
//!
//! [dependencies]
//! lib = { path = "../lib" }
//! hot-lib-reloader = { version = "*", optional = true }
//!
//! [features]
//! default = []
//! reload = ["dep:hot-lib-reloader"]
//! ```
//!
//! In the main function check if the lib has changed and call the exported function:
//!
//! ```ignore
//! #[cfg(feature = "reload")]
//! use hot_lib_reloader::LibReloader;
//!
//! fn main() {
//!     #[cfg(feature = "reload")]
//!     let mut lib_loader = LibReloader::new("target/debug", "liblib").expect("initial load the lib");
//!
//!     loop {
//!         #[cfg(feature = "reload")]
//!         lib_loader.update().expect("lib update"); # will reload lib on change
//!
//!         #[cfg(feature = "reload")]
//!         unsafe {
//!             lib_loader
//!                 .get_symbol::<fn()>(b"do_stuff\0")
//!                 .expect("Load do_stuff()")();
//!         };
//!         #[cfg(not(feature = "reload"))]
//!         lib::do_stuff();
//!
//!         std::thread::sleep(std::time::Duration::from_secs(1));
//!     }
//! }
//! ```
//!
//! ## Running it
//!
//! To start compilation of the library:
//!
//! ```shell
//! cargo watch -w crates/lib -x build
//! ```
//!
//! And in addition to that start compilation of the binary with reload enabled:
//!
//! ```shell
//! cargo watch -w crates/bin -x 'run --features reload'
//! ```
//!
//! A change that you now make to `crates/lib/lib.rs` will have an immediate effect on the app.

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
    lib: Library,
    lib_file: PathBuf,
    last_loaded_lib_file: Option<PathBuf>,
    reload_counter: usize,
}

impl LibReloader {
    pub fn new(
        lib_dir: impl AsRef<Path>,
        lib_name: impl AsRef<Path>,
    ) -> Result<Self, Box<dyn Error>> {
        #[cfg(target_os = "macos")]
        let ext = "dylib";
        #[cfg(target_os = "linux")]
        let ext = "so";
        let lib_file = lib_dir.as_ref().join(lib_name).with_extension(ext);
        let lib = unsafe { Library::new(&lib_file) }?;
        let lib_loader = Self {
            lib_file: lib_file.clone(),
            lib,
            last_loaded_lib_file: None,
            changed: Arc::new(atomic::AtomicBool::new(false)),
            changed_at: Arc::new(Mutex::new(None)),
            reload_counter: 0,
        };

        lib_loader.watch(lib_file)?;

        Ok(lib_loader)
    }

    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        if self.changed.load(Ordering::Relaxed) {
            self.changed.store(false, Ordering::Relaxed);
            self.reload()?;
            let mut changed_at = self.changed_at.lock().unwrap();
            *changed_at = Some(Instant::now());
        }
        Ok(())
    }

    /// Reload library `self.lib_file`.
    fn reload(&mut self) -> Result<(), Box<dyn Error>> {
        // step 1: copy the base lib file to a new lib file with a unique name
        self.reload_counter += 1;
        let counter = self.reload_counter;
        let base_file = self.lib_file.with_extension("");
        let ext = self
            .lib_file
            .extension()
            .unwrap_or_default()
            .to_string_lossy();
        let new_lib_file = format!("{}-{counter}.{ext}", base_file.display()).into();
        std::fs::copy(&self.lib_file, &new_lib_file)?;

        // step 2: load the new lib file and close the old one
        println!(
            "[hot-lib-reloader] reloading lib {} from file {new_lib_file:?}",
            self.lib_file.display()
        );
        self.lib = unsafe { Library::new(&new_lib_file) }?;

        // step 3: if we were running on a reloaded lib, remove that to not
        // leave files sitting around
        let last_loaded_lib_file = self.last_loaded_lib_file.replace(new_lib_file);
        if let Some(last_loaded_lib_file) = last_loaded_lib_file {
            if last_loaded_lib_file.exists() {
                std::fs::remove_file(last_loaded_lib_file)?;
            }
        }

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
        thread::spawn(move || {
            use DebouncedEvent::*;

            let (tx, rx) = mpsc::channel();
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
                    return;
                }

                log::debug!("[hot-lib-reloader] {} changed", lib_file.display());

                last_change = now;
                changed.store(true, Ordering::Relaxed);
                last_changed.lock().unwrap().replace(Instant::now());
            };
            let mut watcher = watcher(tx, Duration::from_millis(50)).unwrap();
            watcher
                .watch(&lib_file, RecursiveMode::NonRecursive)
                .expect("watch lib file");

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
        Ok(self.lib.get(name)?)
    }
}

impl Drop for LibReloader {
    fn drop(&mut self) {
        if let Some(last_loaded_lib_file) = self.last_loaded_lib_file.take() {
            if last_loaded_lib_file.exists() {
                let _ = std::fs::remove_file(last_loaded_lib_file);
            }
        }
    }
}

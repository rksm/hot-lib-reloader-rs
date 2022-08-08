use libloading::Library;
use libloading::Symbol;
use notify::watcher;
use notify::DebouncedEvent;
use notify::RecursiveMode;
use notify::Watcher;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::sync::{atomic, mpsc, Arc};
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crate::error::HotReloaderError;

pub struct LibReloader {
    load_counter: usize,
    lib_dir: PathBuf,
    lib_name: String,
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
    ) -> Result<Self, HotReloaderError> {
        // find the target dir in which the build is happening and where we should find
        // the library
        let lib_dir = find_file_or_dir_in_parent_directories(lib_dir.as_ref())?;
        log::debug!("found lib dir at {lib_dir:?}");

        let load_counter = 0;

        let (watched_lib_file, loaded_lib_file) =
            watched_and_loaded_library_paths(&lib_dir, &lib_name, load_counter);

        let lib = if watched_lib_file.exists() {
            // We don't load the actual lib because this can get problems e.g. on Windows
            // where a file lock would be held, preventing the lib from changing later.
            fs::copy(&watched_lib_file, &loaded_lib_file)?;
            Some(unsafe { Library::new(&loaded_lib_file) }?)
        } else {
            log::debug!("library {watched_lib_file:?} does not yet exist");
            None
        };

        let lib_loader = Self {
            load_counter,
            lib_dir,
            lib_name: lib_name.as_ref().to_string(),
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
    pub fn update(&mut self) -> Result<bool, HotReloaderError> {
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
    fn reload(&mut self) -> Result<(), HotReloaderError> {
        let Self {
            load_counter,
            lib_dir,
            lib_name,
            watched_lib_file,
            loaded_lib_file,
            lib,
            ..
        } = self;

        log::info!("reloading lib {watched_lib_file:?}");

        // Close the loaded lib, copy the new lib to a file we can load, then load it.
        if let Some(lib) = lib.take() {
            lib.close()?;
            if loaded_lib_file.exists() {
                let _ = fs::remove_file(&loaded_lib_file);
            }
        }

        if watched_lib_file.exists() {
            *load_counter += 1;
            let (_, loaded_lib_file) =
                watched_and_loaded_library_paths(lib_dir, lib_name, *load_counter);
            log::debug!("copy {watched_lib_file:?} -> {loaded_lib_file:?}");
            fs::copy(watched_lib_file, &loaded_lib_file)?;
            self.lib = Some(unsafe { Library::new(&loaded_lib_file) }?);
            self.loaded_lib_file = loaded_lib_file;
        } else {
            log::warn!("trying to reload library but it does not exist");
        }

        Ok(())
    }

    /// Watch for changes of `lib_file`.
    fn watch(&self, lib_file: impl AsRef<Path>) -> Result<(), HotReloaderError> {
        let lib_file = lib_file.as_ref().to_path_buf();
        log::info!("start watching changes of file {}", lib_file.display());

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

                log::debug!("{} changed", lib_file.display());

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
                                "{} was removed, trying to watch it again...",
                                lib_file.display()
                            );
                        }
                        loop {
                            if watcher
                                .watch(&lib_file, RecursiveMode::NonRecursive)
                                .is_ok()
                            {
                                log::info!("watching {}", lib_file.display());
                                signal_change();
                                break;
                            }
                            thread::sleep(debounce);
                        }
                    }
                    Ok(change) => {
                        log::trace!("file change event: {change:?}");
                    }
                    Err(err) => {
                        log::error!("file watcher error, stopping reload loop: {err}");
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
    pub unsafe fn get_symbol<T>(&self, name: &[u8]) -> Result<Symbol<T>, HotReloaderError> {
        match &self.lib {
            None => Err(HotReloaderError::LibraryNotLoaded),
            Some(lib) => Ok(lib.get(name)?),
        }
    }
}

impl Drop for LibReloader {
    fn drop(&mut self) {
        if self.loaded_lib_file.exists() {
            let _ = fs::remove_file(&self.loaded_lib_file);
        }
    }
}

fn watched_and_loaded_library_paths(
    lib_dir: impl AsRef<Path>,
    lib_name: impl AsRef<str>,
    load_counter: usize,
) -> (PathBuf, PathBuf) {
    let lib_dir = &lib_dir.as_ref();

    // sort out os dependent file name
    #[cfg(target_os = "macos")]
    let (prefix, ext) = ("lib", "dylib");
    #[cfg(target_os = "linux")]
    let (prefix, ext) = ("lib", "so");
    #[cfg(target_os = "windows")]
    let (prefix, ext) = ("", "dll");
    let lib_name = format!("{prefix}{}", lib_name.as_ref());

    let watched_lib_file = lib_dir.join(&lib_name).with_extension(ext);
    let loaded_lib_file = lib_dir
        .join(format!("{lib_name}-hot-{load_counter}"))
        .with_extension(ext);
    (watched_lib_file, loaded_lib_file)
}

/// Try to find that might be a relative path such as `target/debug/` by walking
/// up the directories, starting from cwd. This helps finding the lib when the
/// app was started from a directory that is not the project/workspace root.
fn find_file_or_dir_in_parent_directories(
    file: impl AsRef<Path>,
) -> Result<PathBuf, HotReloaderError> {
    let mut file = file.as_ref().to_path_buf();
    if !file.exists() && file.is_relative() {
        if let Ok(cwd) = std::env::current_dir() {
            let mut parent_dir = Some(cwd.as_path());
            while let Some(dir) = parent_dir {
                if dir.join(&file).exists() {
                    file = dir.join(&file);
                    break;
                }
                parent_dir = dir.parent();
            }
        }
    }

    if file.exists() {
        Ok(file)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("file {file:?} does not exist"),
        )
        .into())
    }
}

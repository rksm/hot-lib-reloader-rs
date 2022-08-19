use libloading::Library;
use libloading::Symbol;
use notify::watcher;
use notify::DebouncedEvent;
use notify::RecursiveMode;
use notify::Watcher;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

use crate::error::HotReloaderError;

#[cfg(feature = "verbose")]
use log;

/// Manages watches a library (dylib) file, loads it using
/// [`libloading::Library`] and [provides access to its
/// symbols](LibReloader::get_symbol). When the library changes, [`LibReloader`]
/// is able to unload the old version and reload the new version through
/// [`LibReloader::update`].
///
/// Note that the [`LibReloader`] itself will not actively update, i.e. does not
/// manage an update thread calling the update function. This is normally
/// managed by the [`hot_lib_reloader_macro::hot_module`] macro that also
/// manages the [about-to-load and load](crate::LibReloadNotifier) notifications.
///
/// It can load symbols from the library with [LibReloader::get_symbol].
pub struct LibReloader {
    load_counter: usize,
    lib_dir: PathBuf,
    lib_name: String,
    changed: Arc<AtomicBool>,
    lib: Option<Library>,
    watched_lib_file: PathBuf,
    loaded_lib_file: PathBuf,
    lib_file_hash: Arc<AtomicU32>,
    file_change_subscribers: Arc<Mutex<Vec<mpsc::Sender<()>>>>,
    #[cfg(target_os = "macos")]
    codesigner: crate::codesign::CodeSigner,
}

impl LibReloader {
    /// Creates a LibReloader.
    ///  `lib_dir` is expected to be the location where the library to use can
    /// be found. Probably `target/debug` normally.
    /// `lib_name` is the name of the library, not(!) the file name. It should
    /// normally be just the crate name of the cargo project you want to hot-reload.
    /// LibReloader will take care to figure out the actual file name with
    /// platform-specific prefix and extension.
    pub fn new(
        lib_dir: impl AsRef<Path>,
        lib_name: impl AsRef<str>,
        file_watch_debounce: Option<Duration>,
    ) -> Result<Self, HotReloaderError> {
        // find the target dir in which the build is happening and where we should find
        // the library
        let lib_dir = find_file_or_dir_in_parent_directories(lib_dir.as_ref())?;
        log::debug!("found lib dir at {lib_dir:?}");

        let load_counter = 0;

        #[cfg(target_os = "macos")]
        let codesigner = crate::codesign::CodeSigner::new();

        let (watched_lib_file, loaded_lib_file) =
            watched_and_loaded_library_paths(&lib_dir, &lib_name, load_counter);

        let (lib_file_hash, lib) = if watched_lib_file.exists() {
            // We don't load the actual lib because this can get problems e.g. on Windows
            // where a file lock would be held, preventing the lib from changing later.
            log::debug!("copying {watched_lib_file:?} -> {loaded_lib_file:?}");
            fs::copy(&watched_lib_file, &loaded_lib_file)?;
            let hash = hash_file(&loaded_lib_file);
            #[cfg(target_os = "macos")]
            codesigner.codesign(&loaded_lib_file);
            (hash, Some(load_library(&loaded_lib_file)?))
        } else {
            log::debug!("library {watched_lib_file:?} does not yet exist");
            (0, None)
        };

        let lib_file_hash = Arc::new(AtomicU32::new(lib_file_hash));
        let changed = Arc::new(AtomicBool::new(false));
        let file_change_subscribers = Arc::new(Mutex::new(Vec::new()));
        Self::watch(
            watched_lib_file.clone(),
            lib_file_hash.clone(),
            changed.clone(),
            file_change_subscribers.clone(),
            file_watch_debounce.unwrap_or_else(|| Duration::from_millis(500)),
        )?;

        let lib_loader = Self {
            load_counter,
            lib_dir,
            lib_name: lib_name.as_ref().to_string(),
            watched_lib_file,
            loaded_lib_file,
            lib,
            lib_file_hash,
            changed,
            file_change_subscribers,
            #[cfg(target_os = "macos")]
            codesigner,
        };

        Ok(lib_loader)
    }

    // needs to be public as it is used inside the hot_module macro.
    #[doc(hidden)]
    pub fn subscribe_to_file_changes(&mut self) -> mpsc::Receiver<()> {
        log::trace!("subscribe to file change");
        let (tx, rx) = mpsc::channel();
        let mut subscribers = self.file_change_subscribers.lock().unwrap();
        subscribers.push(tx);
        rx
    }

    /// Checks if the watched library has changed. If it has, reload it and return
    /// true. Otherwise return false.
    pub fn update(&mut self) -> Result<bool, HotReloaderError> {
        if !self.changed.load(Ordering::Acquire) {
            return Ok(false);
        }
        self.changed.store(false, Ordering::Release);

        self.reload()?;

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
            log::trace!("copy {watched_lib_file:?} -> {loaded_lib_file:?}");
            fs::copy(watched_lib_file, &loaded_lib_file)?;
            self.lib_file_hash
                .store(hash_file(&loaded_lib_file), Ordering::Release);
            #[cfg(target_os = "macos")]
            self.codesigner.codesign(&loaded_lib_file);
            self.lib = Some(load_library(&loaded_lib_file)?);
            self.loaded_lib_file = loaded_lib_file;
        } else {
            log::warn!("trying to reload library but it does not exist");
        }

        Ok(())
    }

    /// Watch for changes of `lib_file`.
    fn watch(
        lib_file: impl AsRef<Path>,
        lib_file_hash: Arc<AtomicU32>,
        changed: Arc<AtomicBool>,
        file_change_subscribers: Arc<Mutex<Vec<mpsc::Sender<()>>>>,
        debounce: Duration,
    ) -> Result<(), HotReloaderError> {
        let lib_file = lib_file.as_ref().to_path_buf();
        log::info!("start watching changes of file {}", lib_file.display());

        // File watcher thread. We watch `self.lib_file`, when it changes and we haven't
        // a pending change still waiting to be loaded, set `self.changed` to true. This
        // then gets picked up by `self.update`.
        thread::spawn(move || {
            use DebouncedEvent::*;

            let (tx, rx) = mpsc::channel();
            let mut watcher = watcher(tx, debounce).unwrap();
            watcher
                .watch(&lib_file, RecursiveMode::NonRecursive)
                .expect("watch lib file");

            let signal_change = || {
                if hash_file(&lib_file) == lib_file_hash.load(Ordering::Acquire)
                    || changed.load(Ordering::Acquire)
                {
                    // file not changed
                    return false;
                }

                log::debug!("{lib_file:?} changed",);

                changed.store(true, Ordering::Release);

                // inform subscribers
                let subscribers = file_change_subscribers.lock().unwrap();
                log::trace!(
                    "sending ChangedEvent::LibFileChanged to {} subscribers",
                    subscribers.len()
                );
                for tx in &*subscribers {
                    let _ = tx.send(());
                }

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
                                log::info!("watching {lib_file:?} again after removal");
                                signal_change();
                                break;
                            }
                            thread::sleep(Duration::from_millis(500));
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

    /// Helper to log from the macro without requiring the user to have the log
    /// crate around
    #[doc(hidden)]
    pub fn log_info(what: impl std::fmt::Display) {
        log::info!("{}", what);
    }
}

/// Deletes the currently loaded lib file if it exists
impl Drop for LibReloader {
    fn drop(&mut self) {
        if self.loaded_lib_file.exists() {
            log::trace!("removing {:?}", self.loaded_lib_file);
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

fn load_library(lib_file: impl AsRef<Path>) -> Result<Library, HotReloaderError> {
    Ok(unsafe { Library::new(lib_file.as_ref()) }?)
}

fn hash_file(f: impl AsRef<Path>) -> u32 {
    fs::read(f.as_ref())
        .map(|content| crc32fast::hash(&content))
        .unwrap_or_default()
}

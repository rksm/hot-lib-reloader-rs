use libloading::Library;
use libloading::Symbol;
use notify::Watcher;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{atomic, mpsc, Arc};

pub struct LibReloader {
    changed: Arc<atomic::AtomicBool>,
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
            reload_counter: 0,
        };

        lib_loader.watch(lib_file)?;

        Ok(lib_loader)
    }

    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        if self.changed.load(Ordering::Relaxed) {
            self.changed.store(false, Ordering::Relaxed);
            self.reload()?;
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
            "reloading lib {} from file {new_lib_file:?}",
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
        println!("start watching changes of file {}", lib_file.display());

        let changed = self.changed.clone();
        std::thread::spawn(move || {
            let (tx, rx) = mpsc::channel();
            let debounce = std::time::Duration::from_millis(300);
            let mut watcher = notify::watcher(tx, debounce).unwrap();

            loop {
                watcher
                    .watch(&lib_file, notify::RecursiveMode::NonRecursive)
                    .unwrap();

                match rx.recv() {
                    Ok(_changed_event) => {
                        // on macos we can run into endless change loops unless
                        // we debounce after creation as well
                        watcher.unwatch(&lib_file).unwrap();
                        std::thread::sleep(debounce);
                        watcher
                            .watch(&lib_file, notify::RecursiveMode::NonRecursive)
                            .unwrap();
                        changed.store(true, Ordering::Relaxed);
                    }

                    Err(err) => {
                        eprintln!("file watcher error, stopping reload loop: {err}");
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

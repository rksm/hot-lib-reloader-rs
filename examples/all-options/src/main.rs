// dylib: the platform independent library name, typically the crate name
// lib_dir: where to find the library file. Defaults to "target/debug" and "target/release" for debug / release builds
// file_watch_debounce: Debounce duration in milliseconds for the file watcher checking for library changes 500ms is the default.
#[hot_lib_reloader::hot_module(
    dylib = "lib",
    lib_dir = if cfg!(debug_assertions) { "target/debug" } else { "target/release" },
    file_watch_debounce = 500,
    loaded_lib_name_template = "{lib_name}_hot_{pid}_{load_counter}"
)]
mod hot_lib {
    pub use lib::*;

    // embeds hot reloadable proxy functions for all public functions, even
    // those that are not #[unsafe(no_mangle)] in that rust source file
    hot_functions_from_file!("lib/src/lib.rs", ignore_no_mangle = true);

    // manually expose functions. Note there actually isn't such a function in lib.
    #[hot_functions]
    extern "Rust" {
        pub fn do_stuff2(arg: &str) -> u32;
    }

    // allows you to wait for about-to-reload and reloaded events
    #[lib_change_subscription]
    pub fn subscribe() -> hot_lib_reloader::LibReloadObserver {}

    // a monotonically increasing counter (starting with 0) that counts library reloads
    #[lib_version]
    pub fn version() -> usize {}

    // Expose a query function to test if the lib was reloaded. Note that this
    // function will return true only _once_ after a reload.
    #[lib_updated]
    pub fn was_updated() -> bool {}
}

fn main() {
    let mut state = hot_lib::State { counter: 0 };
    loop {
        hot_lib::do_stuff(&mut state);

        let update_blocker = hot_lib::subscribe().wait_for_about_to_reload();
        println!("about to reload...");
        std::thread::sleep(std::time::Duration::from_secs(1));
        drop(update_blocker);
        println!("read for reload...");

        hot_lib::subscribe().wait_for_reload();
        println!("reloaded at version {} now", hot_lib::version());

        assert!(hot_lib::was_updated());
        assert!(!hot_lib::was_updated());
    }
}

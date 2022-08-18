#[hot_lib_reloader::hot_module(dylib = "lib")]
mod hot_lib {
    pub use lib::State;
    hot_functions_from_file!("lib/src/lib.rs");

    #[lib_version]
    pub fn version() -> usize {}

    #[lib_change_subscription]
    pub fn subscribe() -> hot_lib_reloader::LibReloadObserver {}
}

fn main() {
    let mut state = hot_lib::State::default();

    loop {
        state = hot_lib::step(state);
        hot_lib::subscribe().wait_for_reload();
        state.version = hot_lib::version();
    }
}

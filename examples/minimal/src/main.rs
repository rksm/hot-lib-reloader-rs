#[hot_lib_reloader::hot_module(dylib = "lib")]
mod hot_lib {
    pub use lib::State;
    hot_functions_from_file!("lib/src/lib.rs");
}

fn main() {
    let mut state = hot_lib::State { counter: 0 };
    loop {
        hot_lib::do_stuff(&mut state);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

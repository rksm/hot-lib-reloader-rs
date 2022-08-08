#[hot_lib_reloader::hot_module(dylib = "lib")]
mod hot_lib {
    hot_functions_from_file!("../lib/src/lib.rs");
}

fn main() {
    loop {
        hot_lib::do_stuff();
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

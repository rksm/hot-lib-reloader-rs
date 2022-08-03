hot_lib_reloader::define_lib_reloader! {
    unsafe MyLibLoader {
        lib_name: "lib",
        source_files: ["../../lib/src/lib.rs"]
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut lib = MyLibLoader::new().expect("init lib loader");

    loop {
        lib.update().expect("lib update");

        lib.do_stuff();

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

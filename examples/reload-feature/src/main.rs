use lib::*;

#[cfg(feature = "reload")]
hot_lib_reloader::define_lib_reloader! {
    unsafe MyLibLoader {
        lib_name: "lib",
        source_files: ["../lib/src/lib.rs"]
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let mut state = lib::State::default();

    #[cfg(feature = "reload")]
    let mut lib = MyLibLoader::new().expect("init lib loader");

    loop {
        #[cfg(feature = "reload")]
        {
            lib.update().expect("lib update");
            lib.step(&mut state);
        }

        #[cfg(not(feature = "reload"))]
        lib::step(&mut state);

        dbg!(&state);

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

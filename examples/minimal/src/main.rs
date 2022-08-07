hot_lib_reloader::hot_module! {
    unsafe mod lib_hot {
        pub use lib::*;
    }
    lib_name = "lib";
    source_files = ["../lib/src/lib.rs"];
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut state = lib::State::default();

    loop {
        lib_hot::do_stuff("testing");
        lib_hot::do_stuff2("testing", &mut state);
        dbg!(&state);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

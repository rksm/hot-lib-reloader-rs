hot_lib_reloader::define_lib_reloader!(unsafe MyLibLoader {
    lib_name: "lib",
    functions: {
        fn test<'a>(arg1: &'a str, arg2: u8) -> String;
    },
    source_files: ["./input-files/lib.rs"],
});

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut loader = MyLibLoader::new().unwrap();
    loader.update().unwrap();
}

use proc_macro_test::define_lib_reloader;

struct LibReloader {}

impl LibReloader {
    fn new(
        _lib_dir: impl ToString,
        _lib_name: impl ToString,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {})
    }

    pub fn update(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(true)
    }

    pub unsafe fn get_symbol<T>(&self, _name: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        todo!()
    }
}

define_lib_reloader!(MyLibLoader {
    lib_dir: "target/debug",
    lib_name: "lib",
    functions: {
        fn test<'a>(arg1: &'a str, arg2: u8) -> String;
    },
    source_files: ["examples/input-files/lib.rs"],
});

fn main() {
    let mut loader = MyLibLoader::new().unwrap();
    loader.update().unwrap();
}

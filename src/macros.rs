/// Convenience macro. Usage like:
///
/// Example
/// ```no_run
/// hot_lib_reloader::define_lib_reloader!(
///     MyLibLoader("target/debug", "lib") {
///         fn do_stuff() -> ();
///     }
/// );
/// # fn main() {
/// let mut lib = MyLibLoader::new().expect("init lib loader");
/// lib.update().expect("lib update");
/// lib.do_stuff();
/// # }
#[macro_export]
macro_rules! define_lib_reloader {
    (unsafe $name:ident ( $dir:literal, $lib_name:literal ) { $(fn $rest:tt $args:tt -> $ret:ty;)* }) => {
        pub struct $name {
            lib_loader: $crate::LibReloader,
        }
        impl $name {
            pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
                Ok(Self {
                    lib_loader: $crate::LibReloader::new($dir, $lib_name)?,
                })
            }

            /// Checks if the watched library has changed. If it has, reload it and return
            /// true. Otherwise return false.
            pub fn update(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
                self.lib_loader.update()
            }

            $crate::__lib_loader_functions! { $(fn $rest $args -> $ret;)* }
        }
    };
}

#[macro_export]
macro_rules! __lib_loader_functions {
    () => {
    };
    ( fn $fun_name:ident $args:tt -> $ret:ty; $(fn $rest:ident $args2:tt -> $ret2:ty;)*) => {
        $crate::__lib_loader_function! { fn $fun_name $args -> $ret}
        $crate::__lib_loader_functions! { $(fn $rest $args2 -> $ret2;)* }
    };
}

#[macro_export]
macro_rules! __lib_loader_function {
    (fn $fun_name:ident ( $($name:ident : $type:ty),* )) => {
        $crate::__lib_loader_function!(fn $fun_name( $($name : $type),* ) -> ());
    };

    (fn $fun_name:ident ( $($name:ident : $type:ty),* ) -> $ret_type:ty) => {
        pub fn $fun_name(&self, $($name : $type),* ) -> $ret_type {
            unsafe {
                let f = self.lib_loader
                    .get_symbol::<fn($($type),*) -> $ret_type>(concat!(stringify!($fun_name), "\0").as_bytes())
                    .expect(concat!("Cannot load library function", stringify!($fun_name)));
                f($($name),*)
            }
        }
    };
}

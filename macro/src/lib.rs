mod hot_module;
mod lib_reloader;
mod util;

/// This is the deprecated way of defining a type for calling into reloadable code.
///
/// Parses a construct like
///
/// ```ignore
/// unsafe MyLibLoader {
///     lib_dir: "target/debug",
///     lib_name: "lib",
///     functions: {
///         fn test<'a>(arg1: &'a str, arg2: u8) -> String;
///     },
///     source_files: ["path/to/lib.rs"],
///     generate_bevy_systems: true
/// }
/// ```
#[deprecated]
#[proc_macro]
pub fn define_lib_reloader(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as lib_reloader::LibReloaderDefinition);
    (quote::quote! { #input }).into()
}

/// This macro is the top-level interface for making a dynamic Rust library
/// hot-reloadable. The attribute macro will insert code into the module it
/// accompanies that will do several things:
///
/// 1. In the context of that module a global
/// [`hot_lib_reloader::LibReloader`](https://docs.rs/hot-lib-reloader/latest/hot_lib_reloader/struct.LibReloader.html)
/// instance is maintained that loads the library specified by the `dylib`
/// argument and provides access to its symbols.
///
/// 2. A thread is started that drives the `LibReloader`: It waits for library
/// file changes and then
/// [updates](https://docs.rs/hot-lib-reloader/latest/hot_lib_reloader/struct.LibReloader.html#method.update)
/// the library.
///
/// 3. Allows access to a
/// [`hot_lib_reloader::LibReloadNotifier`](https://docs.rs/hot-lib-reloader/latest/hot_lib_reloader/struct.LibReloadNotifier.html)
/// that can be used to get events about library changes. See the
/// `#[lib_change_subscription]` attribute below.
///
/// In addition, the module can contain normal items. You can define functions,
/// types etc normally and you can import and export from other modules. In
/// particular re-exporting all items from the target library can make it easy
/// to create a 1:1 replacement with static modules from that library.
///
/// A few pseudo-macros can appear in the modules context:
///
/// ```
/// // The `dylib` attribute should be the name of the library to hot-reload,
/// // typically the crate name.
/// #[hot_module(dylib = "lib")]
/// mod foo {
///
///   // reads `#[no_mangle]` public functions from `file.rs` and generates
///   // forwarding functions in the context of this module that have the exact
///   // same signatures. Those generated functions will automatically use the
///   // newest version of the library.
///   hot_functions_from_file!("path/to/file.rs");
///
///   // As an alternative to `hot_functions_from_file!` you can manually
///   // declare functions that the library should export and for which hot-reload
///   // implementations should be generated. It is more tedious but plays nicer
///   // with tools like rust-analalyzer and auto completion.
///   #[hot_function]
///   pub fn do_stuff(arg: &str) -> u32 { /*generated*/ }
///
///   // Same as `hot_function` but as a block, multiple declarations are allowed.
///   #[hot_functions]
///   extern "Rust" {
///       pub fn do_stuff(arg: &str) -> u32;
///   }
///
///   // To get access to a `LibReloadObserver` you can create an empty function
///   // with a `#[lib_change_subscription]` attribute.
///    #[lib_change_subscription]
///    pub fn subscribe() -> hot_lib_reloader::LibReloadObserver {}
/// }
/// ```
///
/// In case you get errors when using the macro or are generally curious, run
/// `cargo expand` to see the generated code.
#[proc_macro_attribute]
pub fn hot_module(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args = syn::parse_macro_input!(args as hot_module::HotModuleAttribute);
    let mut module = syn::parse_macro_input!(item as hot_module::HotModule);
    module.hot_module_args = Some(args);

    (quote::quote! { #module }).into()
}

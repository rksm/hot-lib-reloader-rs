#![feature(proc_macro_span)]

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

/// Defines a module that embeds a [hot_lib_reloader::LibReloader] as a global
/// and that generates hot-reloadable functions.
///
/// Parses something like
/// ```ignore
/// #[hot_module(name = "lib")]
/// mod foo {
///   /* ... */
///   hot_functions_from_file!("../lib/src/lib.rs");
///   /* ... */
///   #[hot_function]
///       pub fn do_stuff(arg: &str) -> u32 { /*generated*/ }
///   /* ... */
///   #[hot_functions]
///   extern "Rust" {
///       pub fn do_stuff(arg: &str) -> u32;
///   }
/// }
/// ```
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

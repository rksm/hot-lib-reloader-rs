#![feature(proc_macro_span)]

mod hot_module;
mod lib_reloader;
mod util;

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
#[proc_macro]
pub fn define_lib_reloader(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as lib_reloader::LibReloaderDefinition);
    (quote::quote! { #input }).into()
}

#[proc_macro_attribute]
pub fn hot_module(
    args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attributes = syn::parse_macro_input!(args as hot_module::HotModuleAttribute);
    let mut module = syn::parse_macro_input!(item as hot_module::HotModule);
    module.attributes = Some(attributes);

    (quote::quote! { #module }).into()
}

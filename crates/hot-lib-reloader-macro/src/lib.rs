mod generate_libreloader_struct;
mod parse_definition;
mod types;

use generate_libreloader_struct::generate_lib_reloader_struct;
use proc_macro::TokenStream;
use quote::{quote, TokenStreamExt};
use syn::{braced, parse, parse_macro_input, token, Error, Ident, Result};
use types::LibReloaderDefinition;

use crate::{
    parse_definition::parse_field,
    types::{Field, PendingLibReloaderDefinition},
};

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
/// }
/// ```
impl parse::Parse for LibReloaderDefinition {
    fn parse(stream: parse::ParseStream) -> Result<Self> {
        if stream.is_empty() {
            return Err(Error::new(stream.span(), "No input"));
        }

        let _ = stream.parse::<syn::token::Unsafe>()?;

        let mut pending = PendingLibReloaderDefinition {
            name: Some(stream.parse::<Ident>()?),
            ..Default::default()
        };

        let field_stream;
        braced!(field_stream in stream);

        while !field_stream.is_empty() {
            let field_name = field_stream.parse::<Ident>()?;
            let _ = field_stream.parse::<token::Colon>()?;

            if field_name == "lib_dir" {
                parse_field(Field::LibDir, &field_stream, &mut pending)?;
            } else if field_name == "lib_name" {
                parse_field(Field::LibName, &field_stream, &mut pending)?;
            } else if field_name == "functions" {
                parse_field(Field::Functions, &field_stream, &mut pending)?;
            } else if field_name == "source_files" {
                parse_field(Field::SourceFiles, &field_stream, &mut pending)?;
            } else {
                return Err(Error::new(field_stream.span(), "unknown field"));
            }
            let _ = field_stream.parse::<token::Comma>();
        }

        pending.try_conversion(field_stream.span())
    }
}

impl quote::ToTokens for LibReloaderDefinition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_def = generate_lib_reloader_struct(
            &self.name,
            &self.lib_dir,
            &self.lib_name,
            &self.lib_functions,
        );
        proc_macro2::TokenStream::append_all(tokens, struct_def);
    }
}

#[proc_macro]
pub fn define_lib_reloader(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LibReloaderDefinition);
    (quote! { #input }).into()
}

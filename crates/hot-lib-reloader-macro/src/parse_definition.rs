use proc_macro2::Span;
use std::path::PathBuf;
use syn::{
    braced, bracketed, parse::ParseBuffer, spanned::Spanned, Error, ForeignItemFn, LitStr, Result,
};

use crate::types::{Field, PendingLibReloaderDefinition};

#[inline]
pub(crate) fn parse_field(
    field: Field,
    stream: &ParseBuffer,
    def: &mut PendingLibReloaderDefinition,
) -> Result<()> {
    match field {
        Field::LibDir => {
            def.lib_dir = Some(stream.parse::<LitStr>()?);
        }
        Field::LibName => {
            def.lib_name = Some(stream.parse::<LitStr>()?);
        }
        Field::Functions => {
            let function_stream;
            braced!(function_stream in stream);
            while !function_stream.is_empty() {
                let func: ForeignItemFn = function_stream.parse()?;
                let span = func.span();
                def.lib_functions.push((func, span));
            }
        }
        Field::SourceFiles => {
            let file_name_stream;
            bracketed!(file_name_stream in stream);
            while !file_name_stream.is_empty() {
                let file_name = file_name_stream.parse()?;
                def.lib_functions
                    .extend(parse_functions_from_file(file_name)?);
            }
        }
    }

    Ok(())
}

/// Reads the contents of a Rust source file and finds the top-level functions that have
/// - visibility public
/// - #[no_mangle] attribute
/// It converts these functions into a [syn::ForeignItemFn] so that those can
/// serve as lib function declarations of the lib reloader.
fn parse_functions_from_file(file_name: LitStr) -> Result<Vec<(ForeignItemFn, Span)>> {
    let span = file_name.span();
    let path: PathBuf = file_name.value().into();
    let path = if path.is_relative() {
        let file_with_macro = proc_macro::Span::call_site().source_file();
        file_with_macro
            .path()
            .parent()
            .map(|dir| dir.join(&path))
            .unwrap_or(path)
    } else {
        path
    };

    if !path.exists() {
        return Err(Error::new(span, format!("file does not exist: {path:?}")));
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|err| Error::new(span, format!("Error reading file {path:?}: {err}")))?;

    let ast = syn::parse_file(&content)?;

    let mut functions = Vec::new();

    for item in ast.items {
        match item {
            syn::Item::Fn(fun) => {
                match fun.vis {
                    syn::Visibility::Public(_) => {}
                    _ => continue,
                };

                let no_mangle = fun
                    .attrs
                    .iter()
                    .filter_map(|attr| attr.path.get_ident())
                    .any(|ident| *ident == "no_mangle");

                if !no_mangle {
                    continue;
                };

                let fun = ForeignItemFn {
                    attrs: Vec::new(),
                    vis: fun.vis,
                    sig: fun.sig,
                    semi_token: syn::token::Semi(span),
                };

                functions.push((fun, file_name.span()));
            }
            _ => continue,
        }
    }

    Ok(functions)
}

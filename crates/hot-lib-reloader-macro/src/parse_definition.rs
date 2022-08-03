use std::path::PathBuf;

use syn::{braced, bracketed, parse::ParseBuffer, Error, ForeignItemFn, LitStr, Result};

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
                def.lib_functions.push(function_stream.parse()?);
            }
        }
        Field::SourceFiles => {
            // let mut files = Vec::new();
            let file_name_stream;
            bracketed!(file_name_stream in stream);
            while !file_name_stream.is_empty() {
                // files.push(file_name_stream.parse::<LitStr>()?)
                let fun_declarations = parse_functions_from_file(file_name_stream.parse()?)?;
                def.lib_functions.extend(fun_declarations);
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
fn parse_functions_from_file(file_name: LitStr) -> Result<Vec<ForeignItemFn>> {
    let span = file_name.span();
    let path: PathBuf = file_name.value().into();

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

                // functions.push(fun);

                let fun = ForeignItemFn {
                    attrs: Vec::new(),
                    vis: fun.vis,
                    sig: fun.sig,
                    semi_token: syn::token::Semi(span),
                };

                functions.push(fun);
            }
            _ => continue,
        }
    }

    // println!("{:?}", fun.block);

    // let mut sig = fun.sig.clone();

    // //println!("found public fun {:?}", sig.to_token_stream().to_string());
    // //dbg!(&sig.inputs[0]);
    // //let arg: syn::FnArg = syn::parse_str("lib: Res<LibReloader>").unwrap();
    // let arg: syn::FnArg = syn::parse_quote! { lib: Res<LibReloader> };
    // // let block: syn::Block = syn::parse_quote!()
    // // dbg!(arg);
    // //syn::FnArg::Typed()
    // sig.inputs.insert(0, arg);

    // let result = quote::quote! {
    //     #sig
    // };
    // println!("{}", result);

    Ok(functions)
}

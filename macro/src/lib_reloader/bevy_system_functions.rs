use proc_macro2::{Span, TokenStream};
use syn::{ForeignItemFn, Ident, LitBool, Result};

use crate::util::ident_from_pat;

pub(crate) fn generate_bevy_system_functions(
    bevy_system_functions_flag: Option<LitBool>,
    lib_functions: &[(ForeignItemFn, Span)],
    loader_name: &Ident,
) -> Result<Option<TokenStream>> {
    if !bevy_system_functions_flag
        .map(|flag| flag.value)
        .unwrap_or(false)
    {
        return Ok(None);
    }

    let mut functions = Vec::new();

    for (func, span) in lib_functions {
        let mut sig = func.sig.clone();
        let func_ident = &sig.ident;

        let block = {
            let mut arg_names = Vec::new();
            for arg in &sig.inputs {
                if let syn::FnArg::Typed(typed) = arg {
                    arg_names.push(ident_from_pat(&typed.pat, func_ident, *span)?);
                }
            }

            syn::parse_quote_spanned! {*span=>
               {
                   loader.#func_ident( #( #arg_names ),* );
               }
            }
        };

        // inject loader arg
        let loader_arg: syn::FnArg = syn::parse_quote_spanned! {*span=>
           loader: Res<#loader_name>
        };
        sig.inputs.insert(0, loader_arg);

        let bevy_system_func = syn::ItemFn {
            attrs: Vec::new(),
            vis: syn::Visibility::Public(syn::token::Pub(*span)),
            sig,
            block,
        };

        functions.push(bevy_system_func);
    }

    Ok(Some(quote::quote! {
        #(#functions)*
    }))
}

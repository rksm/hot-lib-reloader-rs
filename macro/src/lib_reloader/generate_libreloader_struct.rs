use proc_macro2::{Span, TokenStream};
use syn::{
    parse_quote, token, FnArg, ForeignItemFn, Ident, ImplItemFn, LitByteStr, LitStr, Receiver,
    Result, Type, Visibility,
};

use crate::util::ident_from_pat;

pub fn generate_lib_reloader_struct(
    name: Ident,
    lib_dir: LitStr,
    lib_name: LitStr,
    lib_functions: Vec<(ForeignItemFn, Span)>,
) -> Result<TokenStream> {
    let mut lib_function_methods = Vec::with_capacity(lib_functions.len());
    for func in lib_functions {
        lib_function_methods.push(generate_impl_method_to_call_lib_function(func)?);
    }

    Ok(quote::quote! {
        pub struct #name {
            lib_loader: ::hot_lib_reloader::LibReloader,
        }

        impl #name {
            pub fn new() -> Result<Self, ::hot_lib_reloader::HotReloaderError> {
                Ok(Self {
                    lib_loader: ::hot_lib_reloader::LibReloader::new(#lib_dir, #lib_name, None)?,
                })
            }

            /// Checks if the watched library has changed. If it has, reload it and return
            /// true. Otherwise return false.
            pub fn update(&mut self) -> Result<bool, ::hot_lib_reloader::HotReloaderError> {
                self.lib_loader.update()
            }

            #( #lib_function_methods )*
        }

    })
}

/// This does two things with the lib_function:
///
/// 1. It extracts its name, args, and return type and uses that to create a method
/// body for the LibReloader struct that calls the library function.
///
/// 2. It generates a function signature that can be used as signature of a
/// method for the specific LibReloader struct.
///
/// Those two things are then put together to create a [syn::ImplItemFn].
fn generate_impl_method_to_call_lib_function(
    (lib_function, span): (ForeignItemFn, Span),
) -> Result<ImplItemFn> {
    let ForeignItemFn { attrs, sig, .. } = lib_function;

    // the symbol inside the library we call needs to be a byte string
    // ending with a nul byte.
    let symbol_name = {
        let mut symbol_name = sig.ident.to_string().into_bytes();
        symbol_name.push(b'\0');
        LitByteStr::new(&symbol_name, Span::call_site())
    };

    let ret_type = &sig.output;

    let err_msg = LitStr::new(
        &format!("Cannot load library function {}", sig.ident),
        Span::call_site(),
    );

    let mut input_types = Vec::new();
    let mut input_names = Vec::new();

    for arg in &sig.inputs {
        match arg {
            FnArg::Receiver(_) => {
                eprintln!("Warning: exported library name has receiver / self type");
                continue;
            }
            FnArg::Typed(typed) => {
                input_types.push(typed.ty.clone());
                input_names.push(ident_from_pat(&typed.pat, &sig.ident, span)?);
            }
        }
    }

    let block = parse_quote! {
        {
            unsafe {
                let f = self.lib_loader
                    .get_symbol::<fn( #( #input_types ),* ) #ret_type >(#symbol_name)
                    .expect(#err_msg);
                f(#( #input_names),* )
            }
        }
    };

    let mut sig = sig.clone();
    sig.inputs.insert(
        0,
        FnArg::Receiver(Receiver {
            attrs: Vec::new(),
            mutability: None,
            self_token: token::SelfValue(Span::call_site()),
            colon_token: None,
            reference: Some((token::And(Span::call_site()), None)),
            ty: Box::new(Type::Verbatim(TokenStream::new())),
        }),
    );

    Ok(ImplItemFn {
        attrs,
        vis: Visibility::Public(token::Pub(Span::call_site())),

        defaultness: None,
        sig,
        block,
    })
}

#[cfg(test)]
mod tests {
    use syn::spanned::Spanned;

    use super::generate_impl_method_to_call_lib_function;

    #[test]
    fn test_generate_impl_method_to_call_lib_function() {
        let lib_function: syn::ForeignItemFn = syn::parse_quote! {
            fn xxx(state: &mut State) -> u8;
        };
        let span = lib_function.span();
        let method = generate_impl_method_to_call_lib_function((lib_function, span)).unwrap();
        let output = quote::quote! { #method }.to_string();
        let expected = r#"pub fn xxx (& self , state : & mut State) -> u8 { unsafe { let f = self . lib_loader . get_symbol :: < fn (& mut State) -> u8 > (b"xxx\0") . expect ("Cannot load library function xxx") ; f (state) } }"#;

        assert_eq!(output, expected);
    }
}

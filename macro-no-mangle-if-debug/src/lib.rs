//! Will add `#[no_mangle]` to the item it is applied but only in debug mode.
//!
//! This is useful for use with [hot-lib-reloader](https://crates.io/crates/hot-lib-reloader) to conditionally expose library functions to the lib reloader only in debug mode.
//! In release mode where a build is to be expected fully static, no additional penalty is paid.
//!
//! ```xxx
//! #[no_mangle_if_debug]
//! fn func() {}
//! ```
//!
//! will expand to
//!
//! ```xxx
//! #[cfg(debug_assertions)]
//! #[no_mangle]
//! fn func() {}
//!
//! #[cfg(not(debug_assertions))]
//! fn func() {}
//! ```

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, token};

/// See package doc.
#[proc_macro_attribute]
pub fn no_mangle_if_debug(
    _args: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut debug_item = parse_macro_input!(item as syn::ItemFn);
    let mut release_item = debug_item.clone();

    debug_item.attrs.push(create_attribute(
        "cfg",
        quote! { (debug_assertions) }.into_token_stream(),
    ));
    debug_item
        .attrs
        .push(create_attribute("no_mangle", Default::default()));

    release_item.attrs.push(create_attribute(
        "cfg",
        quote! { (not(debug_assertions)) }.into_token_stream(),
    ));

    (quote! {
        #debug_item
        #release_item
    })
    .into()
}

fn create_attribute(ident: &str, tokens: TokenStream) -> syn::Attribute {
    let span = proc_macro2::Span::call_site();
    syn::Attribute {
        style: syn::AttrStyle::Outer,
        pound_token: token::Pound { spans: [span] },
        bracket_token: token::Bracket::default(),
        path: syn::Path {
            leading_colon: None,
            segments: [syn::PathSegment {
                ident: syn::Ident::new(ident, span),
                arguments: syn::PathArguments::None,
            }]
            .into_iter()
            .collect(),
        },
        tokens,
    }
}

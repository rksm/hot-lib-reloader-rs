use quote::ToTokens;
use syn::LitBool;
use syn::{
    spanned::Spanned, token, Attribute, ForeignItemFn, Ident, Item, ItemMacro, LitStr, Macro,
    Result, Visibility,
};

use super::code_gen::{
    gen_hot_module_function_for, gen_lib_change_subscription_function, generate_lib_loader_items,
};
use super::HotModuleAttribute;
use crate::hot_module::code_gen::{gen_lib_version_function, gen_lib_was_updated_function};
use crate::util::read_functions_from_file;

pub(crate) struct HotModule {
    pub(crate) vis: Visibility,
    pub(crate) ident: Ident,
    pub(crate) items: Vec<Item>,
    #[allow(dead_code)]
    pub(crate) attributes: Vec<Attribute>,
    pub(crate) hot_module_args: Option<super::HotModuleAttribute>,
}

impl syn::parse::Parse for HotModule {
    fn parse(stream: syn::parse::ParseStream) -> Result<Self> {
        let attributes = syn::Attribute::parse_outer(stream)?;

        let vis = stream
            .parse::<syn::Visibility>()
            .unwrap_or(Visibility::Inherited);

        stream.parse::<token::Mod>()?;

        let ident = stream.parse::<Ident>()?;

        let module_body_stream;
        syn::braced!(module_body_stream in stream);

        let mut items = Vec::new();

        while !module_body_stream.is_empty() {
            let item = module_body_stream.parse::<syn::Item>()?;

            match item {
                // parses the hot_functions_from_file!("path/to/file.rs") marker
                Item::Macro(ItemMacro {
                    mac: Macro { path, tokens, .. },
                    ..
                }) if path.is_ident("hot_functions_from_file") => {
                    let span = path.span();
                    let mut iter = tokens.into_iter();

                    // get filename
                    let file_name = iter
                        .next()
                        .ok_or_else(|| {
                            syn::Error::new(span, "expected path to file as a literal string")
                        })
                        .and_then(|t| syn::parse2::<LitStr>(t.into_token_stream()))?;

                    // parse optional `ignore_no_mangle = true`
                    let ignore_no_mangle = if let Some(tokens) = iter.next() {
                        match tokens {
                            proc_macro2::TokenTree::Punct(p) if p.as_char() == ',' => {
                                let ident = iter
                                    .next()
                                    .ok_or_else(|| syn::Error::new(ident.span(), "expected ident"))
                                    .and_then(|t| syn::parse2::<Ident>(t.to_token_stream()))?;
                                if ident != "ignore_no_mangle" {
                                    return Err(syn::Error::new(ident.span(), "unexpected input"));
                                }

                                iter.next()
                                    .ok_or_else(|| syn::Error::new(ident.span(), "expected ="))
                                    .and_then(|t| syn::parse2::<token::Eq>(t.to_token_stream()))?;

                                let val = iter
                                    .next()
                                    .ok_or_else(|| {
                                        syn::Error::new(ident.span(), "expected true or false")
                                    })
                                    .and_then(|t| syn::parse2::<LitBool>(t.to_token_stream()))?;
                                val.value()
                            }
                            other => {
                                return Err(syn::Error::new(other.span(), "expected comma"));
                            }
                        }
                    } else {
                        false
                    };

                    // read from file
                    let functions = read_functions_from_file(file_name, ignore_no_mangle)?;
                    for (f, span) in functions {
                        let f = gen_hot_module_function_for(f, span)?;
                        items.push(Item::Fn(f));
                    }
                }

                // parses and code gens
                // #[lib_change_subscription]
                // pub fn subscribe() -> ... {}
                syn::Item::Fn(func)
                    if func
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("lib_change_subscription")) =>
                {
                    let span = func.span();
                    let f = ForeignItemFn {
                        attrs: Vec::new(),
                        vis: func.vis,
                        sig: func.sig,
                        semi_token: token::Semi::default(),
                    };
                    let f = gen_lib_change_subscription_function(f, span)?;
                    items.push(Item::Fn(f));
                }

                // parses and code gens
                // #[lib_version]
                // pub fn version() -> usize {}
                syn::Item::Fn(func)
                    if func
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("lib_version")) =>
                {
                    let span = func.span();
                    let f = ForeignItemFn {
                        attrs: Vec::new(),
                        vis: func.vis,
                        sig: func.sig,
                        semi_token: token::Semi::default(),
                    };
                    let f = gen_lib_version_function(f, span)?;
                    items.push(Item::Fn(f));
                }

                // parses and code gens
                // #[lib_updated]
                // pub fn was_updated() -> bool {}
                syn::Item::Fn(func)
                    if func
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("lib_updated")) =>
                {
                    let span = func.span();
                    let f = ForeignItemFn {
                        attrs: Vec::new(),
                        vis: func.vis,
                        sig: func.sig,
                        semi_token: token::Semi::default(),
                    };
                    let f = gen_lib_was_updated_function(f, span)?;
                    items.push(Item::Fn(f));
                }

                // parses and code gens
                // #[hot_function]
                // fn do_stuff(arg: &str) -> u32 {}
                syn::Item::Fn(func)
                    if func
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("hot_function")) =>
                {
                    let span = func.span();
                    let f = ForeignItemFn {
                        attrs: Vec::new(),
                        vis: func.vis,
                        sig: func.sig,
                        semi_token: token::Semi::default(),
                    };
                    let f = gen_hot_module_function_for(f, span)?;
                    items.push(Item::Fn(f));
                }

                // parses and code gens
                // #[hot_functions]
                // extern "Rust" {
                //     pub fn do_stuff(arg: &str) -> u32;
                // }
                syn::Item::ForeignMod(foreign_mod)
                    if foreign_mod
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("hot_functions")) =>
                {
                    for item in foreign_mod.items {
                        match item {
                            syn::ForeignItem::Fn(f) => {
                                let span = f.span();
                                let f = gen_hot_module_function_for(f, span)?;
                                items.push(Item::Fn(f));
                            }
                            _ => {
                                eprintln!(
                                    "[warn] hot_functions extern block includes unexpected items"
                                );
                            }
                        }
                    }
                }

                // otherwise just use the item as is
                item => items.push(item),
            };
        }

        Ok(Self {
            ident,
            vis,
            items,
            attributes,
            hot_module_args: None,
        })
    }
}

impl quote::ToTokens for HotModule {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            vis,
            ident,
            items,
            hot_module_args,
            ..
        } = self;

        let HotModuleAttribute {
            lib_name,
            lib_dir,
            file_watch_debounce_ms,
            crate_name,
            loaded_lib_name_template,
        } = match hot_module_args {
            None => panic!("Expected to have macro attributes"),
            Some(attributes) => attributes,
        };

        let lib_loader = generate_lib_loader_items(
            lib_dir,
            lib_name,
            file_watch_debounce_ms,
            crate_name,
            loaded_lib_name_template,
            tokens.span(),
        )
        .expect("error generating hot lib loader helpers");

        let module_def = quote::quote! {
            #vis mod #ident {
                #( #items )*

                #lib_loader
            }
        };

        proc_macro2::TokenStream::extend(tokens, module_def);
    }
}

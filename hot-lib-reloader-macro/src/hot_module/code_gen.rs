use proc_macro2::{Span, TokenStream};
use syn::{
    token, AngleBracketedGenericArguments, Arm, Field, Fields, FieldsUnnamed, FnArg,
    GenericArgument, GenericParam, Generics, Ident, ItemFn, Lifetime, LifetimeDef, LitByteStr,
    LitStr, Result, Type, TypeTuple, Variant, VisPublic, Visibility,
};
use syn::{ForeignItemFn, ItemMod};

use crate::util::ident_from_pat;

pub(crate) fn generate_hot_lib_module(
    lib_dir: LitStr,
    lib_name: LitStr,
    module_def: ItemMod,
    lib_functions: Vec<(syn::ForeignItemFn, Span)>,
    span: Span,
) -> Result<TokenStream> {
    let ItemMod {
        vis,
        ident,
        content,
        ..
    } = module_def;

    let items = content.map(|(_, items)| items).unwrap_or_default();

    let lib_message_dispatcher =
        generate_global_lib_loader_conroller(lib_dir, lib_name, lib_functions, span)?;

    Ok(quote::quote_spanned! {span=>
        #vis mod #ident {
            #( #items );*

            #lib_message_dispatcher
        }
    })
}

fn generate_global_lib_loader_conroller(
    lib_dir: LitStr,
    lib_name: LitStr,
    lib_functions: Vec<(ForeignItemFn, Span)>,
    span: Span,
) -> Result<TokenStream> {
    let mut lifetime_counter = 0;
    let mut lifetime_generics = None;
    let mut forwarder_functions = Vec::with_capacity(lib_functions.len());
    let mut function_call_variants = Vec::with_capacity(lib_functions.len());
    let mut function_call_return_variants = Vec::with_capacity(lib_functions.len());
    let mut function_call_matches = Vec::with_capacity(lib_functions.len());

    for (i, (f, span)) in lib_functions.into_iter().enumerate() {
        let LibFunction {
            wrapper_function,
            function_call_variant,
            function_call_return_variant,
            function_call_match,
        } = LibFunction::for_lib_function(
            f,
            i,
            &mut lifetime_counter,
            &mut lifetime_generics,
            span,
        )?;
        forwarder_functions.push(wrapper_function);
        function_call_variants.push(function_call_variant);
        function_call_return_variants.push(function_call_return_variant);
        function_call_matches.push(function_call_match);
    }

    let lifetime_generics_static =
        lifetime_generics
            .as_ref()
            .map(|gen| AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: token::Lt { spans: [span] },
                gt_token: token::Gt { spans: [span] },
                args: (0..gen.params.len())
                    .into_iter()
                    .map(|_| {
                        GenericArgument::Lifetime(Lifetime {
                            apostrophe: span,
                            ident: Ident::new("static", span),
                        })
                    })
                    .collect(),
            });

    let result = quote::quote_spanned! {span=>


        #[allow(dead_code)]
        enum LibAccessRequest #lifetime_generics {
            CallFunction(FunctionCall #lifetime_generics, ::std::sync::mpsc::Sender<FunctionCallResult>),
        }

        #[allow(dead_code)]
        enum FunctionCall #lifetime_generics {
            #( #function_call_variants ),*
        }

        #[allow(dead_code)]
        enum FunctionCallResult {
            #( #function_call_return_variants ),*
        }

        struct LibAccess {
            _thread: Option<::std::thread::JoinHandle<()>>,
        }

        impl LibAccess {
            fn start(rx: ::std::sync::mpsc::Receiver<LibAccessRequest #lifetime_generics_static>) -> Self {
                let thread = ::std::thread::spawn(move || {
                    let mut lib_loader = ::hot_lib_reloader::LibReloader::new(#lib_dir, #lib_name).expect("failed to create hot reload loader");
                    const UPDATE_TIMEOUT: ::std::time::Duration = ::std::time::Duration::from_millis(600);

                    // ::log::info!("starting hot lib reloader thread");

                    loop {
                        lib_loader.update().expect("error updating hot lib loader");
                        match rx.recv_timeout(UPDATE_TIMEOUT) {
                            Ok(LibAccessRequest::CallFunction(fcall, result_tx)) => {
                                match fcall {
                                    #( #function_call_matches ),*
                                }
                            }

                            Err(::std::sync::mpsc::RecvTimeoutError::Timeout) => continue,

                            Err(::std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                        }
                    }

                    // ::log::warn!("hot lib reloader thread exiting");
                    eprintln!("hot lib reloader thread exiting");
                });

                LibAccess {
                    _thread: Some(thread),
                }
            }
        }

        static mut LIB_LOADER_CONTROL: Option<::std::sync::Mutex<LibAccess>> = None;
        static mut LIB_LOADER_SENDER: Option<::std::sync::Arc<::std::sync::mpsc::Sender<LibAccessRequest #lifetime_generics_static>>> = None;
        static LIB_LOADER_INIT: ::std::sync::Once = ::std::sync::Once::new();

        fn lib_access_request #lifetime_generics() -> ::std::sync::Arc<::std::sync::mpsc::Sender<LibAccessRequest #lifetime_generics>> {
            LIB_LOADER_INIT.call_once(|| {
                use ::std::borrow::BorrowMut;
                let (tx, rx) = ::std::sync::mpsc::channel();
                unsafe {
                    *LIB_LOADER_SENDER.borrow_mut() = Some(::std::sync::Arc::new(tx));
                    *LIB_LOADER_CONTROL.borrow_mut() =
                        Some(::std::sync::Mutex::new(LibAccess::start(rx)));
                }
            });
            let tx = unsafe { LIB_LOADER_SENDER.as_ref().cloned().unwrap() };
            // Adjust the lifetime. This is only safe because we guarantee that
            // all message processing is blocking from the senders point of
            // view. We will process the entire message before returning from
            // the calling method.
            unsafe { std::mem::transmute(tx) }
        }

        #( #forwarder_functions )*
    };

    Ok(result)
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

/// `for_func` is the exported function from the library. We create a wrapper
/// that calls it via message send using the lib message dispatcher.
struct LibFunction {
    wrapper_function: ItemFn,
    function_call_variant: Variant,
    function_call_return_variant: Variant,
    function_call_match: Arm,
}

impl LibFunction {
    fn for_lib_function(
        lib_function: ForeignItemFn,
        n: usize,
        lifetime_counter: &mut usize,
        all_lifetime_generics: &mut Option<Generics>,
        span: Span,
    ) -> Result<Self> {
        let ForeignItemFn { sig, .. } = lib_function;

        // the symbol inside the library we call needs to be a byte string
        // ending with a nul byte.
        let symbol_name = {
            let mut symbol_name = sig.ident.to_string().into_bytes();
            symbol_name.push(b'\0');
            LitByteStr::new(&symbol_name, Span::call_site())
        };

        // The name of the FunctionCall::_ and FunctionCallResult::_ variant for this
        // lib function
        let lib_function_variant_ident = Ident::new(&format!("CallFn{n}"), span);

        let ret_type_arrowed = &sig.output;
        let ret_type = match ret_type_arrowed {
            syn::ReturnType::Default => Box::new(Type::Tuple(TypeTuple {
                paren_token: token::Paren { span },
                elems: Default::default(),
            })),
            syn::ReturnType::Type(_, ty) => ty.clone(),
        };

        // let err_msg = LitStr::new(
        //     &format!("Cannot load library function {}", sig.ident),
        //     Span::call_site(),
        // );

        let mut input_types = Vec::new();
        let mut input_types_with_lifetimes = Vec::new();
        let mut input_types_without_lifetimes = Vec::new();
        let mut input_names = Vec::new();
        let mut lifetime_vars = Vec::new();

        for arg in &sig.inputs {
            match arg {
                FnArg::Receiver(_) => {
                    eprintln!("Warning: exported library name has receiver / self type");
                    continue;
                }
                FnArg::Typed(typed) => {
                    let ty = typed.ty.clone();
                    let mut ty_with_lifetime = ty.clone();
                    if let Type::Reference(reference) = &mut *ty_with_lifetime {
                        let lifetime_ident = Ident::new(&format!("t{}", *lifetime_counter), span);
                        let lifetime = Lifetime {
                            apostrophe: span,
                            ident: lifetime_ident,
                        };
                        reference.lifetime = Some(lifetime.clone());
                        lifetime_vars.push(Some(lifetime));
                        *lifetime_counter += 1;
                    } else {
                        lifetime_vars.push(None);
                    }

                    let mut ty_without_lifetime = ty.clone();
                    if let Type::Reference(reference) = &mut *ty_without_lifetime {
                        reference.lifetime = None;
                    }

                    input_types.push(ty);
                    input_types_with_lifetimes.push(ty_with_lifetime);
                    input_types_without_lifetimes.push(ty_without_lifetime);
                    input_names.push(ident_from_pat(&typed.pat, &sig.ident, span)?);
                }
            }
        }

        let lifetimes = lifetime_vars
            .into_iter()
            .flatten()
            .map(|ea| {
                GenericParam::Lifetime(LifetimeDef {
                    attrs: Vec::new(),
                    bounds: Default::default(),
                    colon_token: None,
                    lifetime: ea,
                })
            })
            .collect::<Vec<_>>();

        match all_lifetime_generics {
            Some(val) => {
                val.params.extend(lifetimes);
            }
            None => {
                let generics = Generics {
                    lt_token: Some(token::Lt { spans: [span] }),
                    gt_token: Some(token::Gt { spans: [span] }),
                    params: lifetimes.into_iter().collect(),
                    where_clause: None,
                };
                *all_lifetime_generics = Some(generics);
            }
        };

        // The FunctionCall variant
        let function_call_variant = Variant {
            attrs: Vec::new(),
            ident: lib_function_variant_ident.clone(),
            discriminant: None,
            fields: Fields::Unnamed(FieldsUnnamed {
                paren_token: token::Paren { span },
                unnamed: input_types_with_lifetimes
                    .clone()
                    .into_iter()
                    .map(|ty| Field {
                        attrs: Vec::new(),
                        vis: Visibility::Inherited,
                        ident: None,
                        colon_token: None,
                        ty: *ty,
                    })
                    .collect(),
            }),
        };

        // The FunctionCallResult variant
        let function_call_return_variant = Variant {
            attrs: Vec::new(),
            ident: lib_function_variant_ident.clone(),
            discriminant: None,
            fields: Fields::Unnamed(FieldsUnnamed {
                paren_token: token::Paren { span },
                unnamed: [Field {
                    attrs: Vec::new(),
                    vis: Visibility::Inherited,
                    ident: None,
                    colon_token: None,
                    ty: *ret_type,
                }]
                .into_iter()
                .collect(),
            }),
        };

        let err_msg = LitStr::new(
            &format!("Cannot load library function {}", sig.ident),
            Span::call_site(),
        );

        let function_call_match = Arm {
            attrs: Vec::new(),
            guard: None,
            fat_arrow_token: token::FatArrow {
                spans: [span, span],
            },
            comma: None,
            pat: syn::parse_quote! {
                FunctionCall::#lib_function_variant_ident(#(#input_names),*)
            },
            body: syn::parse_quote! {
                {
                    let f = unsafe {
                        lib_loader
                            .get_symbol::<fn( #( #input_types_without_lifetimes ),* ) #ret_type_arrowed >(#symbol_name)
                            .expect(#err_msg)
                    };
                    let result = f(#( #input_names),* );
                    let _ = result_tx.send(FunctionCallResult::#lib_function_variant_ident(result));
                }
            },
        };

        // The wrapping function we export in the hot module that invokes the lib
        // function using message sending
        let function_call_requester = ItemFn {
            attrs: Vec::new(),
            vis: Visibility::Public(VisPublic {
                pub_token: token::Pub(Span::call_site()),
            }),
            sig,
            block: syn::parse_quote! {
                {
                    let tx = lib_access_request();
                    let (result_tx, result_rx) = ::std::sync::mpsc::channel();
                    tx.send(LibAccessRequest::CallFunction(FunctionCall::#lib_function_variant_ident(#( #input_names ),* ), result_tx)).expect("failure requesting to call hot function");
                    #[allow(irrefutable_let_patterns)]
                    if let FunctionCallResult::#lib_function_variant_ident(result) =  result_rx.recv().expect("failure waiting for hot function call result") {
                        result
                    } else {
                        panic!("Wrong hot lib reload return type");
                    }
                }
            },
        };

        Ok(Self {
            wrapper_function: function_call_requester,
            function_call_variant,
            function_call_return_variant,
            function_call_match,
        })
    }
}

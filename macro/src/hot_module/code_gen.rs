use proc_macro2::Span;
use syn::ForeignItemFn;
use syn::{token, FnArg, ItemFn, LitByteStr, LitStr, Result, VisPublic, Visibility};

use crate::util::ident_from_pat;

pub(crate) fn generate_lib_loader_items(
    lib_dir: &LitStr,
    lib_name: &LitStr,
    span: Span,
) -> Result<proc_macro2::TokenStream> {
    let result = quote::quote_spanned! {span=>
        static mut LIB_CHANGE_NOTIFIER: Option<::std::sync::Arc<::std::sync::RwLock<::hot_lib_reloader::LibReloadNotifier>>> = None;
        static LIB_CHANGE_NOTIFIER_INIT: ::std::sync::Once = ::std::sync::Once::new();

        fn __lib_notifier() -> ::std::sync::Arc<::std::sync::RwLock<::hot_lib_reloader::LibReloadNotifier>> {
            LIB_CHANGE_NOTIFIER_INIT.call_once(|| {
                let notifier = ::std::sync::Arc::new(::std::sync::RwLock::new(Default::default()));
                // Safety: guarded by Once, will only be called one time.
                unsafe {
                    use ::std::borrow::BorrowMut;
                    *LIB_CHANGE_NOTIFIER.borrow_mut() = Some(notifier);
                }
            });

            // Safety: Once runs before and initializes the global
            unsafe { LIB_CHANGE_NOTIFIER.as_ref().cloned().unwrap() }
        }

        fn __lib_loader_subscription() -> ::hot_lib_reloader::LibReloadObserver {
            __lib_notifier()
                .write()
                .expect("write lock notifier")
                .subscribe()
        }

        static mut LIB_LOADER: Option<::std::sync::Arc<::std::sync::RwLock<::hot_lib_reloader::LibReloader>>> = None;
        static LIB_LOADER_INIT: ::std::sync::Once = ::std::sync::Once::new();

        fn __lib_loader() -> ::std::sync::Arc<::std::sync::RwLock<::hot_lib_reloader::LibReloader>> {
            LIB_LOADER_INIT.call_once(|| {
                let mut lib_loader = ::hot_lib_reloader::LibReloader::new(#lib_dir, #lib_name)
                    .expect("failed to create hot reload loader");

                let change_rx = lib_loader.subscribe_to_file_changes();
                let lib_loader = ::std::sync::Arc::new(::std::sync::RwLock::new(lib_loader));
                let lib_loader_for_update = lib_loader.clone();

                // update thread that triggers the dylib to be actually updated
                let _thread = ::std::thread::spawn(move || {
                    loop {
                        if let Ok(()) = change_rx.recv() {
                            // inform subscribers about about-to-reload
                            __lib_notifier()
                                .read()
                                .expect("read lock notifier")
                                .send_about_to_reload_event_and_wait_for_blocks();

                            // get lock to lib_loader, make sure to not deadlock on it here
                            loop {
                                if let Ok(mut lib_loader) = lib_loader_for_update.try_write() {
                                    let _ = !lib_loader.update().expect("hot lib update()");
                                    break;
                                }
                                println!("[hot-lib-reloader] trying to get a write lock");
                                ::std::thread::sleep(::std::time::Duration::from_millis(1));
                            }

                            // inform subscribers about lib reloaded
                            __lib_notifier()
                                .read()
                                .expect("read lock notifier")
                                .send_reloaded_event();
                        }
                    }
                });

                // Safety: guarded by Once, will only be called one time.
                unsafe {
                    use ::std::borrow::BorrowMut;
                    *LIB_LOADER.borrow_mut() = Some(lib_loader);
                }
            });

            // Safety: Once runs before and initializes the global
            unsafe { LIB_LOADER.as_ref().cloned().unwrap() }
        }

    };

    Ok(result)
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

pub(crate) fn gen_hot_module_function_for(
    lib_function: ForeignItemFn,
    span: Span,
) -> Result<ItemFn> {
    let ForeignItemFn { sig, .. } = lib_function;

    // the symbol inside the library we call needs to be a byte string
    // ending with a nul byte.
    let fun_ident = &sig.ident;

    let symbol_name = {
        let mut symbol_name = fun_ident.to_string().into_bytes();
        symbol_name.push(b'\0');
        LitByteStr::new(&symbol_name, Span::call_site())
    };

    let ret_type = &sig.output;

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

    let err_msg_load_symbol = LitStr::new(
        &format!("Cannot load library function {}", sig.ident),
        Span::call_site(),
    );

    let block = syn::parse_quote! {
        {
            let lib_loader = __lib_loader();
            let lib_loader = lib_loader.read().expect("lib loader RwLock read failed");
            let sym = unsafe {
                lib_loader
                    .get_symbol::<fn( #( #input_types ),* ) #ret_type >(#symbol_name)
                    .expect(#err_msg_load_symbol)
            };
            sym( #( #input_names ),* )
        }
    };

    // The wrapping function we export in the hot module that invokes the lib
    // function using message sending
    let function = ItemFn {
        attrs: Vec::new(),
        vis: Visibility::Public(VisPublic {
            pub_token: token::Pub(Span::call_site()),
        }),
        sig,
        block,
    };

    Ok(function)
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

/// For something like
/// ```ignore
/// #[lib_change_subscription]
/// pub fn rx() -> std::sync::mpsc::Receiver<hot_lib_reloader::ChangedEvent> {
///     __lib_loader_subscription()
/// }
/// ```
pub(crate) fn gen_lib_change_subscription_function(
    f_decl: ForeignItemFn,
    span: Span,
) -> Result<ItemFn> {
    let ForeignItemFn {
        sig, vis, attrs, ..
    } = f_decl;

    Ok(ItemFn {
        attrs,
        vis,
        sig,
        block: syn::parse_quote_spanned! {span=>
            {
                __lib_loader_subscription()
            }
        },
    })
}
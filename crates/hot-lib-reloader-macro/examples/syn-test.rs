use proc_macro2::Span;
use syn::{
    parse_quote,
    token::{self, Crate},
    FnArg, ForeignItemFn, ImplItemMethod, LitByteStr, LitStr, Receiver, VisCrate, Visibility,
};

/// This does two things with the lib_function:
///
/// 1. It extracts its name, args, and return type and uses that to create a method
/// body for the LibReloader struct that calls the library function.
///
/// 2. It generates a function signature that can be used as signature of a
/// method for the specific LibReloader struct.
///
/// Those two things are then put together to create a [syn::ImplItemMethod].
fn generate_impl_method_to_call_lib_function(lib_function: ForeignItemFn) -> ImplItemMethod {
    let ForeignItemFn { attrs, mut sig, .. } = lib_function;

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
                input_names.push(typed.pat.clone());
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

    sig.inputs.insert(
        0,
        FnArg::Receiver(Receiver {
            attrs: Vec::new(),
            mutability: None,
            self_token: token::SelfValue(Span::call_site()),
            reference: Some((token::And(Span::call_site()), None)),
        }),
    );

    ImplItemMethod {
        attrs,
        vis: Visibility::Crate(VisCrate {
            crate_token: Crate(Span::call_site()),
        }),
        defaultness: None,
        sig,
        block,
    }
}

fn main() {
    if true {
        let content = r#"
impl Foo {
  fn xxx(&self) {}
}
"#;
        let ast = syn::parse_file(content).unwrap();
        dbg!(ast);
    };

    if false {
        let lib_function: ForeignItemFn = parse_quote! {
            fn xxx(state: &mut State);
        };

        let method = generate_impl_method_to_call_lib_function(lib_function);

        let output = quote::quote! {
            #method
        };

        println!("{output}");
    };
}

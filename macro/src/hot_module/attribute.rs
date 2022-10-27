use syn::{
    punctuated::Punctuated, spanned::Spanned, token, Error, ExprAssign, Ident, LitInt, Result,
};

pub(crate) struct HotModuleAttribute {
    pub(crate) lib_name: syn::Expr,
    pub(crate) lib_dir: syn::Expr,
    pub(crate) file_watch_debounce_ms: syn::LitInt,
}

// Parses something like `#[hot(name = "lib")]`.
impl syn::parse::Parse for HotModuleAttribute {
    fn parse(stream: syn::parse::ParseStream) -> Result<Self> {
        let mut lib_name = None;
        let mut lib_dir = None;
        let mut file_watch_debounce_ms = None;

        let args = Punctuated::<syn::Expr, token::Comma>::parse_separated_nonempty(stream)?;

        fn expr_is_ident<I: ?Sized>(expr: &syn::Expr, ident: &I) -> bool
        where
            Ident: PartialEq<I>,
        {
            if let syn::Expr::Path(syn::ExprPath { path, .. }) = expr {
                path.is_ident(ident)
            } else {
                false
            }
        }

        for arg in args {
            match arg {
                syn::Expr::Assign(ExprAssign { left, right, .. }) => match *right {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(lit),
                        ..
                    }) if expr_is_ident(&left, "file_watch_debounce") => {
                        file_watch_debounce_ms = Some(lit.clone());
                        continue;
                    }

                    expr if expr_is_ident(&left, "dylib") => {
                        lib_name = Some(expr);
                        continue;
                    }

                    expr if expr_is_ident(&left, "lib_dir") => {
                        lib_dir = Some(expr);
                        continue;
                    }

                    _ => return Err(Error::new(left.span(), "unexpected attribute name")),
                },

                _ => return Err(Error::new(arg.span(), "unexpected input")),
            }
        }

        let lib_name = match lib_name {
            None => {
                return Err(Error::new(
                    stream.span(),
                    r#"missing field "name": add `name = "name_of_library""#,
                ))
            }
            Some(lib_name) => lib_name,
        };

        let lib_dir = match lib_dir {
            None => {
                if cfg!(debug_assertions) {
                    syn::parse_quote! { concat!(env!("CARGO_MANIFEST_DIR"), "/target/debug") }
                } else {
                    syn::parse_quote! { concat!(env!("CARGO_MANIFEST_DIR"), "/target/release") }
                }
            }
            Some(lib_dir) => lib_dir,
        };

        let file_watch_debounce_ms = match file_watch_debounce_ms {
            None => LitInt::new("500", stream.span()),
            Some(file_watch_debounce_ms) => file_watch_debounce_ms,
        };

        Ok(HotModuleAttribute {
            lib_name,
            lib_dir,
            file_watch_debounce_ms,
        })
    }
}

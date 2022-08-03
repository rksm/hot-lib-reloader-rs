pub fn ident_from_pat(
    pat: &syn::Pat,
    func_name: &proc_macro2::Ident,
    span: proc_macro2::Span,
) -> syn::Result<syn::Ident> {
    match pat {
        syn::Pat::Ident(pat) => Ok(pat.ident.clone()),
        _ => Err(syn::Error::new(
            span,
            format!("generating call for library function: signature of function {func_name} cannot be converted"),
        )),
    }
}

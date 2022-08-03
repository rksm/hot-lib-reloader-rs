use proc_macro2::Span;
use syn::Ident;
use syn::{ForeignItemFn, LitStr};

pub(crate) struct LibReloaderDefinition {
    pub(crate) struct_def: proc_macro2::TokenStream,
}

#[derive(Default)]
pub(crate) struct PendingLibReloaderDefinition {
    pub(crate) name: Option<Ident>,
    pub(crate) lib_dir: Option<LitStr>,
    pub(crate) lib_name: Option<LitStr>,
    pub(crate) lib_functions: Vec<(ForeignItemFn, Span)>,
}

pub(crate) enum Field {
    LibDir,
    LibName,
    Functions,
    SourceFiles,
}

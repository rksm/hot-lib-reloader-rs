use proc_macro2::Span;

pub(crate) struct HotModuleDefinition {
    pub(crate) module_def: proc_macro2::TokenStream,
    // pub(crate) struct_def: proc_macro2::TokenStream,
    // pub(crate) bevy_system_functions: Option<proc_macro2::TokenStream>,
}

pub(crate) struct PendingHotModuleDefinition {
    pub(crate) module_def: syn::ItemMod,
    pub(crate) lib_dir: Option<syn::LitStr>,
    pub(crate) lib_name: Option<syn::LitStr>,
    pub(crate) lib_functions: Vec<(syn::ForeignItemFn, Span)>,
    // pub(crate) generate_bevy_system_functions: Option<LitBool>,
}

pub(crate) enum Field {
    LibDir,
    LibName,
    Functions,
    SourceFiles,
    // GenerateBevySystemFunctions,
}

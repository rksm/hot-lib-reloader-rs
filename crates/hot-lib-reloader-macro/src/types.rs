use proc_macro2::Span;
use syn::Ident;
use syn::{Error, ForeignItemFn, LitStr, Result};

pub(crate) struct LibReloaderDefinition {
    pub(crate) name: Ident,
    pub(crate) lib_dir: LitStr,
    pub(crate) lib_name: LitStr,
    pub(crate) lib_functions: Vec<ForeignItemFn>,
}

#[derive(Default)]
pub(crate) struct PendingLibReloaderDefinition {
    pub(crate) name: Option<Ident>,
    pub(crate) lib_dir: Option<LitStr>,
    pub(crate) lib_name: Option<LitStr>,
    pub(crate) lib_functions: Vec<ForeignItemFn>,
}

pub(crate) enum Field {
    LibDir,
    LibName,
    Functions,
    SourceFiles,
}

impl PendingLibReloaderDefinition {
    pub(crate) fn try_conversion(self, span: Span) -> Result<LibReloaderDefinition> {
        let Self {
            name,
            lib_dir,
            lib_name,
            lib_functions,
        } = self;

        let name = match name {
            None => return Err(Error::new(span, "The name of the struct is missing")),
            Some(name) => name,
        };

        let lib_dir = match lib_dir {
            None => return Err(Error::new(span, "missing field \"lib_dir\"")),
            Some(lib_dir) => lib_dir,
        };

        let lib_name = match lib_name {
            None => return Err(Error::new(span, "missing field \"lib_name\"")),
            Some(lib_name) => lib_name,
        };

        Ok(LibReloaderDefinition {
            name,
            lib_dir,
            lib_name,
            lib_functions,
        })
    }
}

mod code_gen;
mod parse;
mod types;

use proc_macro2::Span;
use quote::TokenStreamExt;
use syn::{spanned::Spanned, token, Error, Ident, LitStr, Result};

pub(crate) use types::HotModuleDefinition;

use self::{
    code_gen::generate_hot_lib_module,
    parse::parse_field,
    types::{Field, PendingHotModuleDefinition},
};

impl syn::parse::Parse for HotModuleDefinition {
    fn parse(stream: syn::parse::ParseStream) -> Result<Self> {
        // if stream.is_empty() {
        //     return Err(Error::new(stream.span(), "No input"));
        // }

        let _ = stream.parse::<syn::token::Unsafe>()?;

        let module_def = match stream.parse::<syn::Item>()? {
            syn::Item::Mod(module_def) => module_def,
            _ => {
                return Err(Error::new(stream.span(), "Expected a module declaration"));
            }
        };

        let mut pending = PendingHotModuleDefinition {
            module_def,
            lib_dir: Default::default(),
            lib_name: Default::default(),
            lib_functions: Default::default(),
        };

        while !stream.is_empty() {
            let field_name = stream.parse::<Ident>()?;
            let _ = stream.parse::<token::Eq>()?;

            if field_name == "lib_dir" {
                parse_field(Field::LibDir, stream, &mut pending)?;
            } else if field_name == "lib_name" {
                parse_field(Field::LibName, stream, &mut pending)?;
            } else if field_name == "functions" {
                parse_field(Field::Functions, stream, &mut pending)?;
            // } else if field_name == "generate_bevy_systems" {
            //     parse_field(
            //         Field::GenerateBevySystemFunctions,
            //         stream,
            //         &mut pending,
            //     )?;
            } else if field_name == "source_files" {
                parse_field(Field::SourceFiles, stream, &mut pending)?;
            } else {
                return Err(Error::new(stream.span(), "unknown field"));
            }
            let _ = stream.parse::<token::Semi>();
        }

        pending.try_conversion(stream.span())
    }
}

impl quote::ToTokens for HotModuleDefinition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // TODO
        // We are generating the module_def already in `try_conversion` so that we can
        // emit errors that point to the offending part of the proc macro. This is
        // useful for debugging but a bit less efficient. The struct generation
        // shouldn't actually be able to error so when things are stable we should
        // consider to inline.
        proc_macro2::TokenStream::append_all(tokens, self.module_def.clone());
        // if let Some(bevy_systems) = &self.bevy_system_functions {
        //     proc_macro2::TokenStream::append_all(tokens, bevy_systems.clone());
        // }
    }
}

impl PendingHotModuleDefinition {
    pub(crate) fn try_conversion(self, span: Span) -> Result<HotModuleDefinition> {
        let Self {
            module_def,
            lib_dir,
            lib_name,
            lib_functions,
            // generate_bevy_system_functions: bevy_system_functions_flag,
        } = self;

        // let name = match name {
        //     None => return Err(Error::new(span, "The name of the struct is missing")),
        //     Some(name) => name,
        // };

        let lib_dir = match lib_dir {
            None => {
                if cfg!(debug_assertions) {
                    LitStr::new("target/debug", span)
                } else {
                    LitStr::new("target/release", span)
                }
            }
            Some(lib_dir) => lib_dir,
        };

        let lib_name = match lib_name {
            None => return Err(Error::new(span, "missing field \"lib_name\"")),
            Some(lib_name) => lib_name,
        };

        // let bevy_system_functions =
        //     generate_bevy_system_functions(bevy_system_functions_flag, &lib_functions, &name)?;
        // let struct_def = generate_lib_reloader_struct(name, lib_dir, lib_name, lib_functions)?;

        let module_span = module_def.span();
        let module_def =
            generate_hot_lib_module(lib_dir, lib_name, module_def, lib_functions, module_span)?;

        // Ok(LibReloaderDefinition {
        //     struct_def,
        //     bevy_system_functions,
        // })

        Ok(HotModuleDefinition { module_def })
    }
}

mod bevy_system_functions;
mod generate_libreloader_struct;
mod parse_definition;
mod types;

use proc_macro2::Span;
use quote::TokenStreamExt;
use syn::{braced, parse, token, Error, Ident, LitStr, Result};

use bevy_system_functions::generate_bevy_system_functions;
use generate_libreloader_struct::generate_lib_reloader_struct;
use parse_definition::parse_field;
use types::{Field, PendingLibReloaderDefinition};

pub(crate) use types::LibReloaderDefinition;

impl parse::Parse for LibReloaderDefinition {
    fn parse(stream: parse::ParseStream) -> Result<Self> {
        if stream.is_empty() {
            return Err(Error::new(stream.span(), "No input"));
        }

        let _ = stream.parse::<syn::token::Unsafe>()?;

        let mut pending = PendingLibReloaderDefinition {
            name: Some(stream.parse::<Ident>()?),
            ..Default::default()
        };

        let field_stream;
        braced!(field_stream in stream);

        while !field_stream.is_empty() {
            let field_name = field_stream.parse::<Ident>()?;
            let _ = field_stream.parse::<token::Colon>()?;

            if field_name == "lib_dir" {
                parse_field(Field::LibDir, &field_stream, &mut pending)?;
            } else if field_name == "lib_name" {
                parse_field(Field::LibName, &field_stream, &mut pending)?;
            } else if field_name == "functions" {
                parse_field(Field::Functions, &field_stream, &mut pending)?;
            } else if field_name == "generate_bevy_systems" {
                parse_field(
                    Field::GenerateBevySystemFunctions,
                    &field_stream,
                    &mut pending,
                )?;
            } else if field_name == "source_files" {
                parse_field(Field::SourceFiles, &field_stream, &mut pending)?;
            } else {
                return Err(Error::new(field_stream.span(), "unknown field"));
            }
            let _ = field_stream.parse::<token::Comma>();
        }

        pending.try_conversion(field_stream.span())
    }
}

impl quote::ToTokens for LibReloaderDefinition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // We are generating the struct_def already in `try_conversion` so that we can
        // emit errors that point to the offending part of the proc macro. This is
        // useful for debugging but a bit less efficient. The struct generation
        // shouldn't actually be able to error so when things are stable we should
        // consider to put `generate_lib_reloader_struct` here again.
        proc_macro2::TokenStream::append_all(tokens, self.struct_def.clone());
        if let Some(bevy_systems) = &self.bevy_system_functions {
            proc_macro2::TokenStream::append_all(tokens, bevy_systems.clone());
        }
    }
}

impl PendingLibReloaderDefinition {
    pub(crate) fn try_conversion(self, span: Span) -> Result<LibReloaderDefinition> {
        let Self {
            name,
            lib_dir,
            lib_name,
            lib_functions,
            generate_bevy_system_functions: bevy_system_functions_flag,
        } = self;

        let name = match name {
            None => return Err(Error::new(span, "The name of the struct is missing")),
            Some(name) => name,
        };

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

        let bevy_system_functions =
            generate_bevy_system_functions(bevy_system_functions_flag, &lib_functions, &name)?;
        let struct_def = generate_lib_reloader_struct(name, lib_dir, lib_name, lib_functions)?;

        Ok(LibReloaderDefinition {
            struct_def,
            bevy_system_functions,
        })
    }
}

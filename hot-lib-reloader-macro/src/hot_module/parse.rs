use syn::{braced, bracketed, parse::ParseBuffer, spanned::Spanned, ForeignItemFn, LitStr, Result};

use crate::util::read_unmangled_functions_from_file;

use super::types::{Field, PendingHotModuleDefinition};

pub(crate) fn parse_field(
    field: Field,
    stream: &ParseBuffer,
    def: &mut PendingHotModuleDefinition,
) -> Result<()> {
    match field {
        Field::LibDir => {
            def.lib_dir = Some(stream.parse::<LitStr>()?);
        }
        Field::LibName => {
            def.lib_name = Some(stream.parse::<LitStr>()?);
        }
        Field::Functions => {
            let function_stream;
            braced!(function_stream in stream);
            while !function_stream.is_empty() {
                let func: ForeignItemFn = function_stream.parse()?;
                let span = func.span();
                def.lib_functions.push((func, span));
            }
        }
        Field::SourceFiles => {
            let file_name_stream;
            bracketed!(file_name_stream in stream);
            while !file_name_stream.is_empty() {
                let file_name = file_name_stream.parse()?;
                def.lib_functions
                    .extend(read_unmangled_functions_from_file(file_name)?);
            }
        } // Field::GenerateBevySystemFunctions => {
          //     def.generate_bevy_system_functions = Some(stream.parse::<LitBool>()?);
          // }
    }

    Ok(())
}

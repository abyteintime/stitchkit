use std::num::{IntErrorKind, ParseIntError};

use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    source::SourceFileId,
};

use crate::diagnostics::notes;

use super::IntLit;

// NOTE: Currently int parsing is not ideal, because the corner case of -0x80000000 is not handled
// correctly, as the negative sign is not part of the integer literal.
impl IntLit {
    fn map_parse_error(
        &self,
        result: Result<i32, ParseIntError>,
        diagnostics: &mut dyn DiagnosticSink,
        source_file_id: SourceFileId,
    ) -> i32 {
        match result {
            Ok(num) => num,
            Err(error) => match error.kind() {
                IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                    diagnostics.emit(
                        Diagnostic::error(source_file_id, "integer does not fit within 32 bits")
                            .with_label(Label::primary(self.span, ""))
                            .with_note(indoc::indoc! {"
                                note: UnrealScript integers are 32 bit;
                                      this means their values are in the range [-2147483648, 2147483647]
                                      (or [-0x80000000, 0x7FFFFFFF] in hex)
                            "}),
                    );
                    0
                }
                _ => {
                    diagnostics.emit(
                        Diagnostic::bug(
                            source_file_id,
                            format!("unexpected error when parsing integer: {error}"),
                        )
                        .with_label(Label::primary(self.span, ""))
                        .with_note(notes::PARSER_BUG),
                    );
                    0
                }
            },
        }
    }

    pub fn parse(
        &self,
        input: &str,
        diagnostics: &mut dyn DiagnosticSink,
        source_file_id: SourceFileId,
    ) -> i32 {
        let int = self.span.get_input(input);
        self.map_parse_error(
            if int.starts_with("0x") || int.starts_with("0X") {
                i32::from_str_radix(&int[2..], 16)
            } else {
                int.parse()
            },
            diagnostics,
            source_file_id,
        )
    }
}

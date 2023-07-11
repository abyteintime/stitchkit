//! Commonly used diagnostic messages.

use indoc::indoc;
use muscript_foundation::{
    errors::{Diagnostic, Label},
    source::{SourceFileId, Span},
};
use muscript_syntax::lexis::token;

pub fn unnecessary_semicolon(source_file_id: SourceFileId, semi: token::Semi) -> Diagnostic {
    Diagnostic::warning(source_file_id, "unnecessary semicolon `;`")
        .with_label(Label::primary(semi.span, ""))
}

pub fn stmt_outside_of_function(source_file_id: SourceFileId, span: Span) -> Diagnostic {
    Diagnostic::error(source_file_id, "statement found outside of function")
        .with_label(Label::primary(span, "statements are not allowed here"))
        .with_note(indoc!("
            note: in contrast to most modern scripting languages, UnrealScript requires all executable code to belong
                    to a function. this is because code is executed in response to game events such as `Tick`;
                    it doesn't execute automatically like in Python or Lua
        "))
}

pub mod notes {
    use indoc::indoc;

    pub const CPP_UNSUPPORTED: &str = "note: MuScript does not support generating C++ headers";
    pub const ACCESS_UNSUPPORTED: &str = indoc! {"
        note: MuScript does not consider access modifiers at the moment;
              all items are treated as `public`
    "};
    pub const WIP: &str = "note: MuScript is still unfinished; you can help contribute at <https://github.com/abyteintime/stitchkit>";
}

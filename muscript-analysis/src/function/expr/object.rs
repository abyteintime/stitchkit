use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label, ReplacementSuggestion},
    ident::CaseInsensitive,
    source::{Span, Spanned},
};
use muscript_syntax::{
    cst,
    lexis::token::{Ident, NameLit},
};

use crate::{
    function::builder::FunctionBuilder,
    ir::{RegisterId, Value},
    Compiler, TypeId,
};

use super::ExprContext;

impl<'a> Compiler<'a> {
    pub(super) fn expr_object(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        outer: &cst::Expr,
        class_ident: Ident,
        name_lit: NameLit,
    ) -> RegisterId {
        let class_name = self.sources.span(builder.source_file_id, &class_ident);
        let object_name = name_lit.parse(self.sources.source(builder.source_file_id));

        if CaseInsensitive::new(class_name) == CaseInsensitive::new("class") {
            // Classes, despite being objects like any other, need to be special-cased because they
            // may come from the current script package which hasn't been fully compiled yet.

            // Classes within packages are not yet supported because it would be a bunch of extra
            // complication that effectively noone uses.
            if let Some(dot_index) = object_name.find('.') {
                let start = name_lit.span.start + 1 + dot_index as u32;
                self.env.emit(
                    Diagnostic::error(
                        builder.source_file_id,
                        "references to classes located within packages are not supported",
                    )
                    .with_label(Label::primary(Span::from(start..start + 1), ""))
                    .with_note((
                        "help: try referencing the class using just its name",
                        ReplacementSuggestion {
                            span: outer.span(),
                            replacement: format!("class'{}'", &object_name[dot_index + 1..]),
                        },
                    )),
                );
                return builder.ir.append_register(
                    outer.span(),
                    "unsupported_package_class_reference",
                    TypeId::ERROR,
                    Value::Void,
                );
            }

            self.env.emit(
                Diagnostic::error(
                    builder.source_file_id,
                    "class references are not yet implemented",
                )
                .with_label(Label::primary(class_ident.span, "")),
            );
            builder.ir.append_register(
                outer.span(),
                "unsupported_class_reference",
                TypeId::ERROR,
                Value::Void,
            )
        } else {
            self.env.emit(
                Diagnostic::error(
                    builder.source_file_id,
                    "object references are not yet implemented",
                )
                .with_label(Label::primary(class_ident.span, "")),
            );
            builder.ir.append_register(
                outer.span(),
                "unsupported_object_reference",
                TypeId::ERROR,
                Value::Void,
            )
        }
    }
}

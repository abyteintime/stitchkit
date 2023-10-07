use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    ident::CaseInsensitive,
    span::Spanned,
};
use muscript_syntax::{
    cst,
    token::{Ident, NameLit},
};

use crate::{
    function::builder::FunctionBuilder,
    ir::{RegisterId, Value},
    ClassId, Compiler, TypeId,
};

use super::ExprContext;

impl<'a> Compiler<'a> {
    pub(super) fn expr_object(
        &mut self,
        builder: &mut FunctionBuilder,
        _context: ExprContext,
        outer: &cst::Expr,
        class_ident: Ident,
        name_lit: NameLit,
    ) -> RegisterId {
        let class_name = self.sources.source(&class_ident);
        let object_name = name_lit.parse(&self.sources.as_borrowed()).to_owned();

        if CaseInsensitive::new(class_name) == CaseInsensitive::new("class") {
            // Classes, despite being objects like any other, need to be special-cased because they
            // may come from the current script package which hasn't been fully compiled yet.

            // Classes within packages are not yet supported because it would be a bunch of extra
            // complication that effectively noone uses.
            if let Some(dot_index) = object_name.find('.') {
                self.env.emit(
                    Diagnostic::error(
                        "references to classes located within packages are not supported",
                    )
                    .with_label(Label::primary(&name_lit, ""))
                    .with_note((
                        "help: try referencing the class using just its name",
                        self.sources.replacement_suggestion(
                            outer,
                            format!("class'{}'", &object_name[dot_index + 1..]),
                        ),
                    )),
                );
                return builder.ir.append_register(
                    outer.span(),
                    "unsupported_package_class_reference",
                    TypeId::ERROR,
                    Value::Void,
                );
            }

            if let Some(class_id) =
                self.lookup_class(builder.source_file_id, &object_name, name_lit.span())
            {
                let class_type_id = self.class_type_id(class_id);
                let class_package = self.class_package(class_id);
                return builder.ir.append_register(
                    outer.span(),
                    "class_reference",
                    class_type_id,
                    Value::Object {
                        class: ClassId::CLASS,
                        package: class_package.to_owned(),
                        name: self.env.class_name(class_id).to_owned(),
                    },
                );
            }

            builder.ir.append_register(
                outer.span(),
                "invalid_class_reference",
                TypeId::ERROR,
                Value::Void,
            )
        } else {
            self.env.emit(
                Diagnostic::error("object references are not yet implemented")
                    .with_label(Label::primary(outer, "")),
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

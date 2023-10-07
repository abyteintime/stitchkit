use std::fmt::Write as _;

use muscript_foundation::{
    self,
    errors::{Diagnostic, DiagnosticSink, Label, Note, NoteKind},
    span::Spanned,
};
use muscript_lexer::token::{Token, TokenSpan};
use muscript_syntax::{
    cst::{self, InfixOperator},
    token::{AnyToken, LeftParen, RightParen},
};

use crate::{
    function::{
        builder::FunctionBuilder,
        mangling::{mangled_operator_function_name, Operator},
        ParamFlags,
    },
    ir::{RegisterId, Value},
    Compiler, FunctionId, TypeId,
};

use super::{void_handling::registers_are_valid, ExpectedType, ExprContext};

pub struct CallSyntax<'a> {
    pub function: &'a cst::Expr,
    pub open: LeftParen,
    pub args: &'a [cst::Arg],
    pub close: RightParen,
}

impl<'a> Compiler<'a> {
    fn expr_operator(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        outer_span: TokenSpan,
        operator: &dyn Spanned<Token>,
        is_prefix: bool,
        arguments: &[RegisterId],
    ) -> RegisterId {
        let operator_str = self.sources.source(&operator.span());
        let operator_function_name = mangled_operator_function_name(Operator {
            operator: operator_str,
            argument_types: arguments
                .iter()
                .map(|&register_id| self.env.type_name(builder.ir.register(register_id).ty)),
            is_prefix,
        });
        if let Some(function_id) = self.lookup_function(builder.class_id, &operator_function_name) {
            builder.ir.append_register(
                outer_span,
                "op",
                self.env.get_function(function_id).return_ty,
                Value::CallFinal {
                    function: function_id,
                    arguments: arguments.to_owned(),
                },
            )
        } else {
            if registers_are_valid(&builder.ir, arguments) {
                let mut error = format!(
                    "no overload of {} `{}` exists for {} of type ",
                    if is_prefix {
                        "prefix operator"
                    } else {
                        "operator"
                    },
                    operator_str,
                    if arguments.len() > 1 {
                        "arguments"
                    } else {
                        "argument"
                    },
                );
                for (i, &register_id) in arguments.iter().enumerate() {
                    if i != 0 {
                        error.push_str(", ");
                    }
                    let type_name = self.env.type_name(builder.ir.register(register_id).ty);
                    _ = write!(error, "`{type_name}`");
                }
                self.env.emit(
                    Diagnostic::error( error)
                        .with_label(Label::primary(&operator.span(), ""))
                        .with_note(Note {
                            kind: NoteKind::Debug,
                            text: format!("this operator's mangled name is `{}`, which was not found in this scope", operator_function_name),
                            suggestion: None,
                        }),
                );
            }
            builder.ir.append_register(
                operator.span(),
                "op",
                context.expected_type.to_type_id(),
                Value::Void,
            )
        }
    }

    pub(super) fn expr_prefix(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        outer: &cst::Expr,
        operator: &AnyToken,
        right: &cst::Expr,
    ) -> RegisterId {
        // TODO: Specialize negation here so that it's interpreted as if it were a part of
        // the literal. Either that, or constant folding maybe?
        let right = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Any,
            },
            right,
        );
        self.expr_operator(builder, context, outer.span(), operator, true, &[right])
    }

    pub(super) fn expr_postfix(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        outer: &cst::Expr,
        operator: &AnyToken,
        left: &cst::Expr,
    ) -> RegisterId {
        let left = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Any,
            },
            left,
        );
        self.expr_operator(builder, context, outer.span(), operator, false, &[left])
    }

    pub(super) fn expr_infix(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        outer: &cst::Expr,
        operator: &InfixOperator,
        left: &cst::Expr,
        right: &cst::Expr,
    ) -> RegisterId {
        let left = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Any,
            },
            left,
        );
        let right = self.expr(
            builder,
            ExprContext {
                expected_type: ExpectedType::Any,
            },
            right,
        );
        self.expr_operator(
            builder,
            context,
            outer.span(),
            operator,
            false,
            &[left, right],
        )
    }

    pub(super) fn expr_call(
        &mut self,
        builder: &mut FunctionBuilder,
        context: ExprContext,
        outer: &cst::Expr,
        syntax: CallSyntax<'_>,
    ) -> RegisterId {
        let CallSyntax {
            function,
            open,
            args,
            close,
        } = syntax;

        if let cst::Expr::Ident(ident) = function {
            let name = self.sources.source(ident);
            if let Some(function_id) = self.lookup_function(builder.class_id, name) {
                let num_params = self.env.get_function(function_id).params.len();

                if args.len() > num_params {
                    let function = self.env.get_function(function_id);
                    self.env.emit(
                        Diagnostic::error(format!(
                            "too many parameters; expected {num_params}, but got {}",
                            args.len()
                        ))
                        .with_label(Label::primary(&args[num_params], ""))
                        .with_label(Label::secondary(&function.name, "function declared here")),
                    );
                }

                let mut arguments = vec![];
                let last_omitted = cst::Arg::Omitted(close.span());
                for i in 0..num_params {
                    let arg = args.get(i).unwrap_or(&last_omitted);
                    let arg = self.expr_call_arg(builder, function_id, arg, i);
                    arguments.push(arg);
                }

                let return_ty = self.env.get_function(function_id).return_ty;
                return builder.ir.append_register(
                    outer.span(),
                    "call",
                    return_ty,
                    Value::CallFinal {
                        function: function_id,
                        arguments,
                    },
                );
            } else {
                // TODO: There should be a better way of suppressing diagnostics within a scope.
                let num_diagnostics = self.env.diagnostics.len();
                let type_id = self.type_id(
                    builder.source_file_id,
                    builder.class_id,
                    &cst::Type {
                        specifiers: vec![],
                        path: cst::Path {
                            components: vec![*ident],
                        },
                        generic: None,
                        cpptemplate: None,
                    },
                );

                if type_id != TypeId::ERROR {
                    if let [arg] = args {
                        if let cst::Arg::Provided(value_expr) = arg {
                            return self.expr_cast(builder, outer, function, type_id, value_expr);
                        } else {
                            self.env
                                .emit(Diagnostic::error("type cast argument cannot be omitted"))
                        }
                    } else {
                        self.env.emit(
                            Diagnostic::error("type cast expects one argument")
                                .with_label(Label::primary(&open.span().join(&close.span()), "")),
                        )
                    }
                } else {
                    self.env
                        .diagnostics
                        .resize_with(num_diagnostics, || unreachable!("must only shrink"));
                    self.env.emit(
                        Diagnostic::error(format!(
                            "function `{name}` could not be found in this scope"
                        ))
                        .with_label(Label::primary(ident, "")),
                    )
                }
            }
        } else {
            self.env.emit(
                Diagnostic::error("expression cannot be called")
                    .with_label(Label::primary(
                        function,
                        "this expression does not denote a function",
                    ))
                    .with_label(Label::secondary(&open, "this `(` begins a function call"))
                    .with_label(Label::secondary(&close, "")),
                // TODO: Examples.
                // TODO: In case LHS is a macro, this needs a better error message.
                // .with_note("note: the left hand side of a function call must be:"),
            );
        }
        builder.ir.append_register(
            outer.span(),
            "call_invalid",
            context.expected_type.to_type_id(),
            Value::Void,
        )
    }

    fn expr_call_arg(
        &mut self,
        builder: &mut FunctionBuilder,
        function_id: FunctionId,
        arg: &cst::Arg,
        param_index: usize,
    ) -> RegisterId {
        let param = &self.env.get_function(function_id).params[param_index];
        let param_var = self.env.get_var(param.var);
        let param_ty = param_var.ty;
        let param_flags = param.flags;

        match arg {
            cst::Arg::Provided(expr) => {
                let value = self.expr(
                    builder,
                    ExprContext {
                        expected_type: ExpectedType::Matching(param_ty),
                    },
                    expr,
                );
                // TODO: Is it okay to pass non-places to `const out`?
                if builder.ir.register(value).ty != TypeId::ERROR
                    && param_flags.contains(ParamFlags::OUT)
                    && !builder.ir.is_place(value)
                {
                    self.env.emit(
                        Diagnostic::error("expression passed to `out` parameter must be a place")
                            .with_label(Label::primary(expr, "this is not a place in memory")),
                    );
                }
                self.coerce_expr(builder, value, param_ty)
            }
            cst::Arg::Omitted(span) => {
                let param = &self.env.get_function(function_id).params[param_index];
                let param_name = self.sources.source(&param_var.name);
                if !param.flags.contains(ParamFlags::OPTIONAL) {
                    self.env.emit(
                        Diagnostic::error(format!(
                            "required argument `{param_name}` was not provided"
                        ))
                        .with_label(Label::primary(span, "argument expected here..."))
                        .with_label(Label::primary(
                            &param_var.name,
                            "...to provide a value for this parameter",
                        )),
                    )
                }
                builder
                    .ir
                    .append_register(*span, "omitted_arg", param_ty, Value::Default)
            }
        }
    }
}

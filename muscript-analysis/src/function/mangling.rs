use std::borrow::Cow;

use heck::ToPascalCase;
use muscript_foundation::source::{SourceFileId, SourceFileSet};
use muscript_syntax::cst;

/// Mangles a function's name more or less according to Unreal's own rules.
///
/// - The names of normal functions, events, and delegates are not mangled.
/// - The names of prefix operators are mangled to `{op}_Pre{right}`.
/// - The names of postfix operators are mangled to `{op}_{left}`.
/// - The names of infix operators are mangled to `{op}_{left}{right}`.
/// - Replace `{op}` with the operator's name, `{left}` with its left-hand side, and `{right}`
///   with its right-hand side.
///
/// You can tell whether the name was mangled by looking at whether the return type is [`Owned`]
/// or [`Borrowed`].
///
/// Note that this mangling is not *exactly* the same as Unreal. It's only really enough to analyze
/// the engine source code correctly.
///
/// [`Owned`]: Cow::Owned
/// [`Borrowed`]: Cow::Borrowed
pub fn mangled_function_name<'a>(
    sources: &'a SourceFileSet,
    source_file_id: SourceFileId,
    function: &cst::ItemFunction,
) -> Cow<'a, str> {
    let source = &sources.get(source_file_id).source;
    let function_name = function.name.span.get_input(source);
    match function.kind {
        // Not sure if delegates should be mangled or not.
        cst::FunctionKind::Function(_)
        | cst::FunctionKind::Event(_)
        | cst::FunctionKind::Delegate(_) => Cow::Borrowed(function_name),
        cst::FunctionKind::Operator(_, _)
        | cst::FunctionKind::PreOperator(_)
        | cst::FunctionKind::PostOperator(_) => {
            let mut mangled = format!(
                "{}_{}",
                operator_name(function_name),
                if let cst::FunctionKind::PreOperator(_) = &function.kind {
                    "Pre"
                } else {
                    ""
                }
            );
            for param in &function.params.params {
                mangled.push_str(&mangled_type_name(sources, source_file_id, &param.ty));
            }
            Cow::Owned(mangled)
        }
    }
}

pub fn operator_name(operator: &str) -> String {
    if operator.chars().all(char::is_alphanumeric) {
        operator.to_owned()
    } else {
        operator.chars().filter_map(operator_char_name).collect()
    }
}

pub fn operator_char_name(c: char) -> Option<&'static str> {
    match c {
        '+' => Some("Add"),
        '-' => Some("Subtract"),
        '*' => Some("Multiply"),
        '/' => Some("Divide"),
        '%' => Some("Percent"),
        '$' => Some("Concat"),
        '@' => Some("At"),
        '<' => Some("Less"),
        '>' => Some("Greater"),
        '~' => Some("Complement"),
        '&' => Some("And"),
        '|' => Some("Or"),
        '^' => Some("Xor"),
        '!' => Some("Not"),
        '=' => Some("Equal"),
        _ => None,
    }
}

/// Mangle a type name.
///
/// Note that this mangling scheme is really primitive; it ignores path segments completely,
/// therefore if you have two preoperators which take `A.T` and `B.T`, they will be considered
/// the same function, and as such the code will fail to compile.
///
/// Generics `Generic<Int, Float>` are mangled to `Generic+lInt+cFloat+g`, because they do not need
/// compatibility with vanilla packages, as no operators ever use generic arguments.
/// `+l` is meant to represent **l**ess-than, `+c` **c**ommas, and `+g` **g**reater-than.
pub fn mangled_type_name<'a>(
    sources: &'a SourceFileSet,
    source_file_id: SourceFileId,
    ty: &cst::Type,
) -> Cow<'a, str> {
    let source = &sources.get(source_file_id).source;
    let ty_name_ident = ty
        .path
        .components
        .last()
        .expect("path must have more than zero components");
    let ty_name = ty_name_ident.span.get_input(source);
    if let Some(generic) = &ty.generic {
        let mut builder = String::from(ty_name);
        builder.push_str("+l");
        for (i, arg) in generic.args.iter().enumerate() {
            if i != 0 {
                builder.push_str("+c");
            }
            builder.push_str(&mangled_type_name(sources, source_file_id, arg));
        }
        builder.push_str("+g");
        Cow::Owned(builder)
    } else if matches!(ty_name.chars().next(), Some(c) if c.is_ascii_uppercase()) {
        Cow::Borrowed(ty_name)
    } else {
        Cow::Owned(ty_name.to_pascal_case())
    }
}

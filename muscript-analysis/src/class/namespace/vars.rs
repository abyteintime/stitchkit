use muscript_foundation::{ident::CaseInsensitive, span::Spanned};
use muscript_syntax::cst::{self, ItemName, NamedItem};
use tracing::info_span;

use crate::{
    class::{Var, VarFlags, VarKind},
    function::{
        builder::FunctionBuilder,
        expr::{ExpectedType, ExprContext},
        Function, FunctionFlags, FunctionImplementation, FunctionKind,
    },
    ir::{interpret::Constant, Terminator},
    partition::{UntypedClassPartitionsExt, VarCst},
    ClassId, Compiler, TypeId, VarId,
};

/// # Class variables
impl<'a> Compiler<'a> {
    pub fn class_var(&mut self, class_id: ClassId, name: &str) -> Option<VarId> {
        let namespace = self.env.class_namespace(class_id);
        if !namespace.vars.contains_key(CaseInsensitive::new_ref(name)) {
            if let Some(partitions) = self.untyped_class_partitions(class_id) {
                if let Some(cst) = partitions.find_var(name) {
                    // Cloning here is kind of inefficient, but otherwise we hold a reference
                    // to the class partitions and thus we cannot register variables within the
                    // environment.
                    let cst = cst.clone();
                    let var_id = self.create_class_var(cst, class_id);
                    let namespace = self.env.class_namespace_mut(class_id);
                    namespace
                        .vars
                        .insert(CaseInsensitive::new(name.to_owned()), Some(var_id));
                }
            }
        }
        let namespace = self.env.class_namespace_mut(class_id);
        namespace
            .vars
            .get(CaseInsensitive::new_ref(name))
            .and_then(|x| x.as_ref())
            .copied()
    }

    fn create_class_var(&mut self, cst: VarCst, class_id: ClassId) -> VarId {
        let name = cst.name();
        let var = match cst {
            VarCst::Const(item_const) => {
                let constant = self.evaluate_const(class_id, name, &item_const.value);
                Var {
                    name,
                    ty: constant.type_id(),
                    kind: VarKind::Const(constant),
                }
            }
            VarCst::Var(item_var) => Var {
                name,
                ty: self.type_id(class_id, &item_var.ty),
                kind: VarKind::Var(VarFlags::from_cst(
                    self.env,
                    &self.sources.as_borrowed(),
                    &item_var.specifiers,
                )),
            },
        };
        self.env.register_var(var)
    }

    fn evaluate_const(
        &mut self,
        class_id: ClassId,
        name_ident: ItemName,
        value: &cst::Expr,
    ) -> Constant {
        let name_str = self.sources.source(&name_ident);
        let function_id = self.env.register_function(Function {
            class_id,
            mangled_name: format!("const-{name_str}"),
            name: name_ident,
            return_ty: TypeId::VOID,
            params: vec![],
            flags: FunctionFlags::empty(),
            kind: FunctionKind::Function,
            implementation: FunctionImplementation::Script,
        });
        let function = self.env.get_function(function_id);
        let mut builder = FunctionBuilder::new(function_id, function, value.span());
        let expr_register = self.expr(
            &mut builder,
            ExprContext {
                expected_type: ExpectedType::Any,
            },
            value,
        );
        builder.ir.set_terminator(Terminator::Return(expr_register));
        self.eval_ir(&builder.ir)
    }

    pub fn all_var_names(&mut self, class_id: ClassId) -> &[String] {
        if self.env.class_namespace(class_id).all_var_names.is_none() {
            let all_var_names = if let Some(partitions) = self.untyped_class_partitions(class_id) {
                partitions
                    .iter()
                    .flat_map(|partition| partition.vars.keys().map(|ci| (**ci).clone()))
                    .collect()
            } else {
                vec![]
            };
            let namespace = self.env.class_namespace_mut(class_id);
            namespace.all_var_names = Some(all_var_names);
        }
        self.env
            .class_namespace(class_id)
            .all_var_names
            .as_ref()
            .unwrap()
    }

    pub fn class_vars(&mut self, class_id: ClassId) -> Vec<VarId> {
        let _span = info_span!(
            "class_vars",
            ?class_id,
            class_name = self.env.class_name(class_id)
        )
        .entered();

        // This clone is less than optimal, but in theory this function should only ever be called
        // once per class (ie. whenever the class is to be emitted,) so not much slowness should
        // happen. *In theory.*
        let all_var_names = self.all_var_names(class_id).to_owned();
        all_var_names
            .iter()
            .filter_map(|name| self.class_var(class_id, name))
            .collect()
    }

    pub fn lookup_class_var(&mut self, class_id: ClassId, name: &str) -> Option<VarId> {
        self.class_var(class_id, name).or_else(|| {
            self.super_class_id(class_id)
                .and_then(|class_id| self.lookup_class_var(class_id, name))
        })
    }
}

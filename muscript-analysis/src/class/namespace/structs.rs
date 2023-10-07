use std::{collections::HashMap, rc::Rc};

use muscript_foundation::{
    errors::{Diagnostic, DiagnosticSink, Label},
    ident::CaseInsensitive,
    source::SourceFileId,
    span::Spanned,
};
use muscript_syntax::cst::{self, ItemName};

use crate::{
    class::{Var, VarFlags, VarKind},
    partition::{ItemSingleVar, TypeCst, UntypedStruct},
    type_system::Type,
    ClassId, Compiler, VarId,
};

// Big TODO: Structs are currently identified via (class_id, type_name) pairs. This isn't ideal
// because the type name is a string which isn't very cheap to clone.
// At some point we should introduce a struct ID just like we have class IDs.

#[derive(Debug, Default)]
enum SuperStruct {
    #[default]
    Unknown,
    Erroneous,
    Known(ClassId, Rc<str>),
}

#[derive(Debug, Default)]
pub struct ClassStruct {
    super_struct: SuperStruct,
    pub fields: HashMap<CaseInsensitive<String>, Option<VarId>>,
}

impl<'a> Compiler<'a> {
    fn untyped_struct<'x>(
        &'x mut self,
        class_id: ClassId,
        struct_name: &str,
    ) -> Option<(SourceFileId, &'x UntypedStruct)> {
        if let Some(partitions) = self.untyped_class_partitions(class_id) {
            let source_file_and_cst = partitions.iter().find_map(|partition| {
                partition
                    .types
                    .get(CaseInsensitive::new_ref(struct_name))
                    .map(|cst| (partition.source_file_id, cst))
            });
            if let Some((source_file_id, TypeCst::Struct(cst))) = source_file_and_cst {
                return Some((source_file_id, cst));
            }
        }
        None
    }

    pub fn class_struct_mut(
        &mut self,
        class_id: ClassId,
        struct_name: &str,
    ) -> Option<&mut ClassStruct> {
        // Create the struct and discard the immutable reference.
        let _ = self.class_struct(class_id, struct_name);

        let namespace = self.env.class_namespace_mut(class_id);
        namespace
            .structs
            .get_mut(CaseInsensitive::new_ref(struct_name))
            .and_then(|x| x.as_mut())
    }

    pub fn class_struct(&mut self, class_id: ClassId, struct_name: &str) -> Option<&ClassStruct> {
        let namespace = self.env.class_namespace_mut(class_id);
        if !namespace
            .structs
            .contains_key(CaseInsensitive::new_ref(struct_name))
        {
            let cst = self
                .untyped_struct(class_id, struct_name)
                .map(|_| ClassStruct::default());
            let namespace = self.env.class_namespace_mut(class_id);
            namespace
                .structs
                .insert(CaseInsensitive::new(struct_name.to_owned()), cst);
        }
        let namespace = self.env.class_namespace_mut(class_id);
        namespace
            .structs
            .get(CaseInsensitive::new_ref(struct_name))
            .and_then(|x| x.as_ref())
    }

    pub fn struct_var(
        &mut self,
        class_id: ClassId,
        struct_name: &str,
        field_name: &str,
    ) -> Option<VarId> {
        if let Some(class_struct) = self.class_struct_mut(class_id, struct_name) {
            if !class_struct
                .fields
                .contains_key(CaseInsensitive::new_ref(field_name))
            {
                let var_id = if let Some((source_file_id, untyped_struct)) =
                    self.untyped_struct(class_id, struct_name)
                {
                    untyped_struct
                        .vars
                        .get(CaseInsensitive::new_ref(field_name))
                        // Somewhat annoyed at the fact we have to clone here, but at least the
                        // result is memoized.
                        .cloned()
                        .map(|item_var| self.create_struct_var(source_file_id, class_id, item_var))
                } else {
                    None
                };
                let class_struct = self.class_struct_mut(class_id, struct_name).unwrap();
                class_struct
                    .fields
                    .insert(CaseInsensitive::new(field_name.to_owned()), var_id);
            }
        }

        self.class_struct(class_id, struct_name)
            .and_then(|class_struct| {
                class_struct
                    .fields
                    .get(CaseInsensitive::new_ref(field_name))
                    .copied()
                    .flatten()
            })
    }

    fn create_struct_var(
        &mut self,
        source_file_id: SourceFileId,
        class_id: ClassId,
        cst: ItemSingleVar,
    ) -> VarId {
        self.check_struct_var_specifiers(source_file_id, &cst.specifiers);
        let var = Var {
            source_file_id,
            name: ItemName::from_spanned(&cst.variable.name),
            ty: self.type_id(source_file_id, class_id, &cst.ty),
            // For now we reuse class flags despite some of them not being
            // meaningful in structs.
            kind: VarKind::Var(VarFlags::from_cst(
                self.env,
                &self.sources.as_borrowed(),
                source_file_id,
                &cst.specifiers,
            )),
        };
        self.env.register_var(var)
    }

    fn check_struct_var_specifiers(
        &mut self,
        source_file_id: SourceFileId,
        specifiers: &[cst::VarSpecifier],
    ) {
        for specifier in specifiers {
            match specifier {
                // NOTE: This list may be inaccurate. This is just a guess as to which specifiers
                // have an effect on structs. Needs verification or something.
                cst::VarSpecifier::BitWise(_)
                | cst::VarSpecifier::Deprecated(_)
                | cst::VarSpecifier::DuplicateTransient(_)
                | cst::VarSpecifier::EditConst(_)
                | cst::VarSpecifier::EditHide(_)
                | cst::VarSpecifier::EditFixedSize(_)
                | cst::VarSpecifier::EditInline(_)
                | cst::VarSpecifier::EditInlineUse(_)
                | cst::VarSpecifier::EditorOnly(_)
                | cst::VarSpecifier::EditTextBox(_)
                | cst::VarSpecifier::Init(_)
                | cst::VarSpecifier::NoClear(_)
                | cst::VarSpecifier::NoExport(_)
                | cst::VarSpecifier::NoImport(_)
                | cst::VarSpecifier::NonTransactional(_)
                | cst::VarSpecifier::NotForConsole(_)
                | cst::VarSpecifier::Private(_, _)
                | cst::VarSpecifier::PrivateWrite(_)
                | cst::VarSpecifier::Protected(_, _)
                | cst::VarSpecifier::ProtectedWrite(_, _)
                | cst::VarSpecifier::Public(_, _)
                | cst::VarSpecifier::Serialize(_)
                | cst::VarSpecifier::SerializeText(_)
                | cst::VarSpecifier::Type(_) => (),

                cst::VarSpecifier::Config(_)
                | cst::VarSpecifier::CrossLevelActive(_)
                | cst::VarSpecifier::CrossLevelPassive(_)
                | cst::VarSpecifier::DataBinding(_)
                | cst::VarSpecifier::Export(_)
                | cst::VarSpecifier::GlobalConfig(_)
                | cst::VarSpecifier::Input(_)
                | cst::VarSpecifier::Instanced(_)
                | cst::VarSpecifier::Interp(_)
                | cst::VarSpecifier::Localized(_)
                | cst::VarSpecifier::RepNotify(_) => {
                    // Could probably use better error messages explaining why a particular
                    // specifier is banned.
                    self.env.emit(
                        Diagnostic::error("specifier cannot be used in struct variables")
                            .with_label(Label::primary(
                                specifier,
                                "this specifier cannot be used in a struct",
                            )),
                    )
                }
            }
        }
    }

    pub fn super_struct(
        &mut self,
        class_id: ClassId,
        struct_name: &str,
    ) -> Option<(ClassId, &Rc<str>)> {
        if let Some(SuperStruct::Unknown) = self
            .class_struct_mut(class_id, struct_name)
            .map(|x| &x.super_struct)
        {
            if let Some((source_file_id, untyped_struct)) =
                self.untyped_struct(class_id, struct_name)
            {
                if let Some(extends) = untyped_struct.extends.clone() {
                    let span = extends.span();
                    let ty = self.type_id(
                        source_file_id,
                        class_id,
                        &cst::Type {
                            specifiers: vec![],
                            path: extends,
                            generic: None,
                            cpptemplate: None,
                        },
                    );
                    let &Type::Struct { outer } = self.env.get_type(ty) else {
                        // TODO: Tell the user here what the type actually is?
                        // TODO: Point to the declaration of the mismatched type?
                        self.env.emit(
                            Diagnostic::error("base type of a struct must also be a struct")
                                .with_label(Label::primary(&span, "this is not a struct type")),
                        );
                        let class_struct = self
                            .class_struct_mut(class_id, struct_name)
                            .expect("should be Some since the outer `if let` matched");
                        class_struct.super_struct = SuperStruct::Erroneous;
                        return None;
                    };
                    let type_name = self.env.type_name(ty).to_string();
                    let class_struct = self
                        .class_struct_mut(class_id, struct_name)
                        .expect("should be Some since the outer `if let` matched");
                    class_struct.super_struct = SuperStruct::Known(outer, Rc::from(type_name));
                }
            }
        }

        match self
            .class_struct(class_id, struct_name)
            .map(|x| &x.super_struct)
        {
            Some(SuperStruct::Known(class_id, struct_name)) => Some((*class_id, struct_name)),
            Some(SuperStruct::Erroneous | SuperStruct::Unknown) | None => None,
        }
    }

    pub fn lookup_struct_var(
        &mut self,
        class_id: ClassId,
        struct_name: &str,
        field_name: &str,
    ) -> Option<VarId> {
        self.struct_var(class_id, struct_name, field_name)
            .or_else(|| {
                if let Some((class_id, struct_name)) = self.super_struct(class_id, struct_name) {
                    let struct_name = Rc::clone(struct_name);
                    self.lookup_struct_var(class_id, &struct_name, field_name)
                } else {
                    None
                }
            })
    }
}

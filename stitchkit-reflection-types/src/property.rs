#![allow(clippy::manual_strip)]

pub mod any;
pub mod defaults;

use std::num::NonZeroU32;

use bitflags::bitflags;
use stitchkit_archive::{index::OptionalPackageObjectIndex, name::ArchivedName};
use stitchkit_core::{serializable_bitflags, Deserialize};
use tracing::warn;

use crate::Field;

#[derive(Debug, Clone, Deserialize)]
pub struct Property {
    pub field: Field<ArchivedName>,
    /// The size of an array property; 1 means that the variable is not an array but a single
    /// element.
    pub array_length: NonZeroU32,
    pub flags: PropertyFlags,
    /// The variable's category in the editor.
    pub category: ArchivedName,
    /// When specified, enum elements are used as array indices instead of plain integers.
    pub index_enum: OptionalPackageObjectIndex,
    /// Purpose currently unknown; only present when `flags` contains [`REPLICATED`].
    /// Incremented by 10 for every `if` clause in a `replication` statement.
    ///
    /// [`REPLICATED`]: [`PropertyFlags`::REPLICATED]
    #[serialized_when(flags.contains(PropertyFlags::REPLICATED))]
    pub replication_index: Option<u16>,
}

bitflags! {
    /// Property flags.
    ///
    /// Most of these flags are triggered by specifiers existing on a property declaration.
    /// Descriptions of functionality are mostly taken from the [old Unreal wiki].
    ///
    /// [old Unreal wiki]: https://unrealwiki.unrealsp.org/index.php/Variables
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PropertyFlags: u64 {
        /// Property is visible in the editor.
        ///
        /// In case this is set, the `category` field of [`Property`] specifies which category the
        /// property belongs to, defaulting to the class name if nothing is specified in parentheses
        /// (like `var() int MyProperty;`.)
        /// Otherwise the category is the name `'None'`.
        const EDITOR                   = 0x0000000000000001;
        /// Property is `const` - read-only in UnrealScript, but writable in C++.
        const CONST                    = 0x0000000000000002;
        /// Property is `input`.
        const INPUT                    = 0x0000000000000004;
        /// Property is `export` - exported as a subobject when copying the object to clipboard.
        const EXPORT                   = 0x0000000000000008;
        /// Property is an `optional` parameter.
        const OPTIONAL_PARAM           = 0x0000000000000010;
        /// Property is used in a `replication` block.
        const REPLICATED               = 0x0000000000000020;
        /// Array property has `editfixedsize` specifier - this disallows adding or removing
        /// elements from a dynamic array in the editor.
        const EDIT_FIXED_SIZE          = 0x0000000000000040;
        /// Property is a parameter.
        const PARAM                    = 0x0000000000000080;
        /// Property is an `out` parameter.
        const OUT_PARAM                = 0x0000000000000100;
        /// Property is the return value of a function.
        const RETURN_VALUE             = 0x0000000000000400;
        /// Property should coerce arguments automatically to its type.
        const COERCE                   = 0x0000000000000800;
        /// Property is `native` - exported to C++.
        const NATIVE                   = 0x0000000000001000;
        /// Property is `transient` - not serialized.
        const TRANSIENT                = 0x0000000000002000;
        /// Property is `config`.
        const CONFIG                   = 0x0000000000004000;
        /// Property is `localized`.
        const LOCALIZED                = 0x0000000000008000;
        /// Property is `editconst` - cannot be changed in the editor.
        const EDIT_CONST               = 0x0000000000020000;
        /// Property is `globalconfig`. Always appears next to `config`.
        const GLOBAL_CONFIG            = 0x0000000000040000;
        /// Present on all owned properties which indirectly own a component, such as struct fields
        /// with components inside them.
        ///
        /// Note that this is only applied to properties which own a component, not merely reference
        /// it (as may be the case with `COMPONENT`.)
        const CONTAINS_OWNED_COMPONENT = 0x0000000000080000;
        /// Property is stored in a `transient` struct.
        const STRUCT_TRANSIENT         = 0x0000000000100000;
        /// Property is `duplicatetransient` - not serialized and not preserved when duplicating
        /// the object.
        const DUPLICATE_TRANSIENT      = 0x0000000000200000;
        /// Property of this type has a destructor which must run when the parent object is
        /// destroyed.
        const DROP                     = 0x0000000000400000;
        /// Property is `noexport` - this is the opposite of `export` for types that are
        /// automatically serialized when copying objects to the clipboard.
        const NO_EXPORT                = 0x0000000000800000;
        /// Property is `noexport` - the property will be ignored when pasting from the clipboard.
        const NO_IMPORT                = 0x0000000001000000;
        /// Property is `noclear` - cannot be cleared to `None` in the editor.
        const NO_CLEAR                 = 0x0000000002000000;
        /// Property is `editinline - its fields are inlined into a class's editor.
        /// This is also present on component properties.
        const EDIT_INLINE              = 0x0000000004000000;
        /// Integer property is `bitwise`, meaning that each bit is like a single bool.
        /// Probably used in the editor, haven't tested this.
        const BITWISE                  = 0x0000000008000000;
        /// Redundant version of `EDIT_INLINE`.
        const EDIT_INLINE_USE          = 0x0000000010000000;
        /// Property is `deprecated` and should not be used.
        const DEPRECATED               = 0x0000000020000000;
        /// Property is declared as a `databinding`.
        const DATA_BINDING             = 0x0000000040000000;
        /// Property is `serializetext`. Works only on `native` properties. The property is not
        /// saved, but its data is transferred during copy-paste operations.
        const SERIALIZE_TEXT           = 0x0000000080000000;
        /// Property is `repnotify` - when this property is replicated, `ReplicatedEvent` is called
        /// with the property's name.
        const REP_NOTIFY               = 0x0000000100000000;
        /// Property is `interp`olated, which means it can be used in Matinee sequences.
        const INTERP                   = 0x0000000200000000;
        /// Property is `nontransactional` - it will not be affected by undo and redo in the editor.
        const NON_TRANSACTIONAL        = 0x0000000400000000;
        /// Property is `editoronly` - it will not be loaded outside the editor.
        const EDITOR_ONLY              = 0x0000000800000000;
        /// Property is `notforconsole` - discarded on console targets.
        const NOT_FOR_CONSOLE          = 0x0000001000000000;
        /// Property is `privatewrite` - it can only be written within the class it's
        /// declared in.
        const PRIVATE_WRITE            = 0x0000004000000000;
        /// Property is `protectedwrite` - it can only be written within the class it's declared
        /// in and its descendants.
        const PROTECTED_WRITE          = 0x0000008000000000;
        /// Property is `edithide` - hidden in the editor.
        const EDIT_HIDE                = 0x0000020000000000;
        /// Property is `edittextbox`. This is used in `MaterialExpressionCustom`, probably to
        /// display a bigger text box for entering the expression?
        const EDIT_TEXT_BOX            = 0x0000040000000000;
        /// Property is `crosslevelactive`.
        const CROSS_LEVEL_ACTIVE       = 0x0000100000000000;
        /// Property is `crosslevelpassive`.
        const CROSS_LEVEL_PASSIVE      = 0x0000200000000000;
        /// A Hat in Time-specific `serialize` specifier; probably used for marking variables as
        /// saved in the game save.
        const SERIALIZE                = 0x0000400000000000;
    }
}

serializable_bitflags! {
    type PropertyFlags;
    validate |flags| {
        if flags.contains(PropertyFlags::GLOBAL_CONFIG) && !flags.contains(PropertyFlags::CONFIG) {
            warn!("Property flags contain GLOBAL_CONFIG without CONFIG")
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ByteProperty {
    pub base: Property,
    /// When not None, specifies that this property is an enum and that the provided enum object
    /// should be used.
    pub enum_object: OptionalPackageObjectIndex,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntProperty {
    pub base: Property,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FloatProperty {
    pub base: Property,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StringProperty {
    pub base: Property,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NameProperty {
    pub base: Property,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArrayProperty {
    pub base: Property,
    /// Property specifying the type of the array (the `T` specified in angle brackets in
    /// `array<T>`).
    pub item_property: OptionalPackageObjectIndex,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObjectProperty {
    pub base: Property,
    /// The class of the object.
    pub object_class: OptionalPackageObjectIndex,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClassProperty {
    pub base: Property,
    /// The `Class` class itself. Included here for _some_ reason.
    pub class: OptionalPackageObjectIndex,
    /// The super class specified in angle brackets with `class<T>`; `Object` if not specified.
    pub super_class: OptionalPackageObjectIndex,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InterfaceProperty {
    pub base: Property,
    /// The class of the stored interface.
    pub interface_class: OptionalPackageObjectIndex,
}

/// A delegate property.
///
/// The way these are encoded is kind of weird, and UCC exports `DelegateProperty` in two
/// different cases:
/// - For delegate variables. In delegate variables, `delegate_function` and `delegate_function_2`
///   contain the same value.
/// - For delegate declarations. In delegate declarations, `delegate_function_2` is zero.
#[derive(Debug, Clone, Deserialize)]
pub struct DelegateProperty {
    pub base: Property,
    /// The delegate function.
    pub delegate_function: OptionalPackageObjectIndex,
    /// When the property is a variable, contains the same data as `delegate_function_1`.
    /// Otherwise is None.
    pub delegate_function_2: OptionalPackageObjectIndex,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StructProperty {
    pub base: Property,
    /// The struct type.
    pub struct_type: OptionalPackageObjectIndex,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComponentProperty {
    pub base: Property,
    /// The component's class.
    pub component_class: OptionalPackageObjectIndex,
}

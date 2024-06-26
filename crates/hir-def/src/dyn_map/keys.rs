//! keys to be used with `DynMap`

use crate::{
    EnumId, EnumStructId, FieldId, FuncenumId, FunctagId, FunctionId, GlobalId, MacroId,
    MethodmapId, NodePtr, PropertyId, StructFieldId, StructId, TypedefId, TypesetId, VariantId,
};

pub type Key<K, V> = crate::dyn_map::Key<K, V>;

pub const FUNCTION: Key<NodePtr, FunctionId> = Key::new();
pub const MACRO: Key<NodePtr, MacroId> = Key::new();
pub const GLOBAL: Key<NodePtr, GlobalId> = Key::new();
pub const ENUM_STRUCT: Key<NodePtr, EnumStructId> = Key::new();
pub const METHODMAP: Key<NodePtr, MethodmapId> = Key::new();
pub const PROPERTY: Key<NodePtr, PropertyId> = Key::new();
pub const ENUM: Key<NodePtr, EnumId> = Key::new();
pub const ENUM_VARIANT: Key<NodePtr, VariantId> = Key::new();
pub const TYPEDEF: Key<NodePtr, TypedefId> = Key::new();
pub const TYPESET: Key<NodePtr, TypesetId> = Key::new();
pub const FUNCTAG: Key<NodePtr, FunctagId> = Key::new();
pub const FUNCENUM: Key<NodePtr, FuncenumId> = Key::new();
pub const STRUCT: Key<NodePtr, StructId> = Key::new();
pub const STRUCT_FIELD: Key<NodePtr, StructFieldId> = Key::new();
pub const FIELD: Key<NodePtr, FieldId> = Key::new();

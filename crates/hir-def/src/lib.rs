use core::hash::Hash;
use item_tree::{
    AstId, EnumStruct, Funcenum, Functag, Function, ItemTreeNode, Macro, Methodmap, Property,
    Struct, Typedef, Typeset, Variable, Variant,
};
use item_tree::{Enum, ItemTree};
use la_arena::Idx;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::{hash::Hasher, sync::Arc};
use stdx::impl_from;
use vfs::FileId;

mod ast_id_map;
pub mod body;
pub mod child_by_source;
mod data;
pub mod db;
mod diagnostics;
pub mod dyn_map;
mod hir;
mod infer;
mod item_tree;
pub mod resolver;
pub mod src;

pub use ast_id_map::NodePtr;
pub use data::{MethodmapExtension, PropertyItem};
pub use db::resolve_include_node;
pub use db::DefDatabase;
pub use db::{BlockDefMapQuery, BlockItemTreeQuery, BodyQuery, FileDefMapQuery, FileItemTreeQuery};
pub use diagnostics::DefDiagnostic;
pub use hir::type_ref::type_string_from_node;
pub use hir::ExprId;
pub use infer::{AttributeId, ConstructorDiagnosticKind, InferenceDiagnostic, InferenceResult};
pub use item_tree::{
    print_item_tree, FileItem, FunctionKind, Name, RawVisibilityId, SpecialMethod,
};

trait Intern {
    type ID;
    fn intern(self, db: &dyn db::DefDatabase) -> Self::ID;
}

pub trait Lookup {
    type Data;
    fn lookup(&self, db: &dyn db::DefDatabase) -> Self::Data;
}

macro_rules! impl_intern_key {
    ($name:ident) => {
        impl salsa::InternKey for $name {
            fn from_intern_id(v: salsa::InternId) -> Self {
                $name(v)
            }
            fn as_intern_id(&self) -> salsa::InternId {
                self.0
            }
        }
    };
}

macro_rules! impl_intern {
    ($id:ident, $loc:ident, $intern:ident, $lookup:ident) => {
        impl_intern_key!($id);

        impl Intern for $loc {
            type ID = $id;
            fn intern(self, db: &dyn db::DefDatabase) -> $id {
                db.$intern(self)
            }
        }

        impl Lookup for $id {
            type Data = $loc;
            fn lookup(&self, db: &dyn db::DefDatabase) -> $loc {
                db.$lookup(*self)
            }
        }
    };
}

macro_rules! impl_serde {
    ($id:ident) => {
        impl Serialize for $id {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_u32(self.0.into())
            }
        }

        impl<'de> Deserialize<'de> for $id {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct Vis;

                impl<'de> Visitor<'de> for Vis {
                    type Value = $id;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a u32 representing a salsa::InternId")
                    }

                    fn visit_u32<E>(self, value: u32) -> Result<$id, E>
                    where
                        E: de::Error,
                    {
                        Ok($id(salsa::InternId::from(value)))
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<$id, E>
                    where
                        E: de::Error,
                    {
                        // Numbers are represented as u64 in JSON.
                        Ok($id(salsa::InternId::from(value as u32)))
                    }
                }

                deserializer.deserialize_u32(Vis)
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemContainerId {
    FileId(FileId),
    EnumId(EnumId),
    EnumStructId(EnumStructId),
    MethodmapId(MethodmapId),
    TypesetId(TypesetId),
    FuncenumId(FuncenumId),
}
impl_from!(FileId, EnumId, EnumStructId, MethodmapId, TypesetId, FuncenumId for ItemContainerId);

/// A Data Type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AdtId {
    EnumStructId(EnumStructId),
    MethodmapId(MethodmapId),
}
impl_from!(EnumStructId, MethodmapId for AdtId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(salsa::InternId);
type FunctionLoc = AssocItemLoc<Function>;
impl_intern!(
    FunctionId,
    FunctionLoc,
    intern_function,
    lookup_intern_function
);
impl_serde!(FunctionId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MacroId(salsa::InternId);
type MacroLoc = AssocItemLoc<Macro>;
impl_intern!(MacroId, MacroLoc, intern_macro, lookup_intern_macro);
impl_serde!(MacroId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EnumStructId(salsa::InternId);
type EnumStructLoc = AssocItemLoc<EnumStruct>;
impl_intern!(
    EnumStructId,
    EnumStructLoc,
    intern_enum_struct,
    lookup_intern_enum_struct
);
impl_serde!(EnumStructId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldId {
    pub parent: EnumStructId,
    pub local_id: LocalFieldId,
}

pub type LocalFieldId = Idx<data::EnumStructItemData>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MethodmapId(salsa::InternId);
type MethodmapLoc = AssocItemLoc<Methodmap>;
impl_intern!(
    MethodmapId,
    MethodmapLoc,
    intern_methodmap,
    lookup_intern_methodmap
);
impl_serde!(MethodmapId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PropertyId(salsa::InternId);
type PropertyLoc = AssocItemLoc<Property>;
impl_intern!(
    PropertyId,
    PropertyLoc,
    intern_property,
    lookup_intern_property
);
impl_serde!(PropertyId);

pub type LocalPropertyId = Idx<data::MethodmapItemData>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumId(salsa::InternId);
type EnumLoc = AssocItemLoc<Enum>;
impl_intern!(EnumId, EnumLoc, intern_enum, lookup_intern_enum);
impl_serde!(EnumId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariantId(salsa::InternId);
type VariantLoc = AssocItemLoc<Variant>;
impl_intern!(VariantId, VariantLoc, intern_variant, lookup_intern_variant);
impl_serde!(VariantId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypedefId(salsa::InternId);
type TypedefLoc = AssocItemLoc<Typedef>;
impl_intern!(TypedefId, TypedefLoc, intern_typedef, lookup_intern_typedef);
impl_serde!(TypedefId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypesetId(salsa::InternId);
type TypesetLoc = AssocItemLoc<Typeset>;
impl_intern!(TypesetId, TypesetLoc, intern_typeset, lookup_intern_typeset);
impl_serde!(TypesetId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctagId(salsa::InternId);
type FunctagLoc = AssocItemLoc<Functag>;
impl_intern!(FunctagId, FunctagLoc, intern_functag, lookup_intern_functag);
impl_serde!(FunctagId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FuncenumId(salsa::InternId);
type FuncenumLoc = AssocItemLoc<Funcenum>;
impl_intern!(
    FuncenumId,
    FuncenumLoc,
    intern_funcenum,
    lookup_intern_funcenum
);
impl_serde!(FuncenumId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructId(salsa::InternId);
type StructLoc = AssocItemLoc<Struct>;
impl_intern!(StructId, StructLoc, intern_struct, lookup_intern_struct);
impl_serde!(StructId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructFieldId {
    pub parent: StructId,
    pub local_id: LocalStructFieldId,
}

pub type LocalStructFieldId = Idx<data::StructFieldData>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct BlockId(salsa::InternId);
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct BlockLoc {
    ast_id: AstId,
    file_id: FileId,
}
impl_intern!(BlockId, BlockLoc, intern_block, lookup_intern_block);
impl_serde!(BlockId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalId(salsa::InternId);
type GlobalLoc = ItemTreeId<Variable>;
impl_intern!(GlobalId, GlobalLoc, intern_variable, lookup_intern_variable);
impl_serde!(GlobalId);

/// Defs which can be visible at the global scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileDefId {
    FunctionId(FunctionId),
    MacroId(MacroId),
    GlobalId(GlobalId),
    EnumStructId(EnumStructId),
    MethodmapId(MethodmapId),
    EnumId(EnumId),
    VariantId(VariantId),
    TypedefId(TypedefId),
    TypesetId(TypesetId),
    FunctagId(FunctagId),
    FuncenumId(FuncenumId),
    StructId(StructId),
}

impl_from!(
    FunctionId, MacroId, GlobalId, EnumStructId, EnumId, VariantId, TypedefId, FunctagId, FuncenumId for FileDefId
);

/// Identifies a particular [`ItemTree`].
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct TreeId {
    file: FileId,
    block: Option<BlockId>,
}

impl TreeId {
    pub(crate) fn new(file: FileId, block: Option<BlockId>) -> Self {
        Self { file, block }
    }

    pub(crate) fn item_tree(&self, db: &dyn DefDatabase) -> Arc<ItemTree> {
        match self.block {
            Some(block) => db.block_item_tree(block),
            None => db.file_item_tree(self.file),
        }
    }

    #[allow(unused)]
    pub(crate) fn file_id(self) -> FileId {
        self.file
    }
}

#[derive(Debug, Clone)]
pub struct AssocItemLoc<N: ItemTreeNode> {
    pub container: ItemContainerId,
    pub id: ItemTreeId<N>,
}

impl<N: ItemTreeNode> Copy for AssocItemLoc<N> {}

impl<N: ItemTreeNode> PartialEq for AssocItemLoc<N> {
    fn eq(&self, other: &Self) -> bool {
        self.container == other.container && self.id == other.id
    }
}

impl<N: ItemTreeNode> Eq for AssocItemLoc<N> {}

impl<N: ItemTreeNode> Hash for AssocItemLoc<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.container.hash(state);
        self.id.hash(state);
    }
}

#[derive(Debug)]
pub struct ItemTreeId<N: ItemTreeNode> {
    tree: TreeId,
    pub value: Idx<N>,
}

impl<N: ItemTreeNode> ItemTreeId<N> {
    pub fn new(tree: TreeId, idx: Idx<N>) -> Self {
        Self { tree, value: idx }
    }

    pub fn file_id(self) -> FileId {
        self.tree.file
    }

    pub fn tree_id(self) -> TreeId {
        self.tree
    }

    pub fn item_tree(self, db: &dyn DefDatabase) -> Arc<ItemTree> {
        self.tree.item_tree(db)
    }
}

impl<N: ItemTreeNode> Copy for ItemTreeId<N> {}
impl<N: ItemTreeNode> Clone for ItemTreeId<N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<N: ItemTreeNode> PartialEq for ItemTreeId<N> {
    fn eq(&self, other: &Self) -> bool {
        self.tree == other.tree && self.value == other.value
    }
}

impl<N: ItemTreeNode> Eq for ItemTreeId<N> {}

impl<N: ItemTreeNode> Hash for ItemTreeId<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tree.hash(state);
        self.value.hash(state);
    }
}

/// The defs which have a body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefWithBodyId {
    FunctionId(FunctionId),
    TypedefId(TypedefId),
    FunctagId(FunctagId),
}

impl_from!(FunctionId, TypedefId, FunctagId for DefWithBodyId);

impl DefWithBodyId {
    pub fn file_id(&self, db: &dyn DefDatabase) -> FileId {
        match self {
            DefWithBodyId::FunctionId(it) => it.lookup(db).id.file_id(),
            DefWithBodyId::TypedefId(it) => it.lookup(db).id.file_id(),
            DefWithBodyId::FunctagId(it) => it.lookup(db).id.file_id(),
        }
    }
}

/// `InFile<T>` stores a value of `T` inside a particular file/syntax tree.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct InFile<T> {
    pub file_id: FileId,
    pub value: T,
}

impl<T> InFile<T> {
    pub fn new(file_id: FileId, value: T) -> InFile<T> {
        InFile { file_id, value }
    }

    pub fn with_value<U>(&self, value: U) -> InFile<U> {
        InFile::new(self.file_id, value)
    }

    pub fn map<F: FnOnce(T) -> U, U>(self, f: F) -> InFile<U> {
        InFile::new(self.file_id, f(self.value))
    }

    pub fn as_ref(&self) -> InFile<&T> {
        self.with_value(&self.value)
    }
}

impl<T: Clone> InFile<&T> {
    pub fn cloned(&self) -> InFile<T> {
        self.with_value(self.value.clone())
    }
}

impl<T> InFile<Option<T>> {
    pub fn transpose(self) -> Option<InFile<T>> {
        let value = self.value?;
        Some(InFile::new(self.file_id, value))
    }
}

use base_db::Tree;
use hir_def::{src::HasChildSource, EnumStructId, InFile, Lookup, StructId};

use crate::{
    db::HirDatabase, Enum, EnumStruct, Funcenum, Functag, Function, Global, LocalSource, Macro,
    Methodmap, Property, Struct, Typedef, Typeset, Variant,
};
use hir_def::src::HasSource as _;

pub trait HasSource<'tree> {
    /// Fetches the definition's source node.
    /// TODO: Lookup what is said below.
    /// Using [`crate::Semantics::source`] is preferred when working with [`crate::Semantics`],
    /// as that caches the parsed file in the semantics' cache.
    fn source(
        self,
        db: &dyn HirDatabase,
        tree: &'tree Tree,
    ) -> Option<InFile<tree_sitter::Node<'tree>>>;
}

macro_rules! has_source {
    ($($ty:path),* $(,)?) => {$(
        impl<'tree> HasSource<'tree> for $ty {
            fn source(
                self,
                db: &dyn HirDatabase,
                tree: &'tree Tree,
            ) -> Option<InFile<tree_sitter::Node<'tree>>> {
                Some(self.id.lookup(db.upcast()).source(db.upcast(), tree))
            }
        }
    )*}
}

has_source![
    Function, Macro, Global, EnumStruct, Methodmap, Enum, Variant, Typedef, Typeset, Functag,
    Funcenum, Property, Struct
];

impl<'tree> HasSource<'tree> for LocalSource<'tree> {
    fn source(
        self,
        _db: &dyn HirDatabase,
        _tree: &'tree Tree,
    ) -> Option<InFile<tree_sitter::Node<'tree>>> {
        Some(self.source)
    }
}

impl<'tree> HasSource<'tree> for crate::Field {
    fn source(
        self,
        db: &dyn HirDatabase,
        tree: &'tree Tree,
    ) -> Option<InFile<tree_sitter::Node<'tree>>> {
        let enum_struct_id = EnumStructId::from(self.parent);
        let src = enum_struct_id.child_source(db.upcast());
        Some(InFile {
            file_id: enum_struct_id.lookup(db.upcast()).id.file_id(),
            value: src.value[self.id]
                .to_node(tree)
                .expect("failed to find a node"),
        })
    }
}

impl<'tree> HasSource<'tree> for crate::StructField {
    fn source(
        self,
        db: &dyn HirDatabase,
        tree: &'tree Tree,
    ) -> Option<InFile<tree_sitter::Node<'tree>>> {
        let struct_id = StructId::from(self.parent);
        let src = struct_id.child_source(db.upcast());
        Some(InFile {
            file_id: struct_id.lookup(db.upcast()).id.file_id(),
            value: src.value[self.id]
                .to_node(tree)
                .expect("failed to find a node"),
        })
    }
}

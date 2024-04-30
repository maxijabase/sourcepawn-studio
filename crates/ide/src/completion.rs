mod item;

use base_db::FilePosition;
use hir::{DefResolution, Semantics};
use hir_def::Name;
use ide_db::{RootDatabase, SymbolKind};
pub use item::CompletionItem;
use itertools::Itertools;
use smol_str::ToSmolStr;
use syntax::{utils::lsp_position_to_ts_point, TSKind};
use tree_sitter::{InputEdit, Point};

pub fn completions(
    db: &RootDatabase,
    pos: FilePosition,
    trigger_character: Option<char>,
) -> Option<Vec<CompletionItem>> {
    let sema = &Semantics::new(db);
    let preprocessing_results = sema.preprocess_file(pos.file_id);
    let preprocessed_text = preprocessing_results.preprocessed_text();

    let tree = sema.parse(pos.file_id);

    let point = lsp_position_to_ts_point(&pos.position);
    let token = "foo";
    let edit = create_edit(&preprocessed_text, point, token);
    let new_source_code = [
        &preprocessed_text[..edit.start_byte],
        &token,
        &preprocessed_text[edit.old_end_byte..],
    ]
    .concat();
    // tree.edit(&edit);
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_sourcepawn::language())
        .unwrap();
    let new_tree = parser.parse(new_source_code.as_bytes(), None)?;

    let root_node = new_tree.root_node();
    // get the node before the cursor
    let mut point_off = point;
    point_off.column = point_off.column.saturating_add(1);

    let node = root_node.descendant_for_point_range(point_off, point_off)?;

    let mut container = node.parent()?;
    // If the node does not have a parent we are at the root, nothing to resolve.
    while !matches!(
        TSKind::from(container),
        TSKind::function_definition
            | TSKind::enum_struct_method
            | TSKind::r#enum
            | TSKind::methodmap_native
            | TSKind::methodmap_native_constructor
            | TSKind::methodmap_native_destructor
            | TSKind::methodmap_method
            | TSKind::methodmap_method_constructor
            | TSKind::methodmap_method_destructor
            | TSKind::methodmap_property_getter
            | TSKind::methodmap_property_setter
            | TSKind::methodmap_property_native
            | TSKind::methodmap_property_method
            | TSKind::typedef
            | TSKind::source_file
    ) {
        if let Some(candidate) = container.parent() {
            container = candidate;
        } else {
            break;
        }
    }

    let defs = match TSKind::from(container) {
        TSKind::function_definition => {
            let name = container.child_by_field_name("name")?;
            let name = tree
                .root_node()
                .descendant_for_byte_range(name.start_byte(), name.end_byte())?;
            let container = name.parent()?;
            let def = sema.find_def(pos.file_id, &name)?;
            let DefResolution::Function(def) = def else {
                return None;
            };
            let body_node = container.child_by_field_name("body")?;
            sema.defs_in_function_scope(pos.file_id, def.id(), point, body_node)
        }
        TSKind::alias_assignment => todo!(),
        _ => sema.defs_in_scope(pos.file_id),
    };

    let res = defs
        .into_iter()
        .flat_map(|it| {
            let out: (Option<Name>, SymbolKind) = match it {
                DefResolution::Function(it) => (it.name(db).into(), it.kind(db).into()),
                DefResolution::Macro(it) => (it.name(db).into(), SymbolKind::Macro),
                DefResolution::EnumStruct(it) => (it.name(db).into(), SymbolKind::Struct),
                DefResolution::Methodmap(it) => (it.name(db).into(), SymbolKind::Methodmap),
                DefResolution::Property(it) => (it.name(db).into(), SymbolKind::Property),
                DefResolution::Enum(it) => (it.name(db).into(), SymbolKind::Enum),
                DefResolution::Variant(it) => (it.name(db).into(), SymbolKind::Variant),
                DefResolution::Typedef(it) => (it.name(db), SymbolKind::Typedef),
                DefResolution::Typeset(it) => (it.name(db).into(), SymbolKind::Typeset),
                DefResolution::Functag(it) => (it.name(db), SymbolKind::Functag),
                DefResolution::Funcenum(it) => (it.name(db).into(), SymbolKind::Funcenum),
                DefResolution::Field(it) => (it.name(db).into(), SymbolKind::Field),
                DefResolution::Global(it) => (it.name(db).into(), SymbolKind::Global),
                DefResolution::Local(it) => (it.name(db), SymbolKind::Local),
                DefResolution::File(_) => unreachable!(),
            };
            let label = out.0?.to_smolstr();
            Some(CompletionItem {
                label,
                kind: out.1,
                detail: None,
                documentation: None,
                deprecated: false,
                trigger_call_info: false,
            })
        })
        .collect_vec();

    res.into()
}

fn create_edit(source_code: &str, point: Point, token: &str) -> InputEdit {
    let mut byte_offset = 0;
    let mut current_row = 0;
    let mut current_col = 0;

    // Calculate the byte offset for the given row and column
    for ch in source_code.chars() {
        if current_row >= point.row && current_col >= point.column {
            break;
        }

        if ch == '\n' {
            current_row += 1;
            current_col = 0;
        } else {
            current_col += 1;
        }

        byte_offset += ch.len_utf8();
    }

    InputEdit {
        start_byte: byte_offset,
        old_end_byte: byte_offset,
        new_end_byte: byte_offset + token.len(),
        start_position: point,
        old_end_position: point,
        new_end_position: Point::new(point.row, point.column + token.chars().count()),
    }
}

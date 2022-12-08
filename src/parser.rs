use lazy_static::lazy_static;
use tree_sitter::Query;

pub mod enum_parser;
pub mod enum_struct_parser;
pub mod function_parser;
pub mod include_parser;
pub mod variable_parser;

lazy_static! {
    static ref VARIABLE_QUERY: Query = {
        Query::new(tree_sitter_sourcepawn::language(), "[(variable_declaration_statement) @declaration.variable (old_variable_declaration_statement)  @declaration.variable]").unwrap()
    };
}

use std::sync::Arc;

use super::Location;
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, Hover, HoverContents, HoverParams,
    LanguageString, MarkedString, Range, Url,
};

use crate::{providers::hover::description::Description, utils::uri_to_file_name};

#[derive(Debug, Clone)]
/// SPItem representation of a SourcePawn enum struct.
pub struct EnumStructItem {
    /// Name of the enum struct.
    pub name: String,

    /// Range of the name of the enum struct.
    pub range: Range,

    /// Range of the whole enum struct, including its block.
    pub full_range: Range,

    /// Description of the enum struct.
    pub description: Description,

    /// Uri of the file where the enum struct is declared.
    pub uri: Arc<Url>,

    /// References to this enum struct.
    pub references: Vec<Location>,
}

impl EnumStructItem {
    /// Return a [CompletionItem](lsp_types::CompletionItem) from an [EnumStructItem].
    ///
    /// # Arguments
    ///
    /// * `_params` - [CompletionParams](lsp_types::CompletionParams) of the request.
    pub(crate) fn to_completion(&self, _params: &CompletionParams) -> Option<CompletionItem> {
        Some(CompletionItem {
            label: self.name.to_string(),
            kind: Some(CompletionItemKind::STRUCT),
            detail: uri_to_file_name(&self.uri),
            ..Default::default()
        })
    }

    /// Return a [Hover] from an [EnumStructItem].
    ///
    /// # Arguments
    ///
    /// * `_params` - [HoverParams] of the request.
    pub(crate) fn to_hover(&self, _params: &HoverParams) -> Option<Hover> {
        Some(Hover {
            contents: HoverContents::Array(vec![
                self.formatted_text(),
                MarkedString::String(self.description.to_md()),
            ]),
            range: None,
        })
    }

    /// Formatted representation of an [EnumStructItem].
    ///
    /// # Exemple
    ///
    /// `enum struct Action`
    fn formatted_text(&self) -> MarkedString {
        MarkedString::LanguageString(LanguageString {
            language: "sourcepawn".to_string(),
            value: format!("enum struct {}", self.name),
        })
    }
}

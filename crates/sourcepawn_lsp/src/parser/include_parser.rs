use parking_lot::RwLock;
use std::sync::Arc;

use anyhow::Context;
use lsp_types::{Range, Url};
use tree_sitter::Node;

use crate::{
    document::{Document, Token},
    spitem::{include_item::IncludeItem, SPItem},
    store::Store,
    utils::{self, ts_range_to_lsp_range},
};

impl Store {
    pub(crate) fn parse_include(
        &mut self,
        document: &mut Document,
        node: &mut Node,
    ) -> anyhow::Result<()> {
        let path_node = node
            .child_by_field_name("path")
            .context("Include path is empty.")?;
        let path = path_node.utf8_text(document.preprocessed_text.as_bytes())?;
        let range = ts_range_to_lsp_range(&path_node.range());

        // Remove leading and trailing "<" and ">" or ".
        if path.len() < 2 {
            // The include path is empty.
            return Ok(());
        }
        let quoted = path.starts_with('"');
        let mut path = path[1..path.len() - 1].trim().to_string();
        match self.resolve_import(&mut path, &document.uri, quoted) {
            Some(uri) => {
                add_include(document, uri, path, range);
            }
            None => {
                document.missing_includes.insert(path, range);
            }
        }

        Ok(())
    }

    /// Resolve an include from its `#include` directive and the file it was imported in.
    ///
    /// # Arguments
    ///
    /// * `include_directories` - List of directories to look for includes files.
    /// * `include_text` - Text of the include such as `"file.sp"` or `<file>`.
    /// * `documents` - Set of known documents.
    /// * `document_uri` - Uri of the document where the include declaration is parsed from.
    pub(crate) fn resolve_import(
        &mut self,
        include_text: &mut String,
        document_uri: &Arc<Url>,
        quoted: bool,
    ) -> Option<Url> {
        // Add the extension to the file if needed.
        let include_text =
            utils::add_include_extension(include_text, self.environment.amxxpawn_mode);

        if quoted {
            // Search for the relative path.
            let document_path = document_uri.to_file_path().ok()?;
            let parent_path = document_path.parent()?;
            let mut include_file_path = parent_path.join(include_text);
            let mut uri = Url::from_file_path(&include_file_path).ok()?;
            if self.documents.contains_key(&uri) {
                return Some(uri);
            }
            if let Ok(Some(main_path_uri)) = self.environment.options.get_main_path_uri() {
                let main_path = main_path_uri.to_file_path().ok()?;
                let main_path_parent = main_path.parent()?;
                if parent_path != main_path_parent {
                    // Don't look for includes in the include folder if we are not at the root
                    // of the project.
                    return None;
                }
                include_file_path = main_path_parent.join("include").join(include_text);
                log::trace!(
                    "Looking for {:#?} in {:#?}",
                    include_text,
                    include_file_path
                );

                uri = Url::from_file_path(&include_file_path).ok()?;
                if self.documents.contains_key(&uri) {
                    return Some(uri);
                }
                return None;
            }
        }

        // Look for the include in the same directory or the closest include directory.
        let document_path = document_uri.to_file_path().ok()?;
        let document_dirpath = document_path.parent()?;
        let mut include_file_path = document_dirpath.join(include_text);
        log::trace!(
            "Looking for {:#?} in {:#?}",
            include_text,
            include_file_path
        );
        if !include_file_path.exists() {
            log::trace!("{:#?} not found", include_text);
            include_file_path = document_dirpath.join("include").join(include_text);
            log::trace!(
                "Looking for {:#?} in {:#?}",
                include_text,
                include_file_path
            );
        }
        let uri = Url::from_file_path(&include_file_path).ok()?;
        if self.documents.contains_key(&uri) {
            return Some(uri);
        }

        // Look for the includes in the include directories.
        for include_directory in self.environment.options.includes_directories.iter() {
            let path = include_directory.clone().join(include_text);
            let uri = Url::from_file_path(path).ok()?;
            if self.documents.contains_key(&uri) {
                return Some(uri);
            }
        }

        None
    }
}

pub(crate) fn add_include(document: &mut Document, include_uri: Url, path: String, range: Range) {
    document.includes.insert(
        include_uri.clone(),
        Token {
            text: path.clone(),
            range,
        },
    );

    let include_uri = Arc::new(include_uri);

    let include_item = IncludeItem {
        name: path,
        range,
        v_range: document.build_v_range(&range),
        uri: document.uri.clone(),
        include_uri,
    };
    let include_item = Arc::new(RwLock::new(SPItem::Include(include_item)));
    document.sp_items.push(include_item);
}

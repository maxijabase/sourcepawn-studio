pub mod completion;
pub mod definition;
pub mod hover;

use std::sync::Arc;

use lsp_types::Url;

use crate::store::Store;

#[derive(Clone)]
pub struct FeatureRequest<P> {
    pub params: P,
    pub store: Store,
    pub uri: Arc<Url>,
}

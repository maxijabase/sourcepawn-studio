use lsp_types::{CompletionList, CompletionParams};

use crate::{
    providers::completion::{
        context::is_ctr_call, getters::get_ctr_completions, include::get_include_completions,
    },
    spitem::get_all_items,
};

use self::{
    context::{is_callback_completion_request, is_method_call},
    getters::{get_callback_completions, get_method_completions, get_non_method_completions},
    include::is_include_statement,
};

use super::FeatureRequest;

pub(crate) mod context;
mod getters;
mod include;
mod matchtoken;

pub fn provide_completions(request: FeatureRequest<CompletionParams>) -> Option<CompletionList> {
    let document = request.store.get(&request.uri)?;
    let all_items = get_all_items(&request.store, false);
    let position = request.params.text_document_position.position;
    let line = document.line(position.line)?;
    let pre_line: String = line.chars().take(position.character as usize).collect();

    let include_st = is_include_statement(&pre_line);
    if let Some(include_st) = include_st {
        return get_include_completions(request, include_st);
    }
    if is_callback_completion_request(request.params.context.clone()) {
        return get_callback_completions(all_items, position);
    }

    if !is_method_call(&pre_line) {
        if is_ctr_call(&pre_line) {
            return get_ctr_completions(all_items, request.params);
        }
        return get_non_method_completions(all_items, request.params);
    }

    get_method_completions(all_items, &pre_line, position, request)
}

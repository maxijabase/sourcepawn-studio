use std::hash::Hash;

use lsp_types::{Position, Range};
use sourcepawn_lexer::{Delta, Symbol, TokenKind};

/// Wrapper around `Symbol` that does not contain range information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeLessSymbol {
    pub(crate) token_kind: TokenKind,
    text: String,
    pub(crate) delta: Delta,
}

impl Hash for RangeLessSymbol {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.token_kind.hash(state);
        self.text.hash(state);
        self.delta.hash(state);
    }
}

impl From<Symbol> for RangeLessSymbol {
    fn from(symbol: Symbol) -> Self {
        Self {
            token_kind: symbol.token_kind,
            text: symbol.inline_text(),
            delta: symbol.delta,
        }
    }
}

impl RangeLessSymbol {
    pub fn to_symbol(&self, prev_range: Range) -> Symbol {
        let range = Range::new(
            Position::new(prev_range.end.line, prev_range.end.character),
            Position::new(
                prev_range.end.line.saturating_add_signed(self.delta.line),
                prev_range
                    .end
                    .character
                    .saturating_add_signed(self.delta.col),
            ),
        );
        Symbol::new(self.token_kind, Some(&self.text), range, self.delta)
    }
}

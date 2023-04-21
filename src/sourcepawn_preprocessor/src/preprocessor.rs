use fxhash::FxHashMap;
use sourcepawn_lexer::{Literal, Operator, PreprocDir, Range, SourcepawnLexer, Symbol, TokenKind};

use crate::{evaluator::IfCondition, macros::expand_symbol};

#[derive(Debug, Clone)]
pub struct SourcepawnPreprocessor<'a> {
    pub(crate) lexer: SourcepawnLexer<'a>,
    pub(crate) macros: FxHashMap<String, Macro>,
    pub(crate) expansion_stack: Vec<Symbol>,
    current_line: String,
    prev_end: usize,
    conditions_stack: Vec<bool>,
    out: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct Macro {
    pub(crate) args: Option<Vec<i8>>,
    pub(crate) body: Vec<Symbol>,
}

impl<'a> SourcepawnPreprocessor<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            lexer: SourcepawnLexer::new(input),
            current_line: "".to_string(),
            prev_end: 0,
            conditions_stack: vec![],
            out: vec![],
            macros: FxHashMap::default(),
            expansion_stack: vec![],
        }
    }

    pub fn preprocess_input(&mut self) -> String {
        while let Some(symbol) = if !self.expansion_stack.is_empty() {
            self.expansion_stack.pop()
        } else {
            self.lexer.next()
        } {
            if !self.conditions_stack.is_empty() && !*self.conditions_stack.last().unwrap() {
                self.process_negative_condition(&symbol);
                continue;
            }
            match &symbol.token_kind {
                TokenKind::PreprocDir(dir) => self.process_directive(dir, &symbol),
                TokenKind::Newline => {
                    self.push_ws(&symbol);
                    self.push_current_line();
                    self.current_line = "".to_string();
                    self.prev_end = 0;
                }
                TokenKind::Identifier => match self.macros.get(&symbol.text()) {
                    Some(_) => {
                        expand_symbol(
                            &mut self.lexer,
                            &mut self.macros,
                            &symbol,
                            &mut self.expansion_stack,
                        );
                    }
                    None => {
                        self.push_symbol(&symbol);
                    }
                },
                TokenKind::Eof => {
                    self.push_ws(&symbol);
                    self.push_current_line();
                    break;
                }
                _ => self.push_symbol(&symbol),
            }
        }

        self.out.join("\n")
    }

    fn process_directive(&mut self, dir: &PreprocDir, symbol: &Symbol) {
        match dir {
            PreprocDir::MIf => {
                let line_nb = symbol.range.start_line;
                let mut if_condition = IfCondition::new(&self.macros);
                while self.lexer.in_preprocessor() {
                    if let Some(symbol) = self.lexer.next() {
                        if_condition.symbols.push(symbol);
                    } else {
                        break;
                    }
                }
                self.conditions_stack.push(if_condition.evaluate());
                let line_diff = if_condition.symbols.last().unwrap().range.end_line - line_nb;
                for _ in 0..line_diff {
                    self.out.push(String::new());
                }
                self.prev_end = 0;
            }
            PreprocDir::MDefine => {
                self.push_symbol(symbol);
                let mut macro_name = String::new();
                let mut macro_ = Macro {
                    args: None,
                    body: vec![],
                };
                enum State {
                    Start,
                    Args,
                    Body,
                }
                let mut args = vec![-1, 10];
                let mut found_args = false;
                let mut state = State::Start;
                let mut args_idx = 0;
                while self.lexer.in_preprocessor() {
                    if let Some(symbol) = self.lexer.next() {
                        self.push_ws(&symbol);
                        self.prev_end = symbol.range.end_col;
                        if symbol.token_kind != TokenKind::Newline {
                            self.current_line.push_str(&symbol.text());
                        }
                        match state {
                            State::Start => {
                                if macro_name.is_empty()
                                    && TokenKind::Identifier == symbol.token_kind
                                {
                                    macro_name = symbol.text();
                                } else if symbol.delta.col == 0
                                    && symbol.token_kind == TokenKind::LParen
                                {
                                    state = State::Args;
                                } else {
                                    macro_.body.push(symbol);
                                    state = State::Body;
                                }
                            }
                            State::Args => {
                                if symbol.delta.col > 0 {
                                    macro_.body.push(symbol);
                                    state = State::Body;
                                    continue;
                                }
                                match &symbol.token_kind {
                                    TokenKind::RParen => {
                                        state = State::Body;
                                    }
                                    TokenKind::Literal(Literal::IntegerLiteral) => {
                                        found_args = true;
                                        args[symbol.to_int().unwrap() as usize] = args_idx;
                                    }
                                    TokenKind::Comma => {
                                        args_idx += 1;
                                    }
                                    TokenKind::Operator(Operator::Percent) => (),
                                    _ => unimplemented!("Unexpected token in macro args"),
                                }
                            }
                            State::Body => {
                                macro_.body.push(symbol);
                            }
                        }
                    }
                }
                if found_args {
                    macro_.args = Some(args);
                }
                self.push_current_line();
                self.current_line = "".to_string();
                self.prev_end = 0;
                self.macros.insert(macro_name, macro_);
            }
            PreprocDir::MEndif => {
                self.conditions_stack.pop();
            }
            PreprocDir::MInclude => {
                self.push_symbol(symbol);
                let mut delta = 0;
                while self.lexer.in_preprocessor() {
                    if let Some(mut symbol) = self.lexer.next() {
                        if symbol.token_kind == TokenKind::Literal(Literal::StringLiteral) {
                            delta += symbol.range.end_line - symbol.range.start_line;
                            let text = symbol.inline_text();
                            symbol = Symbol::new(
                                symbol.token_kind.clone(),
                                Some(&text),
                                Range {
                                    start_line: symbol.range.start_line,
                                    end_line: symbol.range.start_line,
                                    start_col: symbol.range.start_col,
                                    end_col: text.len(),
                                },
                                symbol.delta,
                            );
                        }
                        self.push_symbol(&symbol);
                    }
                }
                for _ in 0..delta {
                    self.out.push(String::new());
                }
            }
            _ => self.push_symbol(symbol),
        }
    }

    fn process_negative_condition(&mut self, symbol: &Symbol) {
        match &symbol.token_kind {
            TokenKind::PreprocDir(dir) => match dir {
                PreprocDir::MEndif => {
                    self.conditions_stack.pop();
                }
                PreprocDir::MElse => {
                    let last = self.conditions_stack.pop().unwrap();
                    self.conditions_stack.push(!last);
                }
                // TODO: Handle #elseif.
                _ => todo!(),
            },
            TokenKind::Newline => {
                // Keep the newline to keep the line numbers in sync.
                self.push_current_line();
                self.current_line = "".to_string();
                self.prev_end = 0;
            }
            // Skip any token that is not a directive or a newline.
            _ => (),
        }
    }

    fn push_ws(&mut self, symbol: &Symbol) {
        self.current_line
            .push_str(&" ".repeat(symbol.delta.col.unsigned_abs() as usize));
    }

    fn push_current_line(&mut self) {
        self.out.push(self.current_line.clone());
    }

    fn push_symbol(&mut self, symbol: &Symbol) {
        if symbol.token_kind == TokenKind::Eof {
            self.push_current_line();
            return;
        }
        self.push_ws(&symbol);
        self.prev_end = symbol.range.end_col;
        self.current_line.push_str(&symbol.text());
    }
}

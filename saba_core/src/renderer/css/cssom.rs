use core::iter::Peekable;

use alloc::{string::String, vec::Vec};

use super::token::{CssToken, CssTokenizer};

#[derive(Debug, Clone)]
pub struct CssParser {
    tokenizer: Peekable<CssTokenizer>
}

impl CssParser {
    pub fn new(tokenizer: CssTokenizer) -> Self {
        Self { tokenizer: tokenizer.peekable() }
    }
}

pub struct StyleSheet {
    pub rules: Vec<QualifiedRule>,
}

impl StyleSheet {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn set_rules(&mut self, rules: Vec<QualifiedRule>) {
        self.rules = rules;
    }
}

pub struct QualifiedRule {
    pub selector: Selector,
    pub declarations: Vec<Declaration>,
}

impl QualifiedRule {
    pub fn new(selector: Selector, declarations: Vec<Declaration>) -> Self {
        Self { selector, declarations }
    }

    pub fn set_selector(&mut self, selector: Selector) {
        self.selector = selector;
    }

    pub fn set_declarations(&mut self, declarations: Vec<Declaration>) {
        self.declarations = declarations;
    }
}

pub enum Selector {
    TypeSelector(String),
    ClassSelector(String),
    IdSelector(String),
    UnknownSelector,
}

pub struct Declaration {
    pub property: String,
    pub value: CssToken,
}

impl Declaration {
    pub fn new() -> Self {
        Self { property: String::new(), value: CssToken::Ident(String::new()) }
    }

    pub fn set_property(&mut self, property: String) {
        self.property = property;
    }

    pub fn set_value(&mut self, value: CssToken) {
        self.value = value;
    }
}

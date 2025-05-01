use core::iter::Peekable;

use alloc::{string::{String, ToString}, vec::Vec};

use super::token::{CssToken, CssTokenizer};

#[derive(Debug, Clone)]
pub struct CssParser {
    tokenizer: Peekable<CssTokenizer>
}

impl CssParser {
    pub fn new(tokenizer: CssTokenizer) -> Self {
        Self { tokenizer: tokenizer.peekable() }
    }

    pub fn parse_stylesheet(&mut self) -> StyleSheet {
        let mut sheet = StyleSheet::new();
        sheet.set_rules(self.consume_list_of_rules());
        sheet
    }

    fn consume_list_of_rules(&mut self) -> Vec<QualifiedRule> {
        let mut rules = Vec::new();

        loop {
            let token = match self.tokenizer.peek() {
                Some(t) => t,
                None => return rules,
            };

            match token {
                CssToken::AtKeyword(_keyword) => {
                    let _rule = self.consume_qualified_rule();
                }
                _ => {
                    let rule = self.consume_qualified_rule();
                    match rule {
                        Some(r) => rules.push(r),
                        None => return rules,
                    }
                }
            }
        }
    }

    fn consume_qualified_rule(&mut self) -> Option<QualifiedRule> {
        let mut rule = QualifiedRule::new();

        loop {
            let token = match self.tokenizer.peek() {
                Some(t) => t,
                None => return None,
            };

            match token {
                CssToken::OpenCurly => {
                    assert_eq!(self.tokenizer.next(), Some(CssToken::OpenCurly));
                    rule.set_declarations(self.consume_list_of_declarations());
                    return Some(rule);
                }
                _ => {
                    rule.set_selector(self.consume_selector());
                }
            }
        }
    }

    fn consume_selector(&mut self) -> Selector {
        let token = match self.tokenizer.next() {
            Some(t) => t,
            None => panic!("should have a token but got None"),
        };

        match token {
            CssToken::HashToken(value) => Selector::IdSelector(value[1..].to_string()),
            CssToken::Delim(delim) => {
                if delim == '.' {
                    return Selector::ClassSelector(self.consume_ident());
                }
                panic!("Parse error: {:?} is an unexpected token.", token);
            },
            CssToken::Ident(ident) => {
                // a:hover のようなセレクタをタイプセレクタとして解釈する
                if self.tokenizer.peek() == Some(&CssToken::Colon) {
                    while self.tokenizer.peek() != Some(&CssToken::OpenCurly) {
                        self.tokenizer.next();
                    }
                }

                Selector::TypeSelector(ident.to_string())
            },
            CssToken::AtKeyword(_keyword) => {
                // @ ではじまるルールはサポートしないので、宣言ブロックの開始直前まで読み捨てる
                while self.tokenizer.peek() != Some(&CssToken::OpenCurly) {
                    self.tokenizer.next();
                }

                Selector::UnknownSelector
            },
            _ => {
                self.tokenizer.next();
                Selector::UnknownSelector
            }
        }
    }

    fn consume_list_of_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();

        loop {
            let token = match self.tokenizer.peek() {
                Some(t) => t,
                None => return declarations
            };

            match token {
                CssToken::CloseCurly => {
                    assert_eq!(self.tokenizer.next(), Some(CssToken::CloseCurly));
                    return declarations;
                }
                CssToken::SemiColon => {
                    assert_eq!(self.tokenizer.next(), Some(CssToken::SemiColon));
                }
                CssToken::Ident(ref _ident) => {
                    if let Some(declaration) = self.consume_declaration() {
                        declarations.push(declaration);
                    }
                }
                _ => {
                    self.tokenizer.next();
                }
            }
        }
    }

    fn consume_declaration(&mut self) -> Option<Declaration> {
        if self.tokenizer.peek().is_none() {
            return None;
        }

        let mut declaration = Declaration::new();
        declaration.set_property(self.consume_ident());

        match self.tokenizer.next() {
            Some(token) => match token {
                CssToken::Colon => {}, // declaration は property : value の形をしているはずなのでコロン以外が来たらおかしい
                _ => return None,
            },
            None => return None,
        }

        declaration.set_value(self.consume_component_value());

        Some(declaration)
    }

    fn consume_ident(&mut self) -> String {
        let token = match self.tokenizer.next() {
            Some(t) => t,
            None => panic!("should have a token but got None")
        };

        match token {
            CssToken::Ident(i) => i,
            _ => panic!("Parse error: {:?} is an unexpected token.", token)
        }
    }

    fn consume_component_value(&mut self) -> CssToken {
        self.tokenizer.next().expect("should have a token in consume_component_value")
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
    pub fn new() -> Self {
        Self { selector: Selector::TypeSelector("".to_string()), declarations: Vec::new() }
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

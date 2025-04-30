use core::iter::Peekable;

use super::token::CssTokenizer;

#[derive(Debug, Clone)]
pub struct CssParser {
    tokenizer: Peekable<CssTokenizer>
}

impl CssParser {
    pub fn new(tokenizer: CssTokenizer) -> Self {
        Self { tokenizer: tokenizer.peekable() }
    }
}

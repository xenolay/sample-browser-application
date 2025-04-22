use alloc::vec::Vec;

use crate::renderer::dom::node::{Node, Window};

use super::token::HtmlTokenizer;

#[derive(Debug, Clone)]
pub struct HtmlParser {
    window: Window,
    current_mode: InsertionMode,
    original_mode: InsertionMode, // https://html.spec.whatwg.org/multipage/parsing.html#original-insertion-mode
    stack_of_open_elements: Vec<Node>, // https://html.spec.whatwg.org/multipage/parsing.html#the-stack-of-open-elements
    tokenizer: HtmlTokenizer,
}

#[derive(Debug, Clone)]
pub enum InsertionMode {
    // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inhtml のうち9種類のみ実装する
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    Text,
    AfterBody,
    AfterAfterBody,
}

impl HtmlParser {
    pub fn new(tokenizer: HtmlTokenizer) -> Self {
        Self { window: Window::new(), current_mode: InsertionMode::Initial, original_mode: InsertionMode::Initial, stack_of_open_elements: Vec::new(), tokenizer }
    }

    pub fn construct_tree(&mut self) -> Window {
        let token = self.tokenizer.next();
        while token.is_some() {
            match self.current_mode {
                InsertionMode::Initial => todo!(),
                InsertionMode::BeforeHtml => todo!(),
                InsertionMode::BeforeHead => todo!(),
                InsertionMode::InHead => todo!(),
                InsertionMode::AfterHead => todo!(),
                InsertionMode::InBody => todo!(),
                InsertionMode::Text => todo!(),
                InsertionMode::AfterBody => todo!(),
                InsertionMode::AfterAfterBody => todo!(),
            }    
        }
        self.window.clone()
    }
}

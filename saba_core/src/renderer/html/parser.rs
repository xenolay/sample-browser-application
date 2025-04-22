use alloc::vec::Vec;

use crate::renderer::dom::node::{Node, Window};

use super::{html_tag_attribute::{AttributeField, HtmlTagAttribute}, token::{HtmlToken, HtmlTokenizer}};

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

    // 本当は token の reprocess が必要なことがあるのだが、色々と実装を妥協している
    pub fn construct_tree(&mut self) -> Window {
        let mut token = self.tokenizer.next();
        while token.is_some() {
            match self.current_mode {
                InsertionMode::Initial => {
                    // https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode
                    // 本当は DOCTYPE token や comment token の処理が必要だが、これらの token を実装していないため文字 token 扱いになる。文字 token のことは単に無視する
                    if let Some(HtmlToken::Char(_)) = token {
                        token = self.tokenizer.next();
                        continue;
                    }

                    // 本のとおり実装するとこうなるが、endTag token や EoF Token は before html で reprocess するはず……？
                    self.current_mode = InsertionMode::BeforeHtml;
                    continue;
                },
                InsertionMode::BeforeHtml => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            if c == ' ' || c == '\n' {
                                token = self.tokenizer.next();
                                continue;
                            }
                        },
                        Some(HtmlToken::StartTag { ref tag, self_closing, ref attributes }) => {
                            if tag == "html" {
                                self.insert_element(tag, attributes.to_vec());
                                self.current_mode = InsertionMode::BeforeHead;
                                token = self.tokenizer.next();
                                continue;
                            }
                        },
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        },
                        _ => {}
                    }
                    self.insert_element("html", Vec::new());
                    self.current_mode = InsertionMode::BeforeHead;
                    continue;
                },
                InsertionMode::BeforeHead => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            if c == ' ' || c == '\n' {
                                token = self.tokenizer.next();
                                continue;
                            }
                        },
                        Some(HtmlToken::StartTag { ref tag, self_closing, ref attributes }) => {
                            if tag == "head" {
                                self.insert_element(tag, attributes.to_vec());
                                self.current_mode = InsertionMode::InHead;
                                token = self.tokenizer.next();
                                continue;
                            }
                        },
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        },
                        _ => {}
                    }
                    self.insert_element("head", Vec::new());
                    self.current_mode = InsertionMode::InHead;
                    continue;                    
                },
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

    fn insert_element(&self, tag: &str, attributes: Vec<HtmlTagAttribute>) {
        todo!();
    }
}

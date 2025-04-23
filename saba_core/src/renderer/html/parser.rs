use core::str::FromStr;

use alloc::{rc::Rc, vec::Vec};

use crate::renderer::dom::node::{Element, ElementKind, Node, NodeKind, Window};

use super::{html_tag_attribute::HtmlTagAttribute, token::{HtmlToken, HtmlTokenizer}};

#[derive(Debug, Clone)]
pub struct HtmlParser {
    window: Window, // 本だと Rc している
    current_mode: InsertionMode,
    original_mode: InsertionMode, // https://html.spec.whatwg.org/multipage/parsing.html#original-insertion-mode
    stack_of_open_elements: Vec<Rc<Node>>, // https://html.spec.whatwg.org/multipage/parsing.html#the-stack-of-open-elements
    tokenizer: HtmlTokenizer,
}

#[derive(Debug, Clone, Copy)]
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
                InsertionMode::InHead => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            if c == ' ' || c == '\n' {
                                // 本だとここ誤植してそう
                                token = self.tokenizer.next();
                                continue;
                            }
                        },
                        Some(HtmlToken::StartTag { ref tag, self_closing, ref attributes }) => {
                            if tag == "style" || tag == "script" {
                                self.insert_element(tag, attributes.to_vec());
                                self.original_mode = self.current_mode;
                                self.current_mode = InsertionMode::Text;
                                token = self.tokenizer.next();
                                continue;
                            }

                            // ここがないと head が省略されている html document で無限ループが出るらしい
                            if tag == "body" {
                                self.pop_until(ElementKind::Head);
                                self.current_mode = InsertionMode::AfterHead;
                                continue;
                            }
                            if let Ok(_element_kind) = ElementKind::from_str(tag) {
                                self.pop_until(ElementKind::Head);
                                self.current_mode = InsertionMode::AfterHead;
                                continue;
                            }
                        },
                        Some(HtmlToken::EndTag { ref tag }) => {
                            if tag == "head" {
                                self.current_mode = InsertionMode::AfterHead;
                                token = self.tokenizer.next();
                                self.pop_until(ElementKind::Head);
                                continue;
                            }

                        },
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                    }
                    token = self.tokenizer.next();
                    continue;                    
                },
                InsertionMode::AfterHead => {
                    match token {
                        Some(HtmlToken::Char(c)) => {
                            if c == ' ' || c == '\n' {
                                token = self.tokenizer.next();
                                continue;
                            }
                        },
                        Some(HtmlToken::StartTag { ref tag, self_closing, ref attributes }) => {
                            if tag == "body" {
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
                    self.insert_element("body", Vec::new());
                    self.current_mode = InsertionMode::InHead;
                    continue;
                },
                InsertionMode::InBody => {
                    match token {
                        Some(HtmlToken::EndTag { ref tag }) => {
                            match tag.as_str() {
                                "body" => {
                                    self.current_mode = InsertionMode::AfterBody;
                                    token = self.tokenizer.next();
                                    if !self.contain_in_stack(ElementKind::Body) {
                                        // [] 13.2.6.4.1 The "initial" insertion mode | HTML Standard
                                        // https://html.spec.whatwg.org/multipage/parsing.html#the-initial-insertion-mode
                                        // ----- Cited From Reference -----
                                        // If the stack of open elements does not have a body element in scope, this is a parse error; ignore the token.
                                        // --------------------------------
                                        continue;
                                    }
                                    self.pop_until(ElementKind::Body);
                                    continue;
                                }
                                "html" => {
                                    if self.pop_current_node(ElementKind::Body) {
                                        self.current_mode = InsertionMode::AfterBody;
                                        assert!(self.pop_current_node(ElementKind::Html))
                                    } else {
                                        token = self.tokenizer.next();
                                    }
                                    continue;
                                }
                                _ => {
                                    token = self.tokenizer.next();
                                }
                            }
                        }
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        _ => {}
                    }
                },
                InsertionMode::Text => {
                    match token {
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        }
                        Some(HtmlToken::EndTag { ref tag }) => {
                            if tag == "style" {
                                self.pop_until(ElementKind::Style);
                                self.current_mode = self.original_mode;
                                token = self.tokenizer.next();
                                continue;
                            }
                            if tag == "script" {
                                self.pop_until(ElementKind::Script);
                                self.current_mode = self.original_mode;
                                token = self.tokenizer.next();
                                continue;
                            }
                        }
                        Some(HtmlToken::Char(c)) => {
                            self.insert_char(c);
                            token = self.tokenizer.next();
                            continue;
                        }
                        _ => {}
                    }

                    self.current_mode = self.original_mode;
                },
                InsertionMode::AfterBody => {
                    match token {
                        Some(HtmlToken::Char(_)) => {
                            token = self.tokenizer.next();
                            continue;
                        },
                        Some(HtmlToken::EndTag { ref tag}) => {
                            if tag == "html" {
                                self.current_mode = InsertionMode::AfterAfterBody;
                                token = self.tokenizer.next();
                                continue;
                            }
                        },
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        },
                        _ => {}
                    }
                    self.current_mode = InsertionMode::InBody;
                    continue;
                },
                InsertionMode::AfterAfterBody => {
                    match token {
                        Some(HtmlToken::Char(_)) => {
                            token = self.tokenizer.next();
                            continue;
                        },
                        Some(HtmlToken::Eof) | None => {
                            return self.window.clone();
                        },
                        _ => {}
                    }
                    self.current_mode = InsertionMode::InBody;
                    continue;
                },
            }    
        }
        self.window.clone()
    }

    fn create_element(&self, tag: &str, attributes: Vec<HtmlTagAttribute>) -> Node {
        Node::new(NodeKind::Element(Element::new(tag, attributes)))
    }

    fn insert_element(&self, tag: &str, attributes: Vec<HtmlTagAttribute>) {
        let window = &self.window;
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n.clone(),
            None => window.document(),
        };

        let node = Rc::new(self.create_element(tag, attributes));

        if current.first_child().is_some() {
            // なんかもうちょいどうにかならんかな。last_sibling が some であることはこのブロックにおける不変条件なので、それが明確になるようにしたい
            let mut last_sibling = current.first_child();
            loop {
                last_sibling = match last_sibling {
                    Some(ref node) => {
                        if node.next_sibling().is_some() {
                            node.next_sibling()
                        } else {
                            break;
                        }
                    }
                    None => unimplemented!("ha?")
                }
            }

            // ここで mutate したいので Node の Fields は RefCell で包まないといけない。なるほど～
            // Rc::get_mut するのは、一般には Rc での参照が1つとは限らないので上手くいかない。
            // let a = Rc::get_mut(&mut last_sibling.unwrap()).unwrap().set_next_sibling(Some(Rc::clone(&node)));
            // last_sibling.unwrap().set_next_sibling(Some(Rc::clone(&node)));
        }
    }

    fn pop_until(&self, kind: ElementKind) {
        todo!();
    }

    fn contain_in_stack(&self, kind: ElementKind) -> bool {
        todo!();
    }

    fn pop_current_node(&self, kind: ElementKind) -> bool {
        todo!();
    }

    fn insert_char(&self, c: char) {
        todo!();
    }
}

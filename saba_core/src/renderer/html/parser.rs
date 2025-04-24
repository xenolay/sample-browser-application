use core::{cell::RefCell, str::FromStr};

use alloc::{rc::Rc, string::ToString, vec::Vec};

use crate::renderer::dom::node::{Element, ElementKind, Node, NodeKind, Window};

use super::{html_tag_attribute::HtmlTagAttribute, token::{HtmlToken, HtmlTokenizer}};

#[derive(Debug, Clone)]
pub struct HtmlParser {
    window: Rc<RefCell<Window>>, // 本だと Rc している。少なくとも単体テスト時には Rc されてないと困る。
    current_mode: InsertionMode,
    original_mode: InsertionMode, // https://html.spec.whatwg.org/multipage/parsing.html#original-insertion-mode
    stack_of_open_elements: Vec<Rc<RefCell<Node>>>, // https://html.spec.whatwg.org/multipage/parsing.html#the-stack-of-open-elements
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
        Self { window: Rc::new(RefCell::new(Window::new())), current_mode: InsertionMode::Initial, original_mode: InsertionMode::Initial, stack_of_open_elements: Vec::new(), tokenizer }
    }

    // 本当は token の reprocess が必要なことがあるのだが、色々と実装を妥協している
    pub fn construct_tree(&mut self) -> Rc<RefCell<Window>> {
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

    fn insert_element(&mut self, tag: &str, attributes: Vec<HtmlTagAttribute>) {
        let window = &self.window;
        let mut current = match self.stack_of_open_elements.last() {
            Some(n) => n.clone(),
            None => window.borrow().document(),
        };

        let node = Rc::new(RefCell::new(self.create_element(tag, attributes)));

        if current.borrow().first_child().is_some() {
            // なんかもうちょいどうにかならんかな。last_sibling が some であることはこのブロックにおける不変条件なので、それが明確になるようにしたい
            let mut last_sibling = current.borrow().first_child();
            loop {
                last_sibling = match last_sibling {
                    Some(ref node) => {
                        if node.borrow().next_sibling().is_some() {
                            node.borrow().next_sibling()
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
            last_sibling.as_ref().unwrap().borrow_mut().set_next_sibling(Some(Rc::clone(&node)));

            node.borrow_mut().set_previous_sibling(Rc::downgrade(&last_sibling.unwrap()));
        } else {
            current.borrow_mut().set_first_child(Some(Rc::clone(&node)));
        }

        current.borrow_mut().set_last_child(Rc::downgrade(&node));
        node.borrow_mut().set_parent(Rc::downgrade(&current));

        self.stack_of_open_elements.push(node);
    }

    fn pop_until(&mut self, kind: ElementKind) {
        loop {
            let current = match self.stack_of_open_elements.pop() {
                Some(n) => n,
                None => return
            };

            if current.borrow().get_element_kind() == Some(kind) {
                return;
            }
        }
    }

    fn contain_in_stack(&self, kind: ElementKind) -> bool {
        // find で書けるから書いたけど別にわかりやすくなった気はしないな
        if let Some(_) = self.stack_of_open_elements.iter().find(|x| x.borrow().get_element_kind() == Some(kind)) {
            true
        } else {
            false
        }
    }

    fn pop_current_node(&mut self, kind: ElementKind) -> bool {
        let current = match self.stack_of_open_elements.last() {
            Some(n) => n,
            None => return false,
        };

        if current.borrow().get_element_kind() == Some(kind) {
            self.stack_of_open_elements.pop();
            return true;
        }

        false
    }

    fn create_char(&self, c: char) -> Node {
        Node::new(NodeKind::Text(c.to_string()))
    }

    fn insert_char(&mut self, c: char) {
        let current = match self.stack_of_open_elements.last() {
            Some(n) => Rc::clone(n),
            None => return, // 本当はこの枝に入る時点で何かがおかしいのでいい感じに弾きたいんだよな。しかしサボってエラーを握りつぶすことにする
        };

        // 現在参照しているノードが Text ならそいつに push すればいいのでそうする
        if let NodeKind::Text(mut s) = current.borrow_mut().node_kind() {
            s.push(c);
            return;
        };

        if c == '\n' || c == ' ' {
            return;
        }

        let node = Rc::new(RefCell::new(self.create_char(c)));

        if current.borrow().first_child().is_some() {
            // 本だとこのパートだけ last_sibling のサーチをサボってるんだけど、やったほうがいいのでは？？？？
            // なんかもうちょいどうにかならんかな（2）。last_sibling が some であることはこのブロックにおける不変条件なので、それが明確になるようにしたい
            let mut last_sibling = current.borrow().first_child();
            loop {
                last_sibling = match last_sibling {
                    Some(ref node) => {
                        if node.borrow().next_sibling().is_some() {
                            node.borrow().next_sibling()
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
            last_sibling.as_ref().unwrap().borrow_mut().set_next_sibling(Some(Rc::clone(&node)));

            node.borrow_mut().set_previous_sibling(Rc::downgrade(&last_sibling.unwrap()));
        } else {
            current.borrow_mut().set_first_child(Some(Rc::clone(&node)));
        }

        current.borrow_mut().set_last_child(Rc::downgrade(&node));
        node.borrow_mut().set_parent(Rc::downgrade(&current));

        self.stack_of_open_elements.push(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{alloc::string::ToString, renderer::html::html_tag_attribute::AttributeField};
    use alloc::vec;

    #[test]
    fn test_empty() {
        let html = "".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let expected = Rc::new(RefCell::new(Node::new(NodeKind::Document)));

        assert_eq!(expected, window.borrow().document());
    }

    #[test]
    fn test_body() {
        let html = "<html><head></head><body></body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let document = window.borrow().document();
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Document))),
            document
        );

        let html = document
            .borrow()
            .first_child()
            .expect("failed to get a first child of document");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "html",
                Vec::new()
            ))))),
            html
        );

        let head = html
            .borrow()
            .first_child()
            .expect("failed to get a first child of html");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "head",
                Vec::new()
            ))))),
            head
        );

        let body = head
            .borrow()
            .next_sibling()
            .expect("failed to get a next sibling of head");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "body",
                Vec::new()
            ))))),
            body
        );
    }

    #[test]
    fn test_text() {
        let html = "<html><head></head><body>text</body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let document = window.borrow().document();
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Document))),
            document
        );

        let html = document
            .borrow()
            .first_child()
            .expect("failed to get a first child of document");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "html",
                Vec::new()
            ))))),
            html
        );

        let body = html
            .borrow()
            .first_child()
            .expect("failed to get a first child of document")
            .borrow()
            .next_sibling()
            .expect("failed to get a next sibling of head");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "body",
                Vec::new()
            ))))),
            body
        );

        let text = body
            .borrow()
            .first_child()
            .expect("failed to get a first child of document");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Text("text".to_string())))),
            text
        );
    }

    #[test]
    fn test_multiple_nodes() {
        let html = "<html><head></head><body><p><a foo=bar>text</a></p></body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let document = window.borrow().document();

        let body = document
            .borrow()
            .first_child()
            .expect("failed to get a first child of document")
            .borrow()
            .first_child()
            .expect("failed to get a first child of document")
            .borrow()
            .next_sibling()
            .expect("failed to get a next sibling of head");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "body",
                Vec::new()
            ))))),
            body
        );

        let p = body
            .borrow()
            .first_child()
            .expect("failed to get a first child of body");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "p",
                Vec::new()
            ))))),
            p
        );

        let mut attr = HtmlTagAttribute::new();
        attr.add_char('f', AttributeField::Name);
        attr.add_char('o', AttributeField::Name);
        attr.add_char('o', AttributeField::Name);
        attr.add_char('b', AttributeField::Value);
        attr.add_char('a', AttributeField::Value);
        attr.add_char('r', AttributeField::Value);
        let a = p
            .borrow()
            .first_child()
            .expect("failed to get a first child of p");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Element(Element::new(
                "a",
                vec![attr]
            ))))),
            a
        );

        let text = a
            .borrow()
            .first_child()
            .expect("failed to get a first child of a");
        assert_eq!(
            Rc::new(RefCell::new(Node::new(NodeKind::Text("text".to_string())))),
            text
        );
    }
}

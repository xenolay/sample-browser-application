use alloc::{string::String, vec::Vec};
use crate::renderer::html::html_tag_attribute::{AttributeField, HtmlTagAttribute};

// [] 13.2.5 Tokenization | HTML Standard
// https://html.spec.whatwg.org/multipage/parsing.html#tokenization
// ----- Cited From Reference -----
// The output of the tokenization step is a series of zero or more of the following tokens: DOCTYPE, start tag, end tag, comment, character, end-of-file. DOCTYPE tokens have a name, a public identifier, a system identifier, and a force-quirks flag. When a DOCTYPE token is created, its name, public identifier, and system identifier must be marked as missing (which is a distinct state from the empty string), and the force-quirks flag must be set to off (its other state is on). Start and end tag tokens have a tag name, a self-closing flag, and a list of attributes, each of which has a name and a value. When a start or end tag token is created, its self-closing flag must be unset (its other state is that it be set), and its attributes list must be empty. Comment and character tokens have data.
// --------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlToken {
    // ...↑のように書いてはあるが、このブラウザでは DOCTYPE token と comment token は実装しない。
    StartTag {
        tag: String,
        self_closing: bool,
        attributes: Vec<HtmlTagAttribute>,
    },

    EndTag {
        tag: String,
    },

    Char(char),

    Eof,
}

// [] 13.2.5 Tokenization | HTML Standard
// https://html.spec.whatwg.org/multipage/parsing.html#tokenization
// ↑ で規定のある State の一部を実装する。本当は80種類あるのだが、全部実装すると日が暮れる……
#[derive(Debug, Clone)]
pub enum TokenizerState {
    Data, // https://html.spec.whatwg.org/multipage/parsing.html#data-state
    TagOpen, // https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state
    EndTagOpen, // https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
    TagName, // https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
    BeforeAttributeName, // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-name-state
    AttributeName, // https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
    AfterAttributeName, // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
    BeforeAttributeValue, // https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
    AttributeValueDoubleQuoted, // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
    AttributeValueSingleQuoted, // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(single-quoted)-state
    AttributeValueUnQuoted, // https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
    AfterAttributeValueQuoted, // https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-(quoted)-state
    SelfClosingStartTag, // https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state
    ScriptData, // https://html.spec.whatwg.org/multipage/parsing.html#script-data-state
    ScriptDataLessThanSign, // https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
    ScriptDataEndTagOpen, // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-open-state
    ScriptDataEndTagName, // https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-name-state
    TemporaryBuffer, // whatwg 上で規定はないが、実装を簡単にするために実装する
}

#[derive(Debug, Clone)]
pub struct HtmlTokenizer {
    state: TokenizerState,
    pos: usize,
    reconsume: bool,
    latest_token: Option<HtmlToken>,
    input: Vec<char>,
    buf: String,
}

impl HtmlTokenizer {
    pub fn new(html: String) -> Self {
        Self {
            state: TokenizerState::Data,
            pos: 0,
            reconsume: false,
            latest_token: None,
            input: html.chars().collect(),
            buf: String::new(),
        }
    }

    fn is_eof(&self) -> bool {
        self.pos > self.input.len()
    }

    fn consume_next_character(&mut self) -> char {
        let c = if self.reconsume {
            // [] 13.2.5.4 Script data state | HTML Standard
            // https://html.spec.whatwg.org/multipage/parsing.html#script-data-state
            // ----- Cited From Reference -----
            // When a state says to reconsume a matched character in a specified state, that means to switch to that state, but when it attempts to consume the next input character, provide it with the current input character instead.
            // --------------------------------
            // [] current input character | HTML Standard
            // https://html.spec.whatwg.org/multipage/parsing.html#current-input-character
            // ----- Cited From Reference -----
            //  The current input character is the last character to have been consumed.
            // --------------------------------
            self.reconsume = false;
            self.input[self.pos - 1]
        } else {
            self.pos += 1;
            self.input[self.pos - 1]
        };
        c
    }

    fn create_start_tag(&mut self) {
        self.latest_token = Some(
            HtmlToken::StartTag { tag: String::new(), self_closing: false, attributes: Vec::new() }
        )
    }

    fn create_end_tag(&mut self) {
        self.latest_token = Some(
            HtmlToken::EndTag { tag: String::new() }
        )
    }

    fn append_tag_name(&mut self, c: char) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag { tag, self_closing: _, attributes: _ } | HtmlToken::EndTag { tag } => tag.push(c),
                _ => panic!("latest_token must be either StartTag or EndTag"),
            }
        }
    }

    fn emit_latest_token(&mut self) -> Option<HtmlToken> {
        assert!(self.latest_token.is_some());

        let t = self.latest_token.as_ref().cloned();
        self.latest_token = None;
        assert!(self.latest_token.is_none());

        t
    }

    fn start_new_attribute(&mut self) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag { tag: _, self_closing: _, attributes } => attributes.push(HtmlTagAttribute::new()),
                _ => panic!("latest_token must be StartTag"),
            }
        }
    }

    fn append_character_to_attribute(&mut self, c: char, field: AttributeField) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag { tag: _, self_closing: _, attributes } => {
                    let len = attributes.len();
                    assert!(len > 0);

                    attributes[len - 1].add_char(c, field)
                },
                _ => panic!("latest_token should be StartTag"),
            }
        }
    }

    fn set_self_closing_flag(&mut self) {
        assert!(self.latest_token.is_some());

        if let Some(t) = self.latest_token.as_mut() {
            match t {
                HtmlToken::StartTag { tag: _, self_closing, attributes: _ } => *self_closing = true,
                _ => panic!("latest_token must be StartTag")
            }
        }
    }
}

impl Iterator for HtmlTokenizer {
    type Item = HtmlToken;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() { // ここは is_eof ではダメ？
            return None
        }

        loop {
            let c = self.consume_next_character();
            match self.state {
                TokenizerState::Data => {
                    if c == '<' {
                        self.state = TokenizerState::TagOpen;
                        continue
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    return Some(HtmlToken::Char(c));
                },
                TokenizerState::TagOpen => {
                    if c == '/' {
                        self.state = TokenizerState::EndTagOpen;
                        continue;
                    }

                    if c.is_ascii_alphabetic() {
                        self.reconsume = true;
                        self.state = TokenizerState::TagName;
                        self.create_start_tag();
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.reconsume = true;
                    self.state = TokenizerState::Data
                },
                TokenizerState::EndTagOpen => {
                    if self.is_eof() {
                        // 本当はパースエラーにする必要がある
                        return Some(HtmlToken::Eof);
                    }

                    if c.is_ascii_alphabetic() {
                        self.reconsume = true;
                        self.state = TokenizerState::TagName;
                        self.create_end_tag();
                    }

                    // 本当は > とかが来たらパースエラーにする必要があるのだが、本に沿っていったんこのままにする
                },
                TokenizerState::TagName => {
                    if c == ' ' { // 本当は tab, LF, FF もこの枝
                        self.state = TokenizerState::BeforeAttributeName;
                        continue;
                    }

                    if c == '/' {
                        self.state = TokenizerState::SelfClosingStartTag;
                        continue;
                    }

                    if c == '>' {
                        self.state = TokenizerState::Data;
                        return self.emit_latest_token();
                    }

                    if c.is_ascii_uppercase() {
                        self.append_tag_name(c.to_ascii_lowercase());
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    // 本当は NULL 文字は U+FFFD に変換するがめんどいのでそのまま
                    self.append_tag_name(c);
                },
                TokenizerState::BeforeAttributeName => {
                    if c == '/' || c == '>' || self.is_eof() {
                        self.reconsume = true;
                        self.state = TokenizerState::AfterAttributeName;
                        continue;
                    }

                    self.reconsume = true;
                    self.state = TokenizerState::AttributeName;
                    self.start_new_attribute();

                    // 本当は = の場合は別の処理がある  とか space を無視するとか色々ある
                },
                TokenizerState::AttributeName => {
                    if c == ' ' || c == '/' || c == '>' || self.is_eof() {
                        self.reconsume = true;
                        self.state = TokenizerState::AfterAttributeName;
                        continue;
                    }

                    if c == '=' {
                        self.state = TokenizerState::BeforeAttributeValue;
                        continue;
                    }

                    if c.is_ascii_uppercase() {
                        self.append_character_to_attribute(c.to_ascii_lowercase(), AttributeField::Name);
                        continue;
                    }

                    self.append_character_to_attribute(c, AttributeField::Name);
                },
                TokenizerState::AfterAttributeName => {
                    if c == ' ' {
                        continue;
                    }

                    if c == '/' {
                        self.state = TokenizerState::SelfClosingStartTag;
                        continue;
                    }

                    if c == '=' {
                        self.state = TokenizerState::BeforeAttributeValue;
                        continue;
                    }

                    if c == '>' {
                        self.state = TokenizerState::Data;
                        return self.emit_latest_token();
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.reconsume = true;
                    self.state = TokenizerState::AttributeName;
                    self.start_new_attribute();
                },
                TokenizerState::BeforeAttributeValue => {
                    if c == ' ' {
                        continue;
                    }

                    if c == '"' {
                        self.state = TokenizerState::AttributeValueDoubleQuoted;
                        continue;
                    }

                    if c == '\'' {
                        self.state = TokenizerState::AttributeValueSingleQuoted;
                        continue;
                    }

                    self.reconsume = true;
                    self.state = TokenizerState::AttributeValueUnQuoted;

                    // > のときの処理はサボってまーす
                },
                TokenizerState::AttributeValueDoubleQuoted => {
                    if c == '"' {
                        self.state = TokenizerState::AfterAttributeValueQuoted;
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.append_character_to_attribute(c, AttributeField::Value);
                },
                TokenizerState::AttributeValueSingleQuoted => {
                    if c == '\'' {
                        self.state = TokenizerState::AfterAttributeValueQuoted;
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.append_character_to_attribute(c, AttributeField::Value);
                },
                TokenizerState::AttributeValueUnQuoted => {
                    if c == ' ' {
                        self.state = TokenizerState::BeforeAttributeName;
                        continue;
                    }

                    if c == '>' {
                        self.state = TokenizerState::Data;
                        return self.emit_latest_token();
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.append_character_to_attribute(c, AttributeField::Value);
                },
                TokenizerState::AfterAttributeValueQuoted => {
                    if c == ' ' {
                        self.state = TokenizerState::BeforeAttributeName;
                        continue;
                    }

                    if c == '/' {
                        self.state = TokenizerState::SelfClosingStartTag;
                        continue;
                    }

                    if c == '>' {
                        self.state = TokenizerState::Data;
                        return self.emit_latest_token();
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    self.reconsume = true;
                    self.state = TokenizerState::BeforeAttributeName;
                    
                },
                TokenizerState::SelfClosingStartTag => {
                    if c == '>' {
                        self.set_self_closing_flag();
                        self.state = TokenizerState::Data;
                        return self.emit_latest_token();
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }
                },
                TokenizerState::ScriptData => {
                    if c == '<' {
                        self.state = TokenizerState::ScriptDataLessThanSign;
                        continue;
                    }

                    if self.is_eof() {
                        return Some(HtmlToken::Eof);
                    }

                    return Some(HtmlToken::Char(c));
                },
                TokenizerState::ScriptDataLessThanSign => {
                    if c == '/' {
                        self.buf = String::new();
                        self.state = TokenizerState::ScriptDataEndTagOpen;
                        continue;
                    }

                    self.reconsume = true;
                    self.state = TokenizerState::ScriptData;
                    return Some(HtmlToken::Char('<'));
                },
                TokenizerState::ScriptDataEndTagOpen => {
                    if c.is_ascii_alphabetic() {
                        self.reconsume = true;
                        self.state = TokenizerState::ScriptDataEndTagName;
                    }

                    self.reconsume = true;
                    self.state = TokenizerState::ScriptData;
                    return Some(HtmlToken::Char('<')); // 本来は </ を返さないといけない
                },
                TokenizerState::ScriptDataEndTagName => {
                    if c == '>' {
                        self.state = TokenizerState::Data;
                        return self.emit_latest_token();
                    }

                    if c.is_ascii_alphabetic() {
                        self.buf.push(c);
                        self.append_tag_name(c.to_ascii_lowercase());
                        continue;
                    }

                    self.state = TokenizerState::TemporaryBuffer;
                    self.buf = String::from("</") + &self.buf;
                    self.buf.push(c);
                    continue;
                },
                TokenizerState::TemporaryBuffer => {
                    self.reconsume = true;

                    if self.buf.chars().count() == 0 {
                        self.state = TokenizerState::ScriptData;
                        continue;
                    }

                    let c = self.buf.chars().nth(0).expect("self.buf should have at least 1 char");
                    self.buf.remove(0);
                    return Some(HtmlToken::Char(c));
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_empty() {
        let html = "".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn test_start_and_end_tag() {
        let html = "<body></body>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = [
            HtmlToken::StartTag {
                tag: "body".to_string(),
                self_closing: false,
                attributes: Vec::new(),
            },
            HtmlToken::EndTag {
                tag: "body".to_string(),
            },
        ];
        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
    }

    #[test]
    fn test_attributes() {
        let html = "<p class=\"A\" id='B' foo=bar></p>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let mut attr1 = HtmlTagAttribute::new();
        attr1.add_char('c', AttributeField::Name);
        attr1.add_char('l', AttributeField::Name);
        attr1.add_char('a', AttributeField::Name);
        attr1.add_char('s', AttributeField::Name);
        attr1.add_char('s', AttributeField::Name);
        attr1.add_char('A', AttributeField::Value);

        let mut attr2 = HtmlTagAttribute::new();
        attr2.add_char('i', AttributeField::Name);
        attr2.add_char('d', AttributeField::Name);
        attr2.add_char('B', AttributeField::Value);

        let mut attr3 = HtmlTagAttribute::new();
        attr3.add_char('f', AttributeField::Name);
        attr3.add_char('o', AttributeField::Name);
        attr3.add_char('o', AttributeField::Name);
        attr3.add_char('b', AttributeField::Value);
        attr3.add_char('a', AttributeField::Value);
        attr3.add_char('r', AttributeField::Value);

        let expected = [
            HtmlToken::StartTag {
                tag: "p".to_string(),
                self_closing: false,
                attributes: vec![attr1, attr2, attr3],
            },
            HtmlToken::EndTag {
                tag: "p".to_string(),
            },
        ];
        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
    }

    #[test]
    fn test_self_closing_tag() {
        let html = "<img />".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = [HtmlToken::StartTag {
            tag: "img".to_string(),
            self_closing: true,
            attributes: Vec::new(),
        }];
        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
    }

    #[test]
    fn test_script_tag() {
        let html = "<script>js code;</script>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = [
            HtmlToken::StartTag {
                tag: "script".to_string(),
                self_closing: false,
                attributes: Vec::new(),
            },
            HtmlToken::Char('j'),
            HtmlToken::Char('s'),
            HtmlToken::Char(' '),
            HtmlToken::Char('c'),
            HtmlToken::Char('o'),
            HtmlToken::Char('d'),
            HtmlToken::Char('e'),
            HtmlToken::Char(';'),
            HtmlToken::EndTag {
                tag: "script".to_string(),
            },
        ];
        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
    }
}

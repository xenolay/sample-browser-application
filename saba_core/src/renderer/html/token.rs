use alloc::{string::String, vec::Vec};
use crate::renderer::html::html_tag_attribute::HtmlTagAttribute;

// [] 13.2.5 Tokenization | HTML Standard
// https://html.spec.whatwg.org/multipage/parsing.html#tokenization
// ----- Cited From Reference -----
// The output of the tokenization step is a series of zero or more of the following tokens: DOCTYPE, start tag, end tag, comment, character, end-of-file. DOCTYPE tokens have a name, a public identifier, a system identifier, and a force-quirks flag. When a DOCTYPE token is created, its name, public identifier, and system identifier must be marked as missing (which is a distinct state from the empty string), and the force-quirks flag must be set to off (its other state is on). Start and end tag tokens have a tag name, a self-closing flag, and a list of attributes, each of which has a name and a value. When a start or end tag token is created, its self-closing flag must be unset (its other state is that it be set), and its attributes list must be empty. Comment and character tokens have data.
// --------------------------------
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
pub enum TokenizationState {
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

pub struct HtmlTokenizer {
    state: TokenizationState,
    pos: usize,
    reconsume: bool,
    latest_token: Option<HtmlToken>,
    input: Vec<char>,
    buf: String,
}

impl HtmlTokenizer {
    pub fn new(html: String) -> Self {
        Self {
            state: TokenizationState::Data,
            pos: 0,
            reconsume: false,
            latest_token: None,
            input: html.chars().collect(),
            buf: String::new(),
        }
    }
}

impl Iterator for HtmlTokenizer {
    type Item = HtmlToken;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None
        }

        loop {
            match self.state {
                TokenizationState::Data => todo!(),
                TokenizationState::TagOpen => todo!(),
                TokenizationState::EndTagOpen => todo!(),
                TokenizationState::TagName => todo!(),
                TokenizationState::BeforeAttributeName => todo!(),
                TokenizationState::AttributeName => todo!(),
                TokenizationState::AfterAttributeName => todo!(),
                TokenizationState::BeforeAttributeValue => todo!(),
                TokenizationState::AttributeValueDoubleQuoted => todo!(),
                TokenizationState::AttributeValueSingleQuoted => todo!(),
                TokenizationState::AttributeValueUnQuoted => todo!(),
                TokenizationState::AfterAttributeValueQuoted => todo!(),
                TokenizationState::SelfClosingStartTag => todo!(),
                TokenizationState::ScriptData => todo!(),
                TokenizationState::ScriptDataLessThanSign => todo!(),
                TokenizationState::ScriptDataEndTagOpen => todo!(),
                TokenizationState::ScriptDataEndTagName => todo!(),
                TokenizationState::TemporaryBuffer => todo!(),
            }
        }
    }
}

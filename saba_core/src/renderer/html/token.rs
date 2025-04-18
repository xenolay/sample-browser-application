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

use alloc::string::String;
// [] 13.2.5 Tokenization | HTML Standard
// https://html.spec.whatwg.org/multipage/parsing.html#tokenization
// ----- Cited From Reference -----
// a list of attributes, each of which has a name and a value.
// --------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlTagAttribute {
    name: String,
    value: String,
}

pub enum AttributeField {
    Name,
    Value
}

impl HtmlTagAttribute {
    pub fn new() -> Self {
        Self { name: String::new(), value: String::new() }
    }

    pub fn add_char(&mut self, c: char, property: AttributeField) {
        match property {
            AttributeField::Name => self.name.push(c),
            AttributeField::Value => self.value.push(c),
        }
    }

    // これ struct の fields を pub にするのではダメか？
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn value(&self) -> String {
        self.value.clone()
    }
}

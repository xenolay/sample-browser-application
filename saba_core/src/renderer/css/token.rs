use alloc::{string::String, vec::Vec};


#[derive(Debug, Clone, PartialEq)]
pub enum CssToken {
    HashToken(String),
    Delim(char),
    Number(f64),
    Colon,
    SemiColon,
    OpenParenthesis,
    CloseParenthesis,
    OpenCurly,
    CloseCurly,
    Ident(String),
    StringToken(String),
    AtKeyword(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CssTokenizer {
    pos: usize,
    input: Vec<char>
}

impl CssTokenizer {
    pub fn new(css: String) -> Self {
        Self { pos: 0, input: css.chars().collect() }
    }

    // 文字列トークンを [start] の引用符でスキャンし、閉じ引用符の位置を返す
    fn consume_string_at(input: &[char], start: usize) -> (String, usize) {
        let ending = input[start];
        let mut s = String::new();
        let mut pos = start + 1;
        while pos < input.len() {
            let c = input[pos];
            if c == ending { break; }
            s.push(c);
            pos += 1;
        }
        (s, pos)
    }

    // 数値トークンを [start] からスキャンし、終端位置を返す
    fn consume_numeric_at(input: &[char], start: usize) -> (f64, usize) {
        let mut num = 0f64;
        let mut floating = false;
        let mut factor = 1f64;
        let mut pos = start;

        while pos < input.len() {
            match input[pos] {
                '0'..='9' => {
                    let digit = input[pos].to_digit(10).unwrap() as f64;
                    if floating {
                        factor *= 0.1;
                        num += digit * factor;
                    } else {
                        num = num * 10.0 + digit;
                    }
                    pos += 1;
                }
                '.' if !floating => {
                    floating = true;
                    pos += 1;
                }
                _ => break,
            }
        }
        (num, pos)
    }

    // 識別子トークンを [start] からスキャンし、終端位置を返す
    fn consume_ident_at(input: &[char], start: usize) -> (String, usize) {
        let mut s = String::new();
        let mut pos = start;
        while pos < input.len() {
            let c = input[pos];
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '#' {
                s.push(c);
                pos += 1;
            } else {
                break;
            }
        }
        (s, pos)
    }
}

impl Iterator for CssTokenizer {
    type Item = CssToken;

    fn next(&mut self) -> Option<Self::Item> {
        let input = &self.input;

        while self.pos < input.len() {
            let c = input[self.pos];

            // 空白をスキップ
            if c.is_whitespace() {
                self.pos += 1;
                continue;
            }

            let token = match c {
                '(' => { self.pos += 1; CssToken::OpenParenthesis }
                ')' => { self.pos += 1; CssToken::CloseParenthesis }
                ',' => { self.pos += 1; CssToken::Delim(',') }
                '.' => { self.pos += 1; CssToken::Delim('.') }
                ':' => { self.pos += 1; CssToken::Colon }
                ';' => { self.pos += 1; CssToken::SemiColon }
                '{' => { self.pos += 1; CssToken::OpenCurly }
                '}' => { self.pos += 1; CssToken::CloseCurly }
                ' ' | '\n' => {
                    self.pos += 1;
                    continue;
                }
                '"' | '\'' => {
                    let (s, next_pos) = Self::consume_string_at(input, self.pos);
                    self.pos = next_pos + 1;
                    CssToken::StringToken(s)
                }
                '0'..='9' => {
                    let (num, next_pos) = Self::consume_numeric_at(input, self.pos);
                    self.pos = next_pos;
                    CssToken::Number(num)
                }
                '#' => {
                    let (ident, next_pos) = Self::consume_ident_at(input, self.pos);
                    self.pos = next_pos;
                    CssToken::HashToken(ident)
                }
                '-' => {
                    let (ident, next_pos) = Self::consume_ident_at(input, self.pos);
                    self.pos = next_pos;
                    CssToken::Ident(ident)
                }
                '@' => {
                    // 次が英字なら at-keyword
                    if input.get(self.pos + 1).map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
                        let (ident, next_pos) = Self::consume_ident_at(input, self.pos + 1);
                        self.pos = next_pos;
                        CssToken::AtKeyword(ident)
                    } else {
                        self.pos += 1;
                        CssToken::Delim('@')
                    }
                }
                c if c.is_ascii_alphabetic() || c == '_' => {
                    let (ident, next_pos) = Self::consume_ident_at(input, self.pos);
                    self.pos = next_pos;
                    CssToken::Ident(ident)
                }
                _ => {
                    unimplemented!("char {} is not supported yet", c)
                }
            };

            return Some(token);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_empty() {
        let style = "".to_string();
        let mut t = CssTokenizer::new(style);
        assert!(t.next().is_none());
    }

    #[test]
    fn test_one_rule() {
        let style = "p { color: red; }".to_string();
        let mut t = CssTokenizer::new(style);
        let expected = [
            CssToken::Ident("p".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Ident("red".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
        ];
        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert!(t.next().is_none());
    }

    #[test]
    fn test_id_selector() {
        let style = "#id { color: red; }".to_string();
        let mut t = CssTokenizer::new(style);
        let expected = [
            CssToken::HashToken("#id".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Ident("red".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
        ];
        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert!(t.next().is_none());
    }

    #[test]
    fn test_class_selector() {
        let style = ".class { color: red; }".to_string();
        let mut t = CssTokenizer::new(style);
        let expected = [
            CssToken::Delim('.'),
            CssToken::Ident("class".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Ident("red".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
        ];
        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert!(t.next().is_none());
    }

    #[test]
    fn test_multiple_rules() {
        // The value like "40px" is not supported yet.
        let style = "p { content: \"Hey\"; } h1 { font-size: 40; color: blue; }".to_string();
        let mut t = CssTokenizer::new(style);
        let expected = [
            CssToken::Ident("p".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("content".to_string()),
            CssToken::Colon,
            CssToken::StringToken("Hey".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
            CssToken::Ident("h1".to_string()),
            CssToken::OpenCurly,
            CssToken::Ident("font-size".to_string()),
            CssToken::Colon,
            CssToken::Number(40.0),
            CssToken::SemiColon,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Ident("blue".to_string()),
            CssToken::SemiColon,
            CssToken::CloseCurly,
        ];
        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert!(t.next().is_none());
    }
}

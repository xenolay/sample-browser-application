use alloc::{string::String, vec::Vec};


#[derive(Debug, Clone, PartialEq)]
pub enum CssToken {
    HashToken(String),
    Delim(char),
    Number(f64),
    Colon,
    Semicolon,
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

    fn consume_string_token(&mut self) -> String {
        let mut s = String::new();

        loop {
            if self.pos >= self.input.len() {
                return s;
            }

            self.pos += 1;

            let c = self.input[self.pos];
            match c {
                '"' | '\'' => break,
                _ => s.push(c),
            }
        }

        s
    }

    fn consume_numeric_token(&mut self) -> f64 {
        let mut num = 0f64;
        let mut floating = false;
        let mut floating_digit = 1f64;

        loop {
            if self.pos >= self.input.len() {
                return num;
            }

            let c = self.input[self.pos];

            // 数字と . でない文字が出てくるまで self.pos を進める。つまり関数を抜けたタイミングで self.pos は数字でも . でもない文字を指している。
            // なので、この関数を抜けた直後で self.pos は -1 しないといけない
            match c {
                '0'..='9' => {
                    if floating {
                        floating_digit *= 1f64/10f64;
                        num += (c.to_digit(10).unwrap() as f64) * floating_digit
                    } else {
                        num = num * 10.0 + (c.to_digit(10).unwrap() as f64)
                    }
                    self.pos += 1;
                },
                '.' => {
                    floating = true;
                    self.pos += 1;
                },
                _ => break,
            }
        }

        num
    }

    fn consume_ident_token(&mut self) -> String {
        let mut s = String::new();
        s.push(self.input[self.pos]);

        loop {
            self.pos += 1;
            let c = self.input[self.pos];
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => {
                    s.push(c)
                }
                _ => break,
            }
        }

        s
    }
}

impl Iterator for CssTokenizer {
    type Item = CssToken;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos > self.input.len() {
                return None;
            }

            let c = self.input.get(self.pos)?;


            let token = match c {
                '(' => CssToken::OpenParenthesis,
                ')' => CssToken::CloseParenthesis,
                ',' => CssToken::Delim(','),
                '.' => CssToken::Delim('.'),
                ':' => CssToken::Colon,
                ';' => CssToken::Semicolon,
                '{' => CssToken::OpenCurly,
                '}' => CssToken::CloseCurly,
                ' ' | '\n' => {
                    self.pos += 1;
                    continue;
                }
                '"' | '\'' => {
                    let value = self.consume_string_token();
                    CssToken::StringToken(value)
                },
                '0'..='9' => {
                    let value = self.consume_numeric_token();
                    CssToken::Number(value)
                },
                '#' => {
                    let value = self.consume_ident_token();
                    self.pos -= 1;
                    CssToken::HashToken(value)
                }
                '-' => {
                    let value = self.consume_ident_token();
                    self.pos -= 1;
                    CssToken::Ident(value)
                }
                '@' => {
                    if self.input[self.pos + 1].is_ascii_alphabetic()
                    && self.input[self.pos + 2].is_alphanumeric()
                    && self.input[self.pos + 3].is_alphabetic() {
                        self.pos += 1;
                        let value = self.consume_ident_token();
                        self.pos -= 1;
                        CssToken::AtKeyword(value)
                    } else {
                        CssToken::Delim('@')
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let value = self.consume_ident_token();
                    self.pos -= 1;
                    CssToken::Ident(value)
                }
                _ => {
                    unimplemented!("char {} is not supported yet", c)
                }
            };

            self.pos += 1;
            return Some(token);
        }
    }
}

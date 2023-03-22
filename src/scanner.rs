use std::collections::HashMap;
use std::fmt::Display;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literal
    String(String),
    Number(f64),
    Boolean(String),
    Null,
    Undefined,

    Identifier(String),
    Comment(String),

    // Arithemic operations
    Plus,
    PlusPlus,
    Minus,
    MinusMinus,
    Mul,
    MulMul,
    Div,

    // Assignment
    Equal,
    PlusEqual,
    MinusEqual,
    MulEqual,
    DivEqual,

    // Punctuation
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    EqualEqual,
    EqualEqualEqual,
    Colon,
    Semicolon,
    Dot,
    Exclamatory,

    // Keywords
    FunctionKeyword,
    IfKeyword,
    ElseKeyword,
    WhileKeyword,
    DoKeyword,
    ForKeyword,
    InKeyword,
    ClassKeyword,
    ExtendsKeyword,
    LetKeyword,
    ConstKeyword,
    ThisKeyword,
    TryKeyword,
    CatchKeyword,
    NewKeyword,
    BreakKeyword,
    ContinueKeyword,
    SuperKeyword,
    ThrowKeyword,
    YieldKeyword,
    ExportKeyword,
    ImportKeyword,
    StaticKeyword,
    SwitchKeyword,
    PrintKeyword,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Scanner {
    current_pos: usize,
    current_line: usize,
    source_code: String,
}

impl Scanner {
    pub fn new(source_code: String) -> Self {
        Self {
            current_pos: 0,
            current_line: 0,
            source_code,
        }
    }

    pub fn next_token(&mut self) -> Option<TokenKind> {
        if self.current_pos >= self.source_code.len() {
            return None;
        }

        let mut chars = self.source_code.chars();
        let current_char = chars.nth(self.current_pos).unwrap();
        let mut cursor = self.current_pos;

        if current_char.is_whitespace() {
            self.current_pos += 1;
            return self.next_token();
        }

        if current_char == '\n' {
            self.current_line += 1;
        }

        let found_token = match current_char {
            ',' => Some(TokenKind::Colon),
            ';' => Some(TokenKind::Semicolon),
            '(' => Some(TokenKind::OpenParen),
            ')' => Some(TokenKind::CloseParen),
            '{' => Some(TokenKind::OpenBrace),
            '}' => Some(TokenKind::CloseBrace),
            '.' => Some(TokenKind::Dot),
            '!' => Some(TokenKind::Exclamatory),
            _ => None,
        };

        if current_char == '/' {
            self.current_pos += 1;

            if let Some('=') = chars.next() {
                self.current_pos += 1;
                return Some(TokenKind::DivEqual);
            }

            if let Some('/') = chars.next() {
                self.current_pos += 1;

                while let Some(char) = chars.next() {
                    cursor += 1;

                    if char == '\n' {
                        break;
                    }
                }

                let token =
                    TokenKind::Comment(self.source_code[self.current_pos..=cursor].to_string());

                self.current_pos = cursor + 1;

                return Some(token);
            } else {
                return Some(TokenKind::Div);
            }
        }

        if current_char == '=' {
            self.current_pos += 1;

            if let Some('=') = chars.next() {
                self.current_pos += 1;

                if let Some('=') = chars.next() {
                    self.current_pos += 1;
                    return Some(TokenKind::EqualEqualEqual);
                }

                return Some(TokenKind::EqualEqual);
            }

            return Some(TokenKind::Equal);
        }

        if current_char == '+' {
            self.current_pos += 1;

            if let Some('=') = chars.next() {
                self.current_pos += 1;
                return Some(TokenKind::PlusEqual);
            }

            if let Some('+') = chars.next() {
                self.current_pos += 1;
                return Some(TokenKind::PlusPlus);
            }

            return Some(TokenKind::Plus);
        }

        if current_char == '*' {
            self.current_pos += 1;

            if let Some('=') = chars.next() {
                self.current_pos += 1;
                return Some(TokenKind::MulEqual);
            }

            if let Some('*') = chars.next() {
                self.current_pos += 1;
                return Some(TokenKind::MulMul);
            }

            return Some(TokenKind::Mul);
        }

        if current_char == '-' {
            self.current_pos += 1;

            if let Some('=') = chars.next() {
                self.current_pos += 1;
                return Some(TokenKind::MinusEqual);
            }

            if let Some('-') = chars.next() {
                self.current_pos += 1;
                return Some(TokenKind::MinusMinus);
            }

            return Some(TokenKind::Minus);
        }

        if let Some(_) = found_token {
            self.current_pos += 1;
            return found_token;
        }

        if current_char.is_digit(10) {
            while let Some(char) = chars.next() {
                if char == '.' {
                    cursor += 1;
                } else if char.is_digit(10) {
                    cursor += 1;
                } else {
                    break;
                }
            }

            let number_str = &self.source_code[self.current_pos..=cursor];
            let number = number_str
                .parse::<f64>()
                .expect("Error during number parsing");
            let token = TokenKind::Number(number);

            self.current_pos = cursor + 1;

            return Some(token);
        }

        if current_char == '"' || current_char == '\'' {
            return self.parse_string_literal(current_char);
        }

        if !current_char.is_alphabetic() {
            unreachable!()
        }

        while let Some(char) = chars.next() {
            if !char.is_alphanumeric() {
                break;
            }

            match char {
                '(' | ')' | '.' | ';' | '\n' | ',' => break,
                _ => {}
            }

            cursor += 1;
        }

        let keywords = HashMap::from([
            ("let", TokenKind::LetKeyword),
            ("const", TokenKind::ConstKeyword),
            ("if", TokenKind::IfKeyword),
            ("else", TokenKind::ElseKeyword),
            ("class", TokenKind::ClassKeyword),
            ("new", TokenKind::NewKeyword),
            ("extends", TokenKind::ExtendsKeyword),
            ("for", TokenKind::ForKeyword),
            ("in", TokenKind::InKeyword),
            ("function", TokenKind::FunctionKeyword),
            ("this", TokenKind::ThisKeyword),
            ("do", TokenKind::DoKeyword),
            ("while", TokenKind::WhileKeyword),
            ("try", TokenKind::TryKeyword),
            ("catch", TokenKind::CatchKeyword),
            ("break", TokenKind::BreakKeyword),
            ("continue", TokenKind::ContinueKeyword),
            ("super", TokenKind::SuperKeyword),
            ("throw", TokenKind::ThrowKeyword),
            ("yield", TokenKind::YieldKeyword),
            ("export", TokenKind::ExportKeyword),
            ("import", TokenKind::ImportKeyword),
            ("static", TokenKind::StaticKeyword),
            ("switch", TokenKind::SwitchKeyword),
            ("true", TokenKind::Boolean("true".to_string())),
            ("false", TokenKind::Boolean("false".to_string())),
            ("null", TokenKind::Null),
            ("undefined", TokenKind::Undefined),
            ("print", TokenKind::PrintKeyword),
        ]);

        let identifier = &self.source_code[self.current_pos..=cursor];

        if keywords.contains_key(identifier) {
            let token_kind = keywords.get(identifier).unwrap();
            self.current_pos += identifier.len();
            return Some(token_kind.clone());
        } else {
            self.current_pos += identifier.len();
            return Some(TokenKind::Identifier(identifier.to_string()));
        }
    }

    fn parse_string_literal(&mut self, quote_char: char) -> Option<TokenKind> {
        let mut cursor = self.current_pos;
        let mut chars = self.source_code[cursor..].chars();

        chars.next();

        while let Some(char) = chars.next() {
            cursor += 1;

            if char == quote_char {
                break;
            }
        }

        let token = TokenKind::String(self.source_code[self.current_pos + 1..cursor].to_string());
        self.current_pos = cursor + 1;
        return Some(token);
    }
}

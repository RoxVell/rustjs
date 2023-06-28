use std::collections::HashMap;
use std::fmt::Display;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literal
    String(String),
    Number(f64),
    Boolean(String),
    Null,
    Undefined,

    Identifier(String),
    Comment(String),

    // Logical operations
    Or,  // ||
    And, // &&

    // Bitwise
    BitwiseOr,
    BitwiseAnd,

    // Comparison operators
    LessThan,
    LessThanOrEqual,
    MoreThan,
    MoreThanOrEqual,

    // Arithemic operations
    Plus,
    PlusPlus,
    Minus,
    MinusMinus,
    Mul,
    MulMul,
    Div,
    Percent,

    // Assignment
    Equal,
    PlusEqual,
    MinusEqual,
    MulEqual,
    DivEqual,
    PercentEqual,
    MulMulEqual,
    LSLSEqual,   // <<=
    RSRSEqual,   // >>=
    RSRSRSEqual, // >>=
    // *= /= %= += -= <<= >>= >>>= &= ^= |= **=

    // Equality
    Equality,
    StrictEquality,
    Inequality,
    StrictInequality,

    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenSquareBracket,
    CloseSquareBracket,

    Comma,
    Semicolon,
    Colon,
    Dot,
    Exclamatory, // !
    Question,    // ?

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
    ReturnKeyword,
}

impl Token {
    pub fn to_keyword(&self) -> String {
        match self {
            Token::String(value) => format!("{} (string)", value),
            Token::Number(value) => format!("{} (number)", value),
            Token::Boolean(value) => format!("{} (boolean)", value),
            Token::Null => "null".to_string(),
            Token::Undefined => "undefined".to_string(),
            Token::Identifier(_) => "identifier".to_string(),
            Token::Comment(_) => "null".to_string(),
            Token::Or => "||".to_string(),
            Token::And => "&&".to_string(),
            Token::BitwiseOr => "|".to_string(),
            Token::BitwiseAnd => "&".to_string(),
            Token::Plus => "+".to_string(),
            Token::PlusPlus => "++".to_string(),
            Token::Minus => "--".to_string(),
            Token::MinusMinus => "--".to_string(),
            Token::Mul => "*".to_string(),
            Token::MulMul => "**".to_string(),
            Token::Div => "/".to_string(),
            Token::Equal => "=".to_string(),
            Token::Percent => "%".to_string(),
            Token::PlusEqual => "+=".to_string(),
            Token::MinusEqual => "-=".to_string(),
            Token::MulEqual => "*=".to_string(),
            Token::DivEqual => "/=".to_string(),
            Token::PercentEqual => "%=".to_string(),
            Token::MulMulEqual => "**=".to_string(),
            Token::LSLSEqual => "<<=".to_string(),
            Token::RSRSEqual => ">>=".to_string(),
            Token::RSRSRSEqual => ">>>=".to_string(),
            Token::OpenParen => "(".to_string(),
            Token::CloseParen => ")".to_string(),
            Token::OpenBrace => "{".to_string(),
            Token::CloseBrace => "}".to_string(),
            Token::Equality => "==".to_string(),
            Token::StrictEquality => "===".to_string(),
            Token::Inequality => "!=".to_string(),
            Token::StrictInequality => "!==".to_string(),
            Token::Comma => ",".to_string(),
            Token::Semicolon => ";".to_string(),
            Token::Dot => ".".to_string(),
            Token::Exclamatory => "!".to_string(),
            Token::FunctionKeyword => "function".to_string(),
            Token::IfKeyword => "if".to_string(),
            Token::ElseKeyword => "else".to_string(),
            Token::WhileKeyword => "while".to_string(),
            Token::DoKeyword => "do".to_string(),
            Token::ForKeyword => "for".to_string(),
            Token::InKeyword => "in".to_string(),
            Token::ClassKeyword => "class".to_string(),
            Token::ExtendsKeyword => "extends".to_string(),
            Token::ConstKeyword => "const".to_string(),
            Token::LetKeyword => "let".to_string(),
            Token::ThisKeyword => "this".to_string(),
            Token::TryKeyword => "try".to_string(),
            Token::CatchKeyword => "catch".to_string(),
            Token::NewKeyword => "new".to_string(),
            Token::BreakKeyword => "break".to_string(),
            Token::ContinueKeyword => "continue".to_string(),
            Token::SuperKeyword => "super".to_string(),
            Token::ThrowKeyword => "throw".to_string(),
            Token::YieldKeyword => "yield".to_string(),
            Token::ExportKeyword => "export".to_string(),
            Token::ImportKeyword => "null".to_string(),
            Token::StaticKeyword => "static".to_string(),
            Token::SwitchKeyword => "switch".to_string(),
            Token::PrintKeyword => "print".to_string(),
            Token::ReturnKeyword => "return".to_string(),
            Token::LessThan => "<".to_string(),
            Token::LessThanOrEqual => "<=".to_string(),
            Token::MoreThan => ">".to_string(),
            Token::MoreThanOrEqual => ">=".to_string(),
            Token::Question => "?".to_string(),
            Token::Colon => ":".to_string(),
            Token::OpenSquareBracket => "[".to_string(),
            Token::CloseSquareBracket => "]".to_string(),
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithLocation {
    pub token: Token,
    pub start: Span,
    pub end: Span,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Span {
    pub line: usize,
    pub row: usize,
}

pub struct Scanner {
    current_pos: usize,
    current_line: usize,
    prev_pos: usize,
    prev_line: usize,
    source_code: String,
}

impl Scanner {
    pub fn new(source_code: String) -> Self {
        Self {
            prev_pos: 0,
            prev_line: 0,
            current_pos: 0,
            current_line: 0,
            source_code,
        }
    }

    fn consume(&self, token: Token) -> TokenWithLocation {
        TokenWithLocation {
            token,
            start: Span {
                line: self.prev_line,
                row: self.prev_pos,
            },
            end: Span {
                line: self.current_line,
                row: self.current_pos,
            },
        }
    }

    pub fn next_token(&mut self) -> Option<TokenWithLocation> {
        self.prev_line = self.current_line;
        self.prev_pos = self.current_pos;
        let mut cursor = self.current_pos;

        if self.current_pos >= self.source_code.len() {
            return None;
        }

        let mut chars = self.source_code.chars();
        let current_char = chars.nth(self.current_pos).unwrap();

        if current_char == '\n' {
            self.current_line += 1;
        }

        if current_char.is_whitespace() {
            self.current_pos += 1;
            return self.next_token();
        }

        let found_token = match current_char {
            ',' => Some(Token::Comma),
            ';' => Some(Token::Semicolon),
            ':' => Some(Token::Colon),
            '(' => Some(Token::OpenParen),
            ')' => Some(Token::CloseParen),
            '{' => Some(Token::OpenBrace),
            '}' => Some(Token::CloseBrace),
            '[' => Some(Token::OpenSquareBracket),
            ']' => Some(Token::CloseSquareBracket),
            '.' => Some(Token::Dot),
            '?' => Some(Token::Question),
            _ => None,
        };

        if current_char == '=' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;

                if let Some('=') = next_char {
                    self.current_pos += 1;
                    return Some(self.consume(Token::StrictEquality));
                }

                return Some(self.consume(Token::Equality));
            }

            return Some(self.consume(Token::Equal));
        }

        if current_char == '!' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;

                if let Some('=') = next_char {
                    self.current_pos += 1;
                    return Some(self.consume(Token::StrictInequality));
                }

                return Some(self.consume(Token::Inequality));
            }

            return Some(self.consume(Token::Exclamatory));
        }

        if current_char == '%' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;
                return Some(self.consume(Token::PercentEqual));
            }

            return Some(self.consume(Token::Percent));
        }

        if current_char == '>' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;
                return Some(self.consume(Token::MoreThanOrEqual));
            }

            return Some(self.consume(Token::MoreThan));
        }

        if current_char == '<' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;
                return Some(self.consume(Token::LessThanOrEqual));
            }

            return Some(self.consume(Token::LessThan));
        }

        if current_char == '/' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;
                return Some(self.consume(Token::DivEqual));
            }

            if let Some('/') = next_char {
                self.current_pos += 1;

                while let Some(char) = chars.next() {
                    cursor += 1;

                    if char == '\n' {
                        break;
                    }
                }

                let token =
                    Token::Comment(self.source_code[self.current_pos..=cursor + 1].to_string());
                self.current_pos = cursor + 2;
                return Some(self.consume(token));
            } else {
                return Some(self.consume(Token::Div));
            }
        }

        if current_char == '=' {
            self.current_pos += 1;

            if let Some('=') = chars.next() {
                self.current_pos += 1;

                if let Some('=') = chars.next() {
                    self.current_pos += 1;
                    return Some(self.consume(Token::StrictEquality));
                }

                return Some(self.consume(Token::Equality));
            }

            return Some(self.consume(Token::Equal));
        }

        if current_char == '+' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;
                return Some(self.consume(Token::PlusEqual));
            }

            if let Some('+') = next_char {
                self.current_pos += 1;
                return Some(self.consume(Token::PlusPlus));
            }

            return Some(self.consume(Token::Plus));
        }

        if current_char == '*' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;
                return Some(self.consume(Token::MulEqual));
            }

            if let Some('*') = next_char {
                self.current_pos += 1;

                let next_char = chars.next();

                if let Some('=') = next_char {
                    self.current_pos += 1;
                    return Some(self.consume(Token::MulMulEqual));
                }

                return Some(self.consume(Token::MulMul));
            }

            return Some(self.consume(Token::Mul));
        }

        if current_char == '-' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;
                return Some(self.consume(Token::MinusEqual));
            }

            if let Some('-') = next_char {
                self.current_pos += 1;
                return Some(self.consume(Token::MinusMinus));
            }

            return Some(self.consume(Token::Minus));
        }

        if current_char == '|' {
            self.current_pos += 1;

            if let Some('|') = chars.next() {
                self.current_pos += 1;
                return Some(self.consume(Token::Or));
            }

            return Some(self.consume(Token::BitwiseOr));
        }

        if current_char == '&' {
            self.current_pos += 1;

            if let Some('&') = chars.next() {
                self.current_pos += 1;
                return Some(self.consume(Token::And));
            }

            return Some(self.consume(Token::BitwiseAnd));
        }

        if let Some(_) = found_token {
            self.current_pos += 1;
            return found_token.map(|x| self.consume(x));
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
            let token = Token::Number(number);

            self.current_pos = cursor + 1;

            return Some(self.consume(token));
        }

        if current_char == '"' || current_char == '\'' {
            return self
                .parse_string_literal(current_char)
                .map(|x| self.consume(x));
        }

        while let Some(char) = chars.next() {
            if !char.is_alphanumeric() && char != '_' {
                break;
            }

            match char {
                '(' | ')' | '.' | ';' | '\n' | ',' => break,
                _ => {}
            }

            cursor += 1;
        }

        let keywords = HashMap::from([
            ("let", Token::LetKeyword),
            ("const", Token::ConstKeyword),
            ("if", Token::IfKeyword),
            ("else", Token::ElseKeyword),
            ("class", Token::ClassKeyword),
            ("new", Token::NewKeyword),
            ("extends", Token::ExtendsKeyword),
            ("for", Token::ForKeyword),
            ("in", Token::InKeyword),
            ("function", Token::FunctionKeyword),
            ("this", Token::ThisKeyword),
            ("do", Token::DoKeyword),
            ("while", Token::WhileKeyword),
            ("try", Token::TryKeyword),
            ("catch", Token::CatchKeyword),
            ("break", Token::BreakKeyword),
            ("continue", Token::ContinueKeyword),
            ("super", Token::SuperKeyword),
            ("throw", Token::ThrowKeyword),
            ("yield", Token::YieldKeyword),
            ("export", Token::ExportKeyword),
            ("import", Token::ImportKeyword),
            ("return", Token::ReturnKeyword),
            ("static", Token::StaticKeyword),
            ("switch", Token::SwitchKeyword),
            ("true", Token::Boolean("true".to_string())),
            ("false", Token::Boolean("false".to_string())),
            ("null", Token::Null),
            ("undefined", Token::Undefined),
            ("print", Token::PrintKeyword),
        ]);

        let identifier = &self.source_code[self.current_pos..=cursor];

        if keywords.contains_key(identifier) {
            let token_kind = keywords.get(identifier).unwrap();
            self.current_pos += identifier.len();
            return Some(self.consume(token_kind.clone()));
        } else {
            self.current_pos += identifier.len();
            return Some(self.consume(Token::Identifier(identifier.to_string())));
        }
    }

    fn parse_string_literal(&mut self, quote_char: char) -> Option<Token> {
        let mut cursor = self.current_pos;
        let mut chars = self.source_code[cursor..].chars();

        chars.next();

        while let Some(char) = chars.next() {
            cursor += 1;

            if char == quote_char {
                break;
            }
        }

        let token = Token::String(self.source_code[self.current_pos + 1..cursor].to_string());
        self.current_pos = cursor + 1;
        return Some(token);
    }
}

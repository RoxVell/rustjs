use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use crate::keywords::{BREAK_KEYWORD, CATCH_KEYWORD, CLASS_KEYWORD, CONST_KEYWORD, CONTINUE_KEYWORD, DO_KEYWORD, ELSE_KEYWORD, EXPORT_KEYWORD, EXTENDS_KEYWORD, FALSE_KEYWORD, FOR_KEYWORD, FUNCTION_KEYWORD, IF_KEYWORD, IMPORT_KEYWORD, IN_KEYWORD, LET_KEYWORD, NEW_KEYWORD, NULL_KEYWORD, RETURN_KEYWORD, STATIC_KEYWORD, SUPER_KEYWORD, SWITCH_KEYWORD, THIS_KEYWORD, THROW_KEYWORD, TRUE_KEYWORD, TRY_KEYWORD, UNDEFINED_KEYWORD, WHILE_KEYWORD, YIELD_KEYWORD};
use crate::scanner::TokenKind::{DoKeyword, ElseKeyword, ForKeyword, IfKeyword, WhileKeyword};

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

    // Arithmetic operations
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
    ReturnKeyword,
}

impl TokenKind {
    pub fn to_keyword(&self) -> String {
        match self {
            TokenKind::String(value) => format!("{} (string)", value),
            TokenKind::Number(value) => format!("{} (number)", value),
            TokenKind::Boolean(value) => format!("{} (boolean)", value),
            TokenKind::Null => NULL_KEYWORD.to_string(),
            TokenKind::Undefined => UNDEFINED_KEYWORD.to_string(),
            TokenKind::Identifier(_) => "identifier".to_string(),
            TokenKind::Comment(_) => NULL_KEYWORD.to_string(),
            TokenKind::Or => "||".to_string(),
            TokenKind::And => "&&".to_string(),
            TokenKind::BitwiseOr => "|".to_string(),
            TokenKind::BitwiseAnd => "&".to_string(),
            TokenKind::Plus => "+".to_string(),
            TokenKind::PlusPlus => "++".to_string(),
            TokenKind::Minus => "--".to_string(),
            TokenKind::MinusMinus => "--".to_string(),
            TokenKind::Mul => "*".to_string(),
            TokenKind::MulMul => "**".to_string(),
            TokenKind::Div => "/".to_string(),
            TokenKind::Equal => "=".to_string(),
            TokenKind::Percent => "%".to_string(),
            TokenKind::PlusEqual => "+=".to_string(),
            TokenKind::MinusEqual => "-=".to_string(),
            TokenKind::MulEqual => "*=".to_string(),
            TokenKind::DivEqual => "/=".to_string(),
            TokenKind::PercentEqual => "%=".to_string(),
            TokenKind::MulMulEqual => "**=".to_string(),
            TokenKind::LSLSEqual => "<<=".to_string(),
            TokenKind::RSRSEqual => ">>=".to_string(),
            TokenKind::RSRSRSEqual => ">>>=".to_string(),
            TokenKind::OpenParen => "(".to_string(),
            TokenKind::CloseParen => ")".to_string(),
            TokenKind::OpenBrace => "{".to_string(),
            TokenKind::CloseBrace => "}".to_string(),
            TokenKind::Equality => "==".to_string(),
            TokenKind::StrictEquality => "===".to_string(),
            TokenKind::Inequality => "!=".to_string(),
            TokenKind::StrictInequality => "!==".to_string(),
            TokenKind::Comma => ",".to_string(),
            TokenKind::Semicolon => ";".to_string(),
            TokenKind::Dot => ".".to_string(),
            TokenKind::Exclamatory => "!".to_string(),
            TokenKind::FunctionKeyword => FUNCTION_KEYWORD.to_string(),
            TokenKind::IfKeyword => IF_KEYWORD.to_string(),
            TokenKind::ElseKeyword => ElseKeyword.to_string(),
            TokenKind::WhileKeyword => WhileKeyword.to_string(),
            TokenKind::DoKeyword => DoKeyword.to_string(),
            TokenKind::ForKeyword => ForKeyword.to_string(),
            TokenKind::InKeyword => IN_KEYWORD.to_string(),
            TokenKind::ClassKeyword => CLASS_KEYWORD.to_string(),
            TokenKind::ExtendsKeyword => EXTENDS_KEYWORD.to_string(),
            TokenKind::ConstKeyword => CONST_KEYWORD.to_string(),
            TokenKind::LetKeyword => LET_KEYWORD.to_string(),
            TokenKind::ThisKeyword => THIS_KEYWORD.to_string(),
            TokenKind::TryKeyword => TRY_KEYWORD.to_string(),
            TokenKind::CatchKeyword => CATCH_KEYWORD.to_string(),
            TokenKind::NewKeyword => NEW_KEYWORD.to_string(),
            TokenKind::BreakKeyword => BREAK_KEYWORD.to_string(),
            TokenKind::ContinueKeyword => CONTINUE_KEYWORD.to_string(),
            TokenKind::SuperKeyword => SUPER_KEYWORD.to_string(),
            TokenKind::ThrowKeyword => THROW_KEYWORD.to_string(),
            TokenKind::YieldKeyword => YIELD_KEYWORD.to_string(),
            TokenKind::ExportKeyword => EXPORT_KEYWORD.to_string(),
            TokenKind::ImportKeyword => IMPORT_KEYWORD.to_string(),
            TokenKind::StaticKeyword => STATIC_KEYWORD.to_string(),
            TokenKind::SwitchKeyword => SWITCH_KEYWORD.to_string(),
            TokenKind::ReturnKeyword => RETURN_KEYWORD.to_string(),
            TokenKind::LessThan => "<".to_string(),
            TokenKind::LessThanOrEqual => "<=".to_string(),
            TokenKind::MoreThan => ">".to_string(),
            TokenKind::MoreThanOrEqual => ">=".to_string(),
            TokenKind::Question => "?".to_string(),
            TokenKind::Colon => ":".to_string(),
            TokenKind::OpenSquareBracket => "[".to_string(),
            TokenKind::CloseSquareBracket => "]".to_string(),
        }
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextSpan {
    pub start: Span,
    pub end: Span,
}

#[derive(Clone, PartialEq)]
pub struct Token {
    pub token: TokenKind,
    pub span: TextSpan,
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Token").field("token", &self.token).finish()
    }
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

    fn consume(&self, token: TokenKind) -> Token {
        Token {
            token,
            span: TextSpan {
                start: Span {
                    line: self.prev_line,
                    row: self.prev_pos,
                },
                end: Span {
                    line: self.current_line,
                    row: self.current_pos,
                }
            },
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
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
            ',' => Some(TokenKind::Comma),
            ';' => Some(TokenKind::Semicolon),
            ':' => Some(TokenKind::Colon),
            '(' => Some(TokenKind::OpenParen),
            ')' => Some(TokenKind::CloseParen),
            '{' => Some(TokenKind::OpenBrace),
            '}' => Some(TokenKind::CloseBrace),
            '[' => Some(TokenKind::OpenSquareBracket),
            ']' => Some(TokenKind::CloseSquareBracket),
            '.' => Some(TokenKind::Dot),
            '?' => Some(TokenKind::Question),
            _ => None,
        };

        if current_char == '=' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;

                if let Some('=') = next_char {
                    self.current_pos += 1;
                    return Some(self.consume(TokenKind::StrictEquality));
                }

                return Some(self.consume(TokenKind::Equality));
            }

            return Some(self.consume(TokenKind::Equal));
        }

        if current_char == '!' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;

                if let Some('=') = next_char {
                    self.current_pos += 1;
                    return Some(self.consume(TokenKind::StrictInequality));
                }

                return Some(self.consume(TokenKind::Inequality));
            }

            return Some(self.consume(TokenKind::Exclamatory));
        }

        if current_char == '%' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;
                return Some(self.consume(TokenKind::PercentEqual));
            }

            return Some(self.consume(TokenKind::Percent));
        }

        if current_char == '>' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;
                return Some(self.consume(TokenKind::MoreThanOrEqual));
            }

            return Some(self.consume(TokenKind::MoreThan));
        }

        if current_char == '<' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;
                return Some(self.consume(TokenKind::LessThanOrEqual));
            }

            return Some(self.consume(TokenKind::LessThan));
        }

        if current_char == '/' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;
                return Some(self.consume(TokenKind::DivEqual));
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
                    TokenKind::Comment(self.source_code[self.current_pos..=cursor + 1].to_string());
                self.current_pos = cursor + 2;
                return Some(self.consume(token));
            } else {
                return Some(self.consume(TokenKind::Div));
            }
        }

        if current_char == '=' {
            self.current_pos += 1;

            if let Some('=') = chars.next() {
                self.current_pos += 1;

                if let Some('=') = chars.next() {
                    self.current_pos += 1;
                    return Some(self.consume(TokenKind::StrictEquality));
                }

                return Some(self.consume(TokenKind::Equality));
            }

            return Some(self.consume(TokenKind::Equal));
        }

        if current_char == '+' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;
                return Some(self.consume(TokenKind::PlusEqual));
            }

            if let Some('+') = next_char {
                self.current_pos += 1;
                return Some(self.consume(TokenKind::PlusPlus));
            }

            return Some(self.consume(TokenKind::Plus));
        }

        if current_char == '*' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;
                return Some(self.consume(TokenKind::MulEqual));
            }

            if let Some('*') = next_char {
                self.current_pos += 1;

                let next_char = chars.next();

                if let Some('=') = next_char {
                    self.current_pos += 1;
                    return Some(self.consume(TokenKind::MulMulEqual));
                }

                return Some(self.consume(TokenKind::MulMul));
            }

            return Some(self.consume(TokenKind::Mul));
        }

        if current_char == '-' {
            self.current_pos += 1;

            let next_char = chars.next();

            if let Some('=') = next_char {
                self.current_pos += 1;
                return Some(self.consume(TokenKind::MinusEqual));
            }

            if let Some('-') = next_char {
                self.current_pos += 1;
                return Some(self.consume(TokenKind::MinusMinus));
            }

            return Some(self.consume(TokenKind::Minus));
        }

        if current_char == '|' {
            self.current_pos += 1;

            if let Some('|') = chars.next() {
                self.current_pos += 1;
                return Some(self.consume(TokenKind::Or));
            }

            return Some(self.consume(TokenKind::BitwiseOr));
        }

        if current_char == '&' {
            self.current_pos += 1;

            if let Some('&') = chars.next() {
                self.current_pos += 1;
                return Some(self.consume(TokenKind::And));
            }

            return Some(self.consume(TokenKind::BitwiseAnd));
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
            let token = TokenKind::Number(number);

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
            (LET_KEYWORD, TokenKind::LetKeyword),
            (CONST_KEYWORD, TokenKind::ConstKeyword),
            (IF_KEYWORD, TokenKind::IfKeyword),
            (ELSE_KEYWORD, TokenKind::ElseKeyword),
            (CLASS_KEYWORD, TokenKind::ClassKeyword),
            (NEW_KEYWORD, TokenKind::NewKeyword),
            (EXTENDS_KEYWORD, TokenKind::ExtendsKeyword),
            (FOR_KEYWORD, TokenKind::ForKeyword),
            (IN_KEYWORD, TokenKind::InKeyword),
            (FUNCTION_KEYWORD, TokenKind::FunctionKeyword),
            (THIS_KEYWORD, TokenKind::ThisKeyword),
            (DO_KEYWORD, TokenKind::DoKeyword),
            (WHILE_KEYWORD, TokenKind::WhileKeyword),
            (TRY_KEYWORD, TokenKind::TryKeyword),
            (CATCH_KEYWORD, TokenKind::CatchKeyword),
            (BREAK_KEYWORD, TokenKind::BreakKeyword),
            (CONTINUE_KEYWORD, TokenKind::ContinueKeyword),
            (SUPER_KEYWORD, TokenKind::SuperKeyword),
            (THROW_KEYWORD, TokenKind::ThrowKeyword),
            (YIELD_KEYWORD, TokenKind::YieldKeyword),
            (EXPORT_KEYWORD, TokenKind::ExportKeyword),
            (IMPORT_KEYWORD, TokenKind::ImportKeyword),
            (RETURN_KEYWORD, TokenKind::ReturnKeyword),
            (STATIC_KEYWORD, TokenKind::StaticKeyword),
            (SWITCH_KEYWORD, TokenKind::SwitchKeyword),
            (TRUE_KEYWORD, TokenKind::Boolean("true".to_string())),
            (FALSE_KEYWORD, TokenKind::Boolean("false".to_string())),
            (NULL_KEYWORD, TokenKind::Null),
            (UNDEFINED_KEYWORD, TokenKind::Undefined),
        ]);

        let identifier = &self.source_code[self.current_pos..=cursor];

        if keywords.contains_key(identifier) {
            let token_kind = keywords.get(identifier).unwrap();
            self.current_pos += identifier.len();
            return Some(self.consume(token_kind.clone()));
        } else {
            self.current_pos += identifier.len();
            return Some(self.consume(TokenKind::Identifier(identifier.to_string())));
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

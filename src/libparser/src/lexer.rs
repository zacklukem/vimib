use crate::parse_context::ParseContext;
use crate::span::Span;
use std::str::Chars;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LiteralKind {
    Float,
    Int,
    String,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    Comment,
    Whitespace,

    Identifier,

    Literal(LiteralKind),

    /// Keywords
    Let,
    Fn,
    If,
    Else,
    Break,
    Loop,
    Return,

    /// Delimiter
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,

    /// Ops
    Dot,
    Star,
    Slash,
    Plus,
    Minus,
    Percent,
    Caret,
    And,
    Not,
    Or,
    Comma,
    Colon,
    Question,
    Equal,
    Lt,
    Gt,

    /// Double Equal
    EqEqual,
    LtEqual,
    GtEqual,
    AndAnd,
    OrOr,
    NotEqual,

    Semi,
    Eof,

    Unknown,
}

fn keyword(text: &str) -> Option<TokenKind> {
    match text {
        "let" => Some(TokenKind::Let),
        "fn" => Some(TokenKind::Fn),
        "if" => Some(TokenKind::If),
        "else" => Some(TokenKind::Else),
        "break" => Some(TokenKind::Break),
        "loop" => Some(TokenKind::Loop),
        "return" => Some(TokenKind::Return),
        _ => None,
    }
}

#[derive(Debug)]
pub struct TokenLen {
    pub kind: TokenKind,
    pub len: usize,
}

fn is_whitespace(c: char) -> bool {
    match c {
        | '\u{0009}' // \t
        | '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space

        // NEXT LINE from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
            => true,
        _ => false,
    }
}

fn is_ident_first(c: char) -> bool {
    match c {
        'A'..='Z' | 'a'..='z' | '_' => true,
        _ => false,
    }
}

fn is_ident(c: char) -> bool {
    match c {
        'A'..='Z' | 'a'..='z' | '_' | '0'..='9' => true,
        _ => false,
    }
}

#[derive(Clone)]
struct Tokenizer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &str) -> Tokenizer {
        Tokenizer { input, pos: 0 }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            return None;
        }
        let token_len = Cursor::new(self.input).next_token();
        self.pos += token_len.len;
        self.input = &self.input[token_len.len..];
        if token_len.kind == TokenKind::Whitespace || token_len.kind == TokenKind::Comment {
            self.next()
        } else {
            Some(Token {
                kind: token_len.kind,
                span: Span::new(self.pos - token_len.len, self.pos),
            })
        }
    }
}

struct Cursor<'a> {
    len: usize,
    chars: Chars<'a>,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Cursor {
        Cursor {
            chars: input.chars(),
            len: input.len(),
        }
    }

    pub fn peek(&self, n: usize) -> char {
        self.chars().nth(n).unwrap_or('\0')
    }

    #[allow(dead_code)]
    pub fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    pub fn chars(&self) -> Chars<'a> {
        self.chars.clone()
    }

    pub fn next(&mut self) -> Option<char> {
        self.chars.next()
    }

    pub fn len_consumed(&self) -> usize {
        self.len - self.chars.as_str().len()
    }

    pub fn next_token(&mut self) -> TokenLen {
        let first = self.next().unwrap();
        let kind = match first {
            // Whitespace
            c if is_whitespace(c) => {
                while is_whitespace(self.peek(0)) {
                    self.next();
                }
                TokenKind::Whitespace
            }

            // Comments (Block and Line)
            '/' => match self.peek(0) {
                '/' => {
                    self.next();
                    loop {
                        match self.peek(0) {
                            '\n' => break,
                            _ => {
                                self.next();
                            }
                        }
                    }
                    TokenKind::Comment
                }
                '*' => {
                    self.next();
                    loop {
                        match self.peek(0) {
                            '*' => match self.peek(1) {
                                '/' => break,
                                _ => {
                                    self.next();
                                }
                            },
                            _ => {
                                self.next();
                            }
                        }
                    }
                    self.next();
                    self.next();
                    TokenKind::Comment
                }

                _ => TokenKind::Slash,
            },

            // String literal
            '"' => {
                while self.peek(0) != '"' {
                    self.next();
                }
                self.next();
                TokenKind::Literal(LiteralKind::String)
            }

            // Number Literal
            '0'..='9' => {
                let mut has_dot = false;
                loop {
                    match self.peek(0) {
                        '.' => {
                            if let Some(c) = self.chars().nth(1) {
                                if is_ident_first(c) {
                                    break;
                                }
                            }
                            has_dot = true;
                            self.next();
                        }
                        '0'..='9' => {
                            self.next();
                        }
                        _ => break,
                    }
                }
                TokenKind::Literal(if has_dot {
                    LiteralKind::Float
                } else {
                    LiteralKind::Int
                })
            }

            // Identifiers
            c if is_ident_first(c) => {
                let mut buf = String::with_capacity(10);
                buf.push(c);
                while is_ident(self.peek(0)) {
                    if let Some(c) = self.next() {
                        buf.push(c);
                    }
                }
                match keyword(buf.as_str()) {
                    Some(text) => text,
                    None => TokenKind::Identifier,
                }
            }

            ';' => TokenKind::Semi,

            '(' => TokenKind::OpenParen,
            ')' => TokenKind::CloseParen,
            '{' => TokenKind::OpenBrace,
            '}' => TokenKind::CloseBrace,
            '[' => TokenKind::OpenBracket,
            ']' => TokenKind::CloseBracket,

            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            '?' => TokenKind::Question,
            ':' => TokenKind::Colon,
            '-' => TokenKind::Minus,
            '+' => TokenKind::Plus,
            '*' => TokenKind::Star,
            '^' => TokenKind::Caret,
            '%' => TokenKind::Percent,

            '!' => {
                if self.peek(0) == '=' {
                    self.next();
                    TokenKind::NotEqual
                } else {
                    TokenKind::Not
                }
            }
            '=' => {
                if self.peek(0) == '=' {
                    self.next();
                    TokenKind::EqEqual
                } else {
                    TokenKind::Equal
                }
            }
            '&' => {
                if self.peek(0) == '&' {
                    self.next();
                    TokenKind::AndAnd
                } else {
                    TokenKind::And
                }
            }
            '|' => {
                if self.peek(0) == '|' {
                    self.next();
                    TokenKind::OrOr
                } else {
                    TokenKind::Or
                }
            }
            '<' => {
                if self.peek(0) == '=' {
                    self.next();
                    TokenKind::LtEqual
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.peek(0) == '=' {
                    self.next();
                    TokenKind::GtEqual
                } else {
                    TokenKind::Gt
                }
            }

            '\0' => TokenKind::Eof,

            _ => TokenKind::Unknown,
        };
        TokenLen {
            kind,
            len: self.len_consumed(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

fn eof() -> Token {
    Token {
        kind: TokenKind::Eof,
        span: Span::dummy(),
    }
}

pub struct Lexer<'a> {
    tokens: Tokenizer<'a>,
    context: &'a ParseContext<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str, context: &'a ParseContext<'a>) -> Lexer<'a> {
        Lexer {
            tokens: Tokenizer::new(input),
            context,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Token {
        self.tokens.next().unwrap_or_else(eof)
    }

    pub fn peek(&self, n: usize) -> Token {
        self.tokens.clone().nth(n).unwrap_or_else(eof)
    }

    pub fn until(&mut self, kind: Vec<TokenKind>) -> Option<Token> {
        let peeked = self.peek(0);
        for k in kind.iter() {
            if peeked.kind == *k {
                return Some(self.next());
            }
        }
        None
    }

    pub fn expect(&mut self, kind: TokenKind, message: &str) -> Option<Token> {
        let peeked = self.peek(0);
        if peeked.kind == kind {
            Some(self.next())
        } else {
            self.context.error(peeked.span, message);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        use super::TokenKind::*;
        use crate::parse_context::ParseContext;

        static INPUT: &str = r#"
// Line Comment
/* Block Comment
 * Block Comment
 */
let ident = int;
-23.5
2512
23.hello
"Hello, World!"
{}
		"#;
        let ctx: ParseContext = ParseContext::new(INPUT);
        let mut lexer = Lexer::new(INPUT, &ctx);

        macro_rules! test_next {
            ($type: expr) => {
                assert_eq!(lexer.next().kind, $type);
            }; // ($type: expr, $val: expr) => {
               // 	{
               // 		let next = lexer.next();
               // 		assert_eq!(next.kind, $type);
               // 		// assert_eq!(next.val, $val);
               // 	}
               // };
        }

        test_next!(Let);
        test_next!(Identifier);
        test_next!(Equal);
        test_next!(Identifier);
        test_next!(Semi);
        test_next!(Minus);
        test_next!(Literal(LiteralKind::Float));
        test_next!(Literal(LiteralKind::Int));
        test_next!(Literal(LiteralKind::Int));
        test_next!(Dot);
        test_next!(Identifier);
        test_next!(Literal(LiteralKind::String));
        test_next!(OpenBrace);
        test_next!(CloseBrace);
    }
}

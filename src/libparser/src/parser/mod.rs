mod expression;
mod statement;

use crate::ast::*;
use crate::lexer::{Lexer, TokenKind};
use crate::parse_context::ParseContext;

pub struct Parser<'a> {
    context: &'a ParseContext<'a>,
    // input: &'a str,
    lexer: Lexer<'a>,
}

impl Parser<'_> {
    pub fn new<'a>(input: &'a str, context: &'a ParseContext<'a>) -> Parser<'a> {
        Parser {
            context,
            // input,
            lexer: Lexer::new(input, context),
        }
    }

    pub fn parse(&mut self) -> Block {
        self.parse_block()
    }

    pub fn parse_block(&mut self) -> Block {
        let mut body: Vec<Statement> = Vec::new();
        while let Some(stmt) = self.parse_statement() {
            body.push(stmt);
        }
        let next = self.lexer.peek(0);
        match next.kind {
            TokenKind::CloseBrace | TokenKind::Eof => {
                self.lexer.next();
            }
            _ => {
                let s = format!("Expected closing brace of EOF, found {:?}", next.kind);
                self.context.error(next.span, s.as_str())
            }
        }
        Block { body }
    }
}

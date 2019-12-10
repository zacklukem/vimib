mod expression;
mod statement;

use crate::ast::*;
use crate::lexer::{Lexer, TokenKind};
use crate::parse_context::ParseContext;

/// Parser class containing a context (for error printing) and lexer
pub struct Parser<'a> {
    context: &'a ParseContext<'a>,
    lexer: Lexer<'a>,
}

impl Parser<'_> {

    /// Create a new parser with the input and context given
    /// ```
    /// # use libparser::parser::*;
    /// # use libparser::parse_context::ParseContext;
    /// static INPUT: &str = "";
    /// let context = ParseContext::new(INPUT);
    /// let parser = Parser::new(INPUT, &context);
    /// ```
    pub fn new<'a>(input: &'a str, context: &'a ParseContext<'a>) -> Parser<'a> {
        Parser {
            context,
            // input,
            lexer: Lexer::new(input, context),
        }
    }

    /// Parse the input
    /// ```
    /// # use libparser::parser::*;
    /// # use libparser::parse_context::ParseContext;
    /// # use libparser::ast::*;
    /// static INPUT: &str = "";
    /// let context = ParseContext::new(INPUT);
    /// let mut parser = Parser::new(INPUT, &context);
    /// let block = parser.parse();
    /// ```
    pub fn parse(&mut self) -> Block {
        self.parse_block()
    }

    /// Parse a block
    /// ```
    /// # use libparser::parser::*;
    /// # use libparser::parse_context::ParseContext;
    /// # use libparser::ast::*;
    /// static INPUT: &str = "";
    /// let context = ParseContext::new(INPUT);
    /// let mut parser = Parser::new(INPUT, &context);
    /// let block = parser.parse();
    /// ```
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

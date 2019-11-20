use super::*;
use crate::ast::*;
use crate::lexer::TokenKind;

impl Parser<'_> {
    pub fn parse_expression(&mut self) -> Expression {
        self.equality()
    }

    fn equality(&mut self) -> Expression {
        let mut expr = self.comparison();

        while let Some(op) = self
            .lexer
            .until(vec![TokenKind::EqEqual, TokenKind::NotEqual])
        {
            let rhs = self.comparison();
            expr = Expression::Binary(Box::new(expr), Op::from(op.kind), Box::new(rhs));
        }

        expr
    }

    fn comparison(&mut self) -> Expression {
        let mut expr = self.addition();

        while let Some(op) = self.lexer.until(vec![
            TokenKind::Lt,
            TokenKind::Gt,
            TokenKind::LtEqual,
            TokenKind::GtEqual,
        ]) {
            let rhs = self.addition();
            expr = Expression::Binary(Box::new(expr), Op::from(op.kind), Box::new(rhs));
        }

        expr
    }

    fn addition(&mut self) -> Expression {
        let mut expr = self.multiplication();

        while let Some(op) = self.lexer.until(vec![TokenKind::Plus, TokenKind::Minus]) {
            let rhs = self.multiplication();
            expr = Expression::Binary(Box::new(expr), Op::from(op.kind), Box::new(rhs));
        }

        expr
    }

    fn multiplication(&mut self) -> Expression {
        let mut expr = self.unary();

        while let Some(op) = self.lexer.until(vec![TokenKind::Star, TokenKind::Slash]) {
            let rhs = self.multiplication();
            expr = Expression::Binary(Box::new(expr), Op::from(op.kind), Box::new(rhs));
        }

        expr
    }

    fn unary(&mut self) -> Expression {
        if let Some(op) = self.lexer.until(vec![TokenKind::Star, TokenKind::Slash]) {
            let rhs = self.multiplication();
            Expression::Unary(Op::from(op.kind), Box::new(rhs))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Expression {
        let next = self.lexer.peek(0);
        match next.kind {
            TokenKind::Literal(kind) => {
                self.lexer.next();
                let kind = LiteralKind::from(kind);
                Expression::Literal {
                    kind,
                    val: next.span,
                }
            }
            TokenKind::Identifier => {
                let peeked = self.lexer.peek(1);
                match peeked.kind {
                    TokenKind::OpenParen => self.parse_function_call(),
                    _ => {
                        self.lexer.next();
                        Expression::Ident { val: next.span }
                    }
                }
            }
            TokenKind::OpenParen => {
                self.lexer.next();
                let expr = self.parse_expression();
                self.lexer
                    .expect(TokenKind::CloseParen, "Expected Close Parentheses");
                expr
            }
            _ => {
                self.lexer.next();
                self.context.error(next.span, "Expected a value");
                Expression::Dummy
            }
        }
    }

    fn parse_function_call(&mut self) -> Expression {
        let next = self.lexer.next();
        let paren = self.lexer.next();
        if paren.kind == TokenKind::OpenParen {
            if self.lexer.peek(0).kind == TokenKind::CloseParen {
                self.lexer.next();
                Expression::FunctionCall(next.span, vec![])
            } else {
                // FIXME: ignore whitespace: print    (params)
                let mut args: Vec<Expression> = Vec::new();
                loop {
                    let expression = self.parse_expression();

                    args.push(expression);

                    let next = self.lexer.next();

                    match next.kind {
                        TokenKind::CloseParen => break,
                        TokenKind::Comma => continue,
                        _ => self
                            .context
                            .error(next.span, "Expected close paren or comma"),
                    }
                }
                Expression::FunctionCall(next.span, args)
            }
        } else {
            self.context
                .error(paren.span, "Missing parentheses in function call");
            Expression::Dummy
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_expression() {
        // FIXME: Make this test more comprehensive
        use crate::ast::{Expression, Op};
        use crate::parse_context::ParseContext;
        use crate::parser::Parser;

        static INPUT: &str = "5 + 3 * (3 + 2)";
        let ctx: ParseContext = ParseContext::new(INPUT);

        let mut parser = Parser::new(INPUT, &ctx);
        let expr = parser.parse_expression();
        match expr {
            Expression::Binary(_, op, _) => assert_eq!(op, Op::Plus),
            _ => assert!(false),
        }
    }
}

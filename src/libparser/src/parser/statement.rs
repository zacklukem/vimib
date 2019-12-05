use super::*;
use crate::ast::*;
use crate::lexer::TokenKind;

impl Parser<'_> {
    pub fn parse_statement(&mut self) -> Option<Statement> {
        let next = self.lexer.peek(0);
        match next.kind {
            TokenKind::Let => {
                self.lexer.next(); // let keyword
                let ident = self
                    .lexer
                    .expect(TokenKind::Identifier, "Expected identifier");
                let equal = self.lexer.expect(TokenKind::Equal, "Expected equal sign");
                let expr = self.parse_expression();
                if ident != None && equal != None {
                    Some(Statement::Assign(ident.unwrap().span, expr))
                } else {
                    Some(Statement::Dummy)
                }
            }
            TokenKind::Return => {
                let keyword = self.lexer.next(); // return keyword
                let expr = self.parse_expression();
                Some(Statement::Return(expr, keyword.span))
            }
            TokenKind::Fn => self.parse_function_decl(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::Loop => {
                self.lexer.next(); // loop keyword
                let open_brace = self
                    .lexer
                    .expect(TokenKind::OpenBrace, "Expected open brace");
                if open_brace == None {
                    return Some(Statement::Dummy);
                }
                Some(Statement::Loop(self.parse_block()))
            }
            TokenKind::Break => {
                self.lexer.next();
                Some(Statement::Break)
            }
            TokenKind::Identifier if self.lexer.peek(1).kind == TokenKind::Equal => {
                let var = self.lexer.next();
                self.lexer.next(); // = token
                let expr = self.parse_expression();
                Some(Statement::Mutate(var.span, expr))
            }
            TokenKind::Identifier | TokenKind::Literal(_) => {
                Some(Statement::Expression(self.parse_expression()))
            }
            _ => None,
        }
    }

    fn parse_type(&mut self) -> Type {
        let next = self.lexer.peek(0);
        let out = match next.kind {
            TokenKind::I32 => Type::Int,
            TokenKind::F32 => Type::Float,
            _ => {
                self.lexer.context.error(next.span, "Expected type");
                Type::Void
            }
        };
        self.lexer.next();
        out
    }

    fn parse_function_decl(&mut self) -> Option<Statement> {
        self.lexer.next(); // fn keyword
        let ident = self
            .lexer
            .expect(TokenKind::Identifier, "Expected Identifier");
        if let Some(ident) = ident {
            let open_paren = self
                .lexer
                .expect(TokenKind::OpenParen, "Expected open paren");
            if open_paren == None {
                return Some(Statement::Dummy);
            }
            let mut args = vec![];

            // Parse args
            loop {
                let peeked = self.lexer.peek(0);
                if peeked.kind == TokenKind::Identifier {
                    let ident = self.lexer.next(); // consume
                    let colon = self
                        .lexer
                        .expect(TokenKind::Colon, "Expected colon and type");
                    if colon == None {
                        return Some(Statement::Dummy);
                    }
                    let arg_type = self.parse_type();
                    args.push(Ident::Typed(ident.span, arg_type));
                    let peeked = self.lexer.peek(0);
                    if peeked.kind == TokenKind::Comma {
                        self.lexer.next(); // Comma
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            let close_paren = self
                .lexer
                .expect(TokenKind::CloseParen, "Expected close paren");
            if close_paren == None {
                return Some(Statement::Dummy);
            }

            let mut ret_type = Type::Void;

            let arrow = self.lexer.peek(0);

            if arrow.kind == TokenKind::Arrow {
                self.lexer.next();
                ret_type = self.parse_type();
            }

            let open_brace = self
                .lexer
                .expect(TokenKind::OpenBrace, "Expected open brace");
            if open_brace == None {
                return Some(Statement::Dummy);
            }

            let block = self.parse_block();

            Some(Statement::FnDecl {
                name: ident.span,
                return_type: ret_type,
                args,
                block,
            })
        } else {
            Some(Statement::Dummy)
        }
    }

    fn parse_if_statement(&mut self) -> Option<Statement> {
        self.lexer.next(); // if keyword
        let expr = self.parse_expression();
        let open_brace = self
            .lexer
            .expect(TokenKind::OpenBrace, "Expected open brace");
        if open_brace == None {
            return Some(Statement::Dummy);
        }
        let block = self.parse_block();
        let next = if self.lexer.peek(0).kind == TokenKind::Else {
            self.lexer.next(); // consume else
            if self.lexer.peek(0).kind == TokenKind::If {
                Some(Box::new(self.parse_if_statement()?))
            } else {
                let open_brace = self
                    .lexer
                    .expect(TokenKind::OpenBrace, "Expected open brace");
                if open_brace == None {
                    return Some(Statement::Dummy);
                }
                let block = self.parse_block();
                Some(Box::new(Statement::Else(block)))
            }
        } else {
            None
        };
        Some(Statement::If(expr, block, next))
    }
}

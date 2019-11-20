use crate::lexer::{self, TokenKind};
use crate::span::Span;

#[derive(Debug)]
pub enum LiteralKind {
    String,
    Int,
    Float,
}

impl From<lexer::LiteralKind> for LiteralKind {
    fn from(kind: lexer::LiteralKind) -> LiteralKind {
        match kind {
            lexer::LiteralKind::Float => LiteralKind::Float,
            lexer::LiteralKind::Int => LiteralKind::Int,
            lexer::LiteralKind::String => LiteralKind::String,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Op {
    Star,
    Slash,
    Plus,
    Minus,
    Mod,
    Eq,
    NotEq,
    LtEq,
    GtEq,
    Lt,
    Gt,
    Not,
}

impl From<TokenKind> for Op {
    fn from(token_kind: TokenKind) -> Op {
        match token_kind {
            TokenKind::Star => Op::Star,
            TokenKind::Slash => Op::Slash,
            TokenKind::Plus => Op::Plus,
            TokenKind::Minus => Op::Minus,
            TokenKind::Percent => Op::Mod,
            TokenKind::EqEqual => Op::Eq,
            TokenKind::NotEqual => Op::NotEq,
            TokenKind::LtEqual => Op::LtEq,
            TokenKind::GtEqual => Op::GtEq,
            TokenKind::Lt => Op::Lt,
            TokenKind::Gt => Op::Gt,
            TokenKind::Not => Op::Not,
            _ => panic!("Help me!"),
        }
    }
}

#[derive(Debug)]
pub enum Expression {
    Literal { val: Span, kind: LiteralKind },
    Binary(Box<Expression>, Op, Box<Expression>),
    Unary(Op, Box<Expression>),
    Ident { val: Span },
    FunctionCall(Span, Vec<Expression>),
    Dummy,
}

#[derive(Debug)]
pub enum Type {
    Str,
    Int,
    Float,
    Void,
}

#[derive(Debug)]
pub enum Ident {
    Typed(Span, Type),
    Untyped(Span),
}

#[derive(Debug)]
pub enum Statement {
    Assign(Span, Expression),
    FnDecl {
        name: Span,
        return_type: Type,
        args: Vec<Ident>,
        block: Block,
    },
    Mutate(Span, Expression),
    If(Expression, Block, Option<Box<Statement>>),
    Else(Block),
    Loop(Block),
    Break,
    Expression(Expression),
    Dummy,
}

#[derive(Debug)]
pub struct Block {
    pub body: Vec<Statement>,
}

use crate::lexer::{self, TokenKind};
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum LiteralKind {
    String,
    Int,
    Float,
}

impl From<lexer::LiteralKind> for LiteralKind {
    /// Converts from lexer literal kind to ast literal kind
    fn from(kind: lexer::LiteralKind) -> LiteralKind {
        match kind {
            lexer::LiteralKind::Float => LiteralKind::Float,
            lexer::LiteralKind::Int => LiteralKind::Int,
            lexer::LiteralKind::String => LiteralKind::String,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
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
    /// Converts from lexer token kind to ast op. Panics if token kind is not an op.
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
            _ => panic!("Not an operator"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal { val: Span, kind: LiteralKind },
    Binary(Box<Expression>, Op, Box<Expression>, Span),
    Unary(Op, Box<Expression>, Span),
    Ident { val: Span },
    FunctionCall(Span, Vec<Expression>),
    Dummy,
}

#[derive(Debug, Clone)]
pub enum Type {
    Str,
    Int,
    Float,
    Void,
}

#[derive(Debug, Clone)]
pub enum Ident {
    Typed(Span, Type),
    Untyped(Span),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assign(Span, Expression),
    FnDecl {
        name: Span,
        return_type: Type,
        args: Vec<Ident>,
        block: Block,
    },
    Return(Expression, Span),
    Mutate(Span, Expression),
    If(Expression, Block, Option<Box<Statement>>),
    Else(Block),
    Loop(Block),
    Break,
    Expression(Expression),
    Dummy,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub body: Vec<Statement>,
}

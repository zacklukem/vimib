use libparser::ast::*;
use libvm::consts::*;
use std::collections::HashMap;

pub struct OpcodeGenerator<'a> {
    input: &'a str,
    var_map: HashMap<String, u8>,
    var_index: u8,
    break_me: Vec<usize>,
    out: Vec<u8>
}

impl OpcodeGenerator<'_> {
    pub fn new(input: &str) -> OpcodeGenerator {
        OpcodeGenerator {
            input,
            var_map: HashMap::new(),
            var_index: 0,
            break_me: Vec::new(),
            out: Vec::new()
        }
    }
    fn to_str(&self, span: &libparser::span::Span) -> String {
        String::from(&self.input[span.pos.0..span.pos.1])
    }

    pub fn gen(&self) -> &[u8] {
        self.out.as_slice()
    }

    pub fn gen_block(&mut self, block: &Block) {
        for stmt in block.body.iter() {
            match stmt {
                Statement::Expression(expr) => self.gen_expr(expr),
                Statement::Assign(name, expr) => {
                    self.gen_expr(expr);
                    let name = self.to_str(name);
                    
                    self.out.push(STO_I);
                    if let Some(index) = self.var_map.get(&name) {
                        self.out.push(*index);
                    } else {
                        self.var_map.insert(name, self.var_index);
                        self.out.push(self.var_index);
                        self.var_index += 4;// FIXME: Detect Type
                    }
                }
                Statement::Mutate(name, expr) => {
                    self.gen_expr(expr);
                    let name = self.to_str(name);
                    
                    self.out.push(STO_I);
                    if let Some(index) = self.var_map.get(&name) {
                        self.out.push(*index);
                    } else {
                        panic!("Variable is undefined");
                    }
                }
                Statement::If(expr, block, _next) => {
                    self.gen_expr(expr);
                    self.out.push(IF_F);
                    let set_me = self.out.len();
                    self.out.push(0);
                    
                    self.gen_block(block);
                    *self.out.get_mut(set_me).unwrap() = self.out.len() as u8;
                }
                Statement::Loop(block) => {
                    let start = self.out.len();
                    self.gen_block(block);
                    self.out.push(GOTO);
                    self.out.push(start as u8);
                    let end = self.out.len() + 1;
                    for i in self.break_me.iter() {
                        *self.out.get_mut(*i).unwrap() = end as u8;
                    }
                    self.break_me.clear();
                }
                Statement::Break => {
                    self.out.push(GOTO);
                    self.out.push(0);
                    self.break_me.push(self.out.len()-1);
                }
                _ => unimplemented!(),
            }
        }
    }

    pub fn gen_expr(&mut self, expr: &Expression) {

        match expr {
            Expression::Binary(lhs, op, rhs) => {
                self.gen_expr(lhs);
                self.gen_expr(rhs);
                self.out.push(match op {
                    Op::Plus => ADD_I,
                    Op::Minus => SUB_I,
                    Op::Star => MUL_I,
                    Op::Slash => DIV_I,
                    Op::Mod => MOD_I,
                    Op::Eq => EQ,
                    Op::NotEq => NE,
                    Op::Lt => LT,
                    Op::Gt => GT,
                    Op::LtEq => LE,
                    Op::GtEq => GE,
                    _ => unimplemented!()
                });
            }
            Expression::FunctionCall(ident, exprs) => {
                if self.to_str(ident).as_str() == "println" {
                    self.gen_expr(exprs.get(0).unwrap());
                    self.out.push(VIRTUAL);
                    self.out.push(0);
                }
            }
            Expression::Ident {val} => {
                let ident = self.to_str(val);
                self.out.push(LOAD_I);
                if let Some(index) = self.var_map.get(&String::from(ident)) {
                    self.out.push(*index);
                }
            }
            Expression::Literal { val, .. } => {
                self.out.push(PUSH_I);
                let num = self.to_str(val);
                let num = num.parse::<i32>().unwrap(); // TODO: Match literal kind
                let x = num as u32;
                let b1: u8 = ((x >> 24) & 0xff) as u8;
                let b2: u8 = ((x >> 16) & 0xff) as u8;
                let b3: u8 = ((x >> 8) & 0xff) as u8;
                let b4: u8 = (x & 0xff) as u8;
                self.out.push(b1);
                self.out.push(b2);
                self.out.push(b3);
                self.out.push(b4);
            }
            _ => unimplemented!(),
        }
    }
}

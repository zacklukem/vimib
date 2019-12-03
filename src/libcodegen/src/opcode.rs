use libparser::ast::*;
use libvm::consts::*;
use libvm::function::Function;
use libvm::module::Module;
use libvm::vm_type;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct OpcodeGenerator<'a> {
    input: &'a str,
    var_map: HashMap<String, u8>,
    var_index: u8,
    break_me: Vec<usize>,
    out: Vec<u8>,
    module: Rc<RefCell<Module>>,
    functions: HashMap<String, usize>,
}

impl OpcodeGenerator<'_> {
    /// Creates a new Opcode Generator
    /// ```
    /// # use libcodegen::opcode::*;
    /// # static INPUT: &str = "";
    /// let gen = OpcodeGenerator::new(INPUT);
    /// ```
    pub fn new(input: &str) -> OpcodeGenerator {
        OpcodeGenerator {
            input,
            var_map: HashMap::new(),
            var_index: 0,
            break_me: Vec::new(),
            out: Vec::new(),
            module: Rc::new(RefCell::new(Default::default())),
            functions: HashMap::new(),
        }
    }

    fn to_str(&self, span: &libparser::span::Span) -> String {
        String::from(&self.input[span.pos.0..span.pos.1])
    }
    /// Clones the generated module and returns a reference to it.
    /// ```
    /// # use libcodegen::opcode::*;
    /// # static INPUT: &str = "";
    /// let gen = OpcodeGenerator::new(INPUT);
    /// // gen.gen_module(), etc
    /// let module = gen.gen();
    /// println!("{:?}", module);
    /// ```
    pub fn gen(&self) -> Rc<RefCell<Module>> {
        Rc::clone(&self.module)
    }

    /// Get current output buffer
    pub fn out(&self) -> Vec<u8> {
        self.out.clone()
    }

    /// Generates a module
    /// ```
    /// # use libcodegen::opcode::*;
    /// # use libparser::*;
    ///  
    /// static INPUT: &str = r"
    ///     fn a() {
    ///         
    ///     }
    /// ";
    /// let parse_context = libparser::parse_context::ParseContext::new(INPUT);
    /// let mut parser = libparser::parser::Parser::new(INPUT, &parse_context);
    /// let mut gen = OpcodeGenerator::new(INPUT);
    ///
    /// gen.gen_module(&parser.parse_block());
    ///
    /// let module = gen.gen();
    /// let module = module.borrow();
    /// let func = module.functions().get(&0);
    /// assert_ne!(func, None)
    /// ```
    pub fn gen_module(&mut self, block: &Block) {
        for stmt in block.body.iter() {
            match stmt {
                Statement::FnDecl {
                    name, block, args, ..
                } => {
                    let name = self.to_str(name);
                    if let Some(_func) = self.functions.get(&name) {
                        panic!("Function already exists")
                    } else {
                        let index = self.module.borrow_mut().new_const(name.clone().as_str());
                        self.functions.insert(name.clone(), index);
                        let args: Vec<vm_type::Type> = args
                            .iter()
                            .map(|v| match *v {
                                Ident::Untyped(span) => {
                                    self.var_map.insert(self.to_str(&span), self.var_index);
                                    self.var_index += 4;
                                    vm_type::Type::I32
                                }
                                _ => unimplemented!(),
                            })
                            .collect();
                        self.gen_block(block);
                        let instructions = self.out.clone();
                        self.reset();
                        let func = Function::new(instructions, args, Rc::clone(&self.module));
                        self.module.borrow_mut().push_fn(index, func);
                    }
                }
                _ => panic!("Only function decls in root block"), // TODO: fix this msg
            }
        }
    }

    /// Reset after generating a function
    fn reset(&mut self) {
        self.out.clear();
        self.break_me.clear();
        self.var_map.clear();
        self.var_index = 0;
    }

    /// Generate a block (inside a function)
    /// ```
    /// # use libcodegen::opcode::*;
    /// # use libparser::*;
    /// # use libvm::consts::*;
    ///  
    /// static INPUT: &str = r"
    ///     let a = 2
    ///     print_int(a)
    /// ";
    /// let parse_context = libparser::parse_context::ParseContext::new(INPUT);
    /// let mut parser = libparser::parser::Parser::new(INPUT, &parse_context);
    /// let mut gen = OpcodeGenerator::new(INPUT);
    ///
    /// gen.gen_block(&parser.parse_block());
    ///
    /// let out = gen.out();
    /// assert_eq!(out, vec![
    ///     PUSH_I, 0, 0, 0, 2,
    ///     STO_I, 0,
    ///     LOAD_I, 0,
    ///     VIRTUAL, 0
    /// ])
    /// ```
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
                        self.var_index += 4; // FIXME: Detect Type
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
                    let end = self.out.len();
                    for i in self.break_me.iter() {
                        *self.out.get_mut(*i).unwrap() = end as u8;
                    }
                    self.break_me.clear();
                }
                Statement::Return(expr) => {
                    self.gen_expr(expr);
                    self.out.push(RET_I);
                }
                Statement::Break => {
                    self.out.push(GOTO);
                    self.out.push(0);
                    self.break_me.push(self.out.len() - 1);
                }
                _ => unimplemented!(),
            }
        }
    }

    /// Generate an expression (inside a block)
    /// ```
    /// # use libcodegen::opcode::*;
    /// # use libparser::*;
    /// # use libparser::ast::*;
    /// # use libvm::consts::*;
    ///  
    /// static INPUT: &str = r"
    ///     5 + 5
    /// ";
    /// let parse_context = libparser::parse_context::ParseContext::new(INPUT);
    /// let mut parser = libparser::parser::Parser::new(INPUT, &parse_context);
    /// let mut gen = OpcodeGenerator::new(INPUT);
    ///
    /// if let Statement::Expression(expr) = parser.parse_block().body.get(0).unwrap() {
    ///     gen.gen_expr(&expr);
    /// } else {
    ///     panic!("Should have expression")
    /// }
    ///
    /// let out = gen.out();
    /// assert_eq!(out, vec![
    ///     PUSH_I, 0, 0, 0, 5,
    ///     PUSH_I, 0, 0, 0, 5,
    ///     ADD_I
    /// ])
    /// ```
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
                    _ => unimplemented!(),
                });
            }
            Expression::FunctionCall(ident, exprs) => match self.to_str(ident).as_str() {
                "print_int" => {
                    self.gen_expr(exprs.get(0).unwrap());
                    self.out.push(VIRTUAL);
                    self.out.push(0);
                }
                "print_str" => {
                    self.gen_expr(exprs.get(0).unwrap());
                    self.out.push(VIRTUAL);
                    self.out.push(2);
                }
                ident => {
                    for expr in exprs.iter() {
                        self.gen_expr(expr);
                    }
                    if let Some(index) = self.functions.get(ident) {
                        self.out.push(CALL);
                        self.out.push(*index as u8);
                    } else {
                        panic!("Unknown function: {}", ident); // TODO: Fix this message
                    }
                }
            },
            Expression::Ident { val } => {
                let ident = self.to_str(val);
                self.out.push(LOAD_I);
                if let Some(index) = self.var_map.get(&ident) {
                    self.out.push(*index);
                }
            }
            Expression::Literal { val, kind } => {
                match *kind {
                    LiteralKind::Int => {
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
                    LiteralKind::String => {
                        let val = self.to_str(val);
                        let c_index = self.module.borrow_mut().new_const(&val[1..val.len() - 1]);
                        self.out.push(LDC);
                        self.out.push(c_index as u8);
                    }
                    LiteralKind::Float => unimplemented!(),
                }
            }
            _ => unimplemented!(),
        }
    }
}

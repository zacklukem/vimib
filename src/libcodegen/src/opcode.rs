use libparser::ast::*;
use libparser::parse_context::ParseContext;
use libvm::consts::*;
use libvm::function::Function;
use libvm::module::Module;
use libvm::vm_type;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct OpcodeGenerator<'a> {
    input: &'a str,
    var_map: HashMap<String, (u8, vm_type::Type)>,
    var_index: u8,
    break_me: Vec<usize>,
    out: Vec<u8>,
    module: Rc<RefCell<Module>>,
    functions: HashMap<String, (usize, Statement)>,
    context: ParseContext<'a>,
}

fn ast_type_to_vm_type(t: &Type) -> vm_type::Type {
    match t {
        Type::Int => vm_type::Type::I32,
        Type::Float => vm_type::Type::F32,
        Type::Void => vm_type::Type::Void,
        Type::Str => vm_type::Type::String,
    }
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
            context: ParseContext::new(input),
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
                    name,
                    block,
                    args,
                    return_type,
                } => {
                    let span = name;
                    let name = self.to_str(span);
                    if let Some(_func) = self.functions.get(&name) {
                        self.context.error(*span, "Function already exists");
                        panic!()
                    } else {
                        let index = self.module.borrow_mut().new_const(name.clone().as_str());
                        self.functions.insert(name.clone(), (index, stmt.clone()));
                        let args: Vec<vm_type::Type> = args
                            .iter()
                            .map(|v| match v {
                                Ident::Typed(span, arg_type) => {
                                    let arg_type = ast_type_to_vm_type(&arg_type);
                                    self.var_map.insert(
                                        self.to_str(&span),
                                        (self.var_index, arg_type.clone()),
                                    );
                                    self.var_index += 4;
                                    arg_type
                                }
                                _ => unimplemented!(),
                            })
                            .collect();
                        self.gen_block(block, ast_type_to_vm_type(return_type));
                        let instructions = self.out.clone();
                        self.reset();
                        let func = Function::new(
                            instructions,
                            args,
                            ast_type_to_vm_type(return_type),
                            Rc::clone(&self.module),
                        );
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
    /// gen.gen_block(&parser.parse_block(), Default::default());
    ///
    /// let out = gen.out();
    /// assert_eq!(out, vec![
    ///     PUSH_I, 0, 0, 0, 2,
    ///     STO_I, 0,
    ///     LOAD_I, 0,
    ///     VIRTUAL, 0
    /// ])
    /// ```
    pub fn gen_block(&mut self, block: &Block, return_type: vm_type::Type) {
        for stmt in block.body.iter() {
            match stmt {
                Statement::Expression(expr) => {
                    self.gen_expr(expr);
                }
                Statement::Assign(name, expr) => {
                    let var_type = self.gen_expr(expr);
                    let name = self.to_str(name);

                    self.out.push(STO_I);
                    if let Some((index, _)) = self.var_map.get(&name) {
                        self.out.push(*index);
                    } else {
                        self.var_map.insert(name, (self.var_index, var_type));
                        self.out.push(self.var_index);
                        self.var_index += 4; // FIXME: Detect Type
                    }
                }
                Statement::Mutate(name, expr) => {
                    self.gen_expr(expr);
                    let span = name;
                    let name = self.to_str(span);

                    self.out.push(STO_I);
                    if let Some((index, _)) = self.var_map.get(&name) {
                        self.out.push(*index);
                    } else {
                        self.context.error(*span, "Variable is undefined");
                        panic!();
                    }
                }
                Statement::If(expr, block, _next) => {
                    self.gen_expr(expr);
                    self.out.push(IF_F);
                    let set_me = self.out.len();
                    self.out.push(0);

                    self.gen_block(block, return_type.clone());
                    *self.out.get_mut(set_me).unwrap() = self.out.len() as u8;
                }
                Statement::Loop(block) => {
                    let start = self.out.len();
                    self.gen_block(block, return_type.clone());
                    self.out.push(GOTO);
                    self.out.push(start as u8);
                    let end = self.out.len();
                    for i in self.break_me.iter() {
                        *self.out.get_mut(*i).unwrap() = end as u8;
                    }
                    self.break_me.clear();
                }
                Statement::Return(expr, span) => {
                    let expr_type = self.gen_expr(expr);
                    self.out.push(RET_I);
                    if expr_type != return_type {
                        self.context.error(
                            *span,
                            format!("Expected {:?} found {:?}", return_type, expr_type).as_str(),
                        );
                        panic!()
                    }
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
    pub fn gen_expr(&mut self, expr: &Expression) -> vm_type::Type {
        match expr {
            Expression::Binary(lhs, op, rhs, span) => {
                let lhs = self.gen_expr(lhs);
                let rhs = self.gen_expr(rhs);

                if lhs != rhs {
                    self.context.error(
                        *span,
                        format!("{:?} is not compatible with {:?}", lhs, rhs).as_str(),
                    );
                    panic!()
                }

                self.out.push(match op {
                    Op::Plus if lhs == vm_type::Type::F32 => ADD_F,
                    Op::Minus if lhs == vm_type::Type::F32 => SUB_F,
                    Op::Star if lhs == vm_type::Type::F32 => MUL_F,
                    Op::Slash if lhs == vm_type::Type::F32 => DIV_F,
                    Op::Mod if lhs == vm_type::Type::F32 => MOD_F,
                    Op::Lt if lhs == vm_type::Type::F32 => LT_F,
                    Op::Gt if lhs == vm_type::Type::F32 => GT_F,
                    Op::LtEq if lhs == vm_type::Type::F32 => LE_F,
                    Op::GtEq if lhs == vm_type::Type::F32 => GE_F,
                    Op::Lt => LT_I,
                    Op::Gt => GT_I,
                    Op::LtEq => LE_I,
                    Op::GtEq => GE_I,
                    Op::Plus => ADD_I,
                    Op::Minus => SUB_I,
                    Op::Star => MUL_I,
                    Op::Slash => DIV_I,
                    Op::Mod => MOD_I,
                    Op::Eq => EQ,
                    Op::NotEq => NE,
                    _ => unimplemented!(),
                });
                lhs
            }
            Expression::FunctionCall(ident_span, exprs) => match self.to_str(ident_span).as_str() {
                "print_int" => {
                    self.gen_expr(exprs.get(0).unwrap());
                    self.out.push(VIRTUAL);
                    self.out.push(0);
                    vm_type::Type::Void
                }
                "debug" => {
                    self.out.push(VIRTUAL);
                    self.out.push(1);
                    vm_type::Type::Void
                }
                "print_float" => {
                    self.gen_expr(exprs.get(0).unwrap());
                    self.out.push(VIRTUAL);
                    self.out.push(3);
                    vm_type::Type::Void
                }
                "print_str" => {
                    self.gen_expr(exprs.get(0).unwrap());
                    self.out.push(VIRTUAL);
                    self.out.push(2);
                    vm_type::Type::Void
                }
                ident => {
                    for expr in exprs.iter() {
                        self.gen_expr(expr);
                    }
                    if let Some((index, stmt)) = self.functions.get(ident) {
                        self.out.push(CALL);
                        self.out.push(*index as u8);
                        if let Statement::FnDecl { return_type, .. } = stmt {
                            ast_type_to_vm_type(return_type)
                        } else {
                            vm_type::Type::Void
                        }
                    } else {
                        self.context.error(*ident_span, "Unknown function");
                        panic!() // TODO: Fix this message
                    }
                }
            },
            Expression::Ident { val } => {
                let ident = self.to_str(val);
                self.out.push(LOAD_I);
                if let Some((index, var_type)) = self.var_map.get(&ident) {
                    self.out.push(*index);
                    var_type.clone()
                } else {
                    self.context.error(*val, "Variable doesn't exist");
                    panic!()
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
                        vm_type::Type::I32
                    }
                    LiteralKind::String => {
                        let val = self.to_str(val);
                        let c_index = self.module.borrow_mut().new_const(&val[1..val.len() - 1]);
                        self.out.push(LDC);
                        self.out.push(c_index as u8);
                        vm_type::Type::String
                    }
                    LiteralKind::Float => {
                        self.out.push(PUSH_I);
                        let num = self.to_str(val);
                        let num = num.parse::<f32>().unwrap(); // TODO: Match literal kind
                        unsafe {
                            // TODO: Perhaps split this kind of thing into a separate utility library
                            let x = std::mem::transmute::<f32, [u8; 4]>(num);
                            self.out.push(x[0]);
                            self.out.push(x[1]);
                            self.out.push(x[2]);
                            self.out.push(x[3]);
                        }
                        vm_type::Type::F32
                    }
                }
            }
            Expression::Unary(op, expr, span) => {
                let expr = self.gen_expr(expr);
                let instruction = match *op {
                    Op::Minus => NEG_I,
                    Op::Not => NOT,
                    _ => {
                        self.context
                            .error(*span, "Only '-' or '!' in unary expressions");
                        panic!()
                    }
                };
                self.out.push(instruction);
                expr
            }
            Expression::Dummy => panic!(),
        }
    }
}

use crate::consts::*;
use crate::module::Module;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::io;
use std::io::Cursor;
use std::rc::Rc;

/// A stack based interpreted virtual machine with registers
pub struct Vm<'a> {
    program: &'a [u8],
    index: usize,
    regs: Vec<u8>,
    stack: Vec<u8>,
    module: Rc<RefCell<Module>>,
    is_debug: bool,
}

impl Vm<'_> {
    /// Create a new vm with an empty state.  Regs are initialized with the
    /// values passed in the `regs` argument.  These are used to initialize
    /// parameter variables.
    /// ```
    /// # use libvm::vm::Vm;
    /// let vm = Vm::new(&[], Vec::new(), Default::default());
    /// ```
    pub fn new(program: &[u8], regs: Vec<u8>, module: Rc<RefCell<Module>>) -> Vm {
        Vm {
            program,
            index: 0,
            regs,
            stack: Vec::new(),
            module,
            is_debug: std::env::var("VIMIB_DEBUG").is_ok(),
        }
    }

    /// Goto the next instruction / byte
    fn next(&mut self) -> u8 {
        let ret = self.program[self.index];
        self.index += 1;
        ret
    }

    /// Get the current instruction / byte
    fn current(&self) -> u8 {
        self.program[self.index]
    }

    /// Consumes 4 bytes of instructions
    fn next_int(&mut self) -> [u8; 4] {
        let mut out = [self.next(), self.next(), self.next(), self.next()];
        out.reverse();
        out
    }

    /// Push a byte onto the stack
    fn push(&mut self, v: u8) {
        self.stack.push(v)
    }

    /// Push a 32 bit number as 4 bytes onto the stack
    fn push_32(&mut self, v: [u8; 4]) {
        for i in v.iter() {
            self.stack.push(*i);
        }
    }

    /// Push an f32 onto the stack
    fn push_f32(&mut self, v: f32) {
        let mut a = vec![];
        a.write_f32::<BigEndian>(v).unwrap();
        self.stack.extend(a.iter());
    }

    /// Push an i32 onto the stack
    fn push_i32(&mut self, v: i32) {
        let mut a = vec![];
        a.write_i32::<LittleEndian>(v).unwrap();
        self.stack.extend(a.iter());
    }

    /// Pop a byte from the stack
    fn pop(&mut self) -> u8 {
        self.stack.pop().unwrap()
    }

    /// Pop 4 bytes off the stack
    fn pop_32(&mut self) -> [u8; 4] {
        let mut out = [self.pop(), self.pop(), self.pop(), self.pop()];
        out.reverse();
        out
    }

    /// Pop an 32 bit num in the form of an f32 off the stack
    fn pop_f32(&mut self) -> f32 {
        let array = self.pop_32();
        let mut rdr = Cursor::new(Vec::from(&array as &[u8]));
        rdr.read_f32::<BigEndian>().unwrap()
    }

    /// Pop an int in the form of an i32 off the stack
    fn pop_i32(&mut self) -> i32 {
        let array = self.pop_32();
        let mut rdr = Cursor::new(Vec::from(&array as &[u8]));
        rdr.read_i32::<LittleEndian>().unwrap()
    }

    /// Get an int in the form of an array from the stack
    fn get_int(&self) -> [u8; 4] {
        let mut out = [0; 4];
        for (i, v) in out.iter_mut().enumerate() {
            *v = *self.stack.get(self.stack.len() - i - 1).unwrap();
        }
        out.reverse();
        out
    }

    /// Run the program and return a vector of bytes containing a returned
    /// value
    /// ```
    /// # use libvm::vm::Vm;
    /// # use libvm::consts::*;
    /// let program = &[
    ///     PUSH_I, 0, 0, 0, 5,
    ///     PUSH_I, 0, 0, 0, 6,
    ///     ADD_I,
    ///     RET_I
    /// ];
    /// let mut vm = Vm::new(program, Vec::new(), Default::default());
    /// let out = vm.run();
    /// assert_eq!(out, vec![11, 0, 0, 0]);
    /// ```
    pub fn run(&mut self) -> Vec<u8> {
        while self.index < self.program.len() {
            if let Some(ret) = self.execute() {
                return ret;
            }
        }
        vec![]
    }

    #[allow(clippy::cognitive_complexity)] // TODO: split this function up
    fn execute(&mut self) -> Option<Vec<u8>> {
        macro_rules! ordering {
            ($a: expr) => {{
                let location = self.next();
                let v = self.pop();
                if v == $a {
                    self.index = location as usize;
                }
            }};
            ($a: expr, $b: expr) => {{
                let location = self.next();
                let v = self.pop();
                if v == $a || v == $b {
                    self.index = location as usize;
                }
            }};
        }
        macro_rules! binary_operator {
			(i$op: tt) => {
				{
					let rhs = self.pop_i32();
					let lhs = self.pop_i32();
					self.push_i32(lhs $op rhs);
				}
            };
            (f$op: tt) => {
				{
					let rhs = self.pop_f32();
                    let lhs = self.pop_f32();
					self.push_f32(lhs $op rhs);
				}
            };
			(ib$op: tt) => {
				{
					let rhs = self.pop_i32();
					let lhs = self.pop_i32();
					self.push((lhs $op rhs) as u8);
				}
            };
			(fb$op: tt) => {
				{
					let rhs = self.pop_f32();
					let lhs = self.pop_f32();
					self.push((lhs $op rhs) as u8);
				}
			};
        }

        if self.is_debug {
            let mut out = String::new();
            let mut program = self.program.iter().enumerate();
            macro_rules! push_n {
                ($n: expr) => {
                    for _ in 0..$n {
                        out.push(' ');
                        out.push_str(&program.next().unwrap().1.to_string());
                    }
                };
            }
            if let Some((i, v)) = program.nth(self.index) {
                let i_str = i.to_string();
                out.push_str("\u{001b}[33m"); // red
                out.push_str(&i_str);
                out.push_str(": ");
                out.push_str("\u{001b}[0m"); // reset
                for _ in 0..(3 - i_str.len()) {
                    out.push(' ');
                }
                out.push_str("\u{001b}[31m"); // blue
                let in_str = disassemble_each(*v).unwrap();
                out.push_str(in_str);
                for _ in 0..(8 - in_str.len()) {
                    out.push(' ');
                }
                out.push_str("\u{001b}[0m"); // reset
                match *v {
                    PUSH_I => push_n!(4),
                    VIRTUAL
                    | GOTO
                    | STO_I
                    | LOAD_I
                    | STO_V
                    | LOAD_V
                    | LDC
                    | CALL
                    | IF_T..=IF_GE => push_n!(1),

                    _ => {}
                }
                out.push('\n');
            }
            println!("{}", out);
            println!("STACK: {:?}", self.stack);
            println!("REGS:  {:?}", self.regs);
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Couldn't read line");
        }
        match self.next() {
            PUSH_I => {
                let val = self.next_int();
                self.push_32(val);
            }
            ADD_I => binary_operator!(i+),
            SUB_I => binary_operator!(i-),
            MUL_I => binary_operator!(i*),
            DIV_I => binary_operator!(i/),
            MOD_I => binary_operator!(i%),
            ADD_F => binary_operator!(f+),
            SUB_F => binary_operator!(f-),
            MUL_F => binary_operator!(f*),
            DIV_F => binary_operator!(f/),
            MOD_F => binary_operator!(f%),

            NEG_I => {
                let n = self.pop_i32();
                self.push_i32(-n);
            }

            NOT => {
                let n = self.pop() != 0;
                self.push((!n) as u8);
            }

            NE => binary_operator!(ib!=),
            EQ => binary_operator!(ib==),
            GT_I => binary_operator!(ib>),
            LT_I => binary_operator!(ib<),
            GE_I => binary_operator!(ib>=),
            LE_I => binary_operator!(ib<=),
            GT_F => binary_operator!(fb>),
            LT_F => binary_operator!(fb<),
            GE_F => binary_operator!(fb>=),
            LE_F => binary_operator!(fb<=),

            DUP_I => {
                self.push_32(self.get_int());
            }
            GOTO => {
                let location = self.next();
                self.index = location as usize;
            }
            STO_I => {
                let reg = self.next() as usize;
                let val = self.pop_32();
                if self.regs.len() <= reg + 3 {
                    for v in val.iter() {
                        self.regs.push(*v);
                    }
                } else {
                    for (i, v) in val.iter().enumerate() {
                        *self.regs.get_mut(reg + i).unwrap() = *v;
                    }
                }
            }
            LOAD_I => {
                let reg = self.next() as usize;
                for i in 0..4 {
                    self.push(self.regs[reg + i]);
                }
            }
            STO_V => {
                let reg = self.next() as usize;
                let len = self.pop() as usize;
                if self.regs.len() <= reg + len + 1 {
                    self.regs.push(len as u8);
                    for _ in 0..len {
                        let v = self.pop();
                        self.regs.push(v);
                    }
                } else {
                    *self.regs.get_mut(reg).unwrap() = len as u8;
                    for i in 0..len {
                        let v = self.pop();
                        *self.regs.get_mut(reg + i + 1).unwrap() = v;
                    }
                }
            }
            LOAD_V => {
                let reg = self.next() as usize;
                let len = self.regs[reg] as usize;
                for i in 0..=len {
                    self.push(self.regs[reg + len - i]);
                }
            }
            CALL => {
                let index = self.next() as usize;
                let ret = self.module.borrow().call(index, &mut self.stack);
                self.stack.extend(ret.iter());
            }
            VIRTUAL => {
                let call = self.next();
                match call {
                    0x00 => println!("{}", self.pop_i32()),
                    0x01 => println!("STACK: {:?}\nREGS: {:?}", self.stack, self.regs),
                    0x02 => {
                        let len = self.pop();
                        let mut val = Vec::with_capacity(len as usize);
                        for _ in 0..len {
                            val.push(self.pop());
                        }
                        println!("{}", std::str::from_utf8(val.as_slice()).unwrap());
                    }
                    0x03 => println!("{}", self.pop_f32()),
                    _ => {}
                }
            }
            LDC => {
                let index = self.next() as usize;
                let len = self.module.borrow().constants()[index];
                let mut constant = Vec::with_capacity(len as usize + 1);
                for i in 0..=len {
                    constant.push(self.module.borrow().constants()[index + i as usize])
                }
                constant.reverse();
                self.stack.extend(constant.iter());
            }
            RET_I => return Some(Vec::from(&self.pop_32() as &[u8])), // TODO: fix return values
            CMP_I => {
                let a = self.pop_i32();
                let b = self.pop_i32();
                self.push(match a.cmp(&b) {
                    Ordering::Equal => 0x00,
                    Ordering::Greater => 0x01,
                    Ordering::Less => 0x02,
                })
            }
            IF_T => ordering!(0x01),
            IF_F => ordering!(0x00),
            IF_NE => ordering!(0x01, 0x02),
            IF_EQ => ordering!(0x00),
            IF_GT => ordering!(0x01),
            IF_LT => ordering!(0x02),
            IF_LE => ordering!(0x02, 0x00),
            IF_GE => ordering!(0x01, 0x00),
            _ => panic!("Unknown opcode: {}", self.current()),
        }
        None
    }
}

use crate::consts::*;
use crate::module::Module;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

pub struct Vm<'a> {
    program: &'a [u8],
    index: usize,
    regs: Vec<u8>,
    stack: Vec<u8>,
    module: Rc<RefCell<Module>>,
}

impl Vm<'_> {
    /// Create a new vm
    /// ```
    /// # use libvm::vm::Vm;
    ///
    /// let vm = Vm::new(&[], Vec::new(), Default::default());
    /// ```
    pub fn new(program: &[u8], regs: Vec<u8>, module: Rc<RefCell<Module>>) -> Vm {
        Vm {
            program,
            index: 0,
            regs,
            stack: Vec::new(),
            module,
        }
    }

    fn next(&mut self) -> u8 {
        let ret = self.program[self.index];
        self.index += 1;
        ret
    }

    fn current(&self) -> u8 {
        self.program[self.index]
    }

    fn next_int(&mut self) -> [u8; 4] {
        let mut out = [self.next(), self.next(), self.next(), self.next()];
        out.reverse();
        out
    }

    /// Push a byte onto the stack
    fn push(&mut self, v: u8) {
        self.stack.push(v)
    }

    /// Push an int in the form of an array onto the stack
    fn push_int(&mut self, v: [u8; 4]) {
        for i in v.iter() {
            self.stack.push(*i);
        }
    }

    /// Push an 32 bit num in the form of an f32 onto the stack
    fn push_int_f(&mut self, v: f32) {
        unsafe {
            let mut a = std::mem::transmute::<f32, [u8; 4]>(v);
            a.reverse();
            self.push_int(a)
        }
    }

    /// Push an int in the form of an i32 onto the stack
    fn push_int_i(&mut self, v: i32) {
        let x = v as u32;
        let b1: u8 = ((x >> 24) & 0xff) as u8;
        let b2: u8 = ((x >> 16) & 0xff) as u8;
        let b3: u8 = ((x >> 8) & 0xff) as u8;
        let b4: u8 = (x & 0xff) as u8;
        self.push_int([b4, b3, b2, b1])
    }

    /// Pop a byte from the stack
    fn pop(&mut self) -> u8 {
        self.stack.pop().unwrap()
    }

    /// Pop a int in the form of an array off the stack
    fn pop_int(&mut self) -> [u8; 4] {
        let mut out = [self.pop(), self.pop(), self.pop(), self.pop()];
        out.reverse();
        out
    }

    /// Pop an 32 bit num in the form of an f32 off the stack
    fn pop_int_f(&mut self) -> f32 {
        let mut array = self.pop_int();
        array.reverse();
        unsafe { std::mem::transmute(array) }
    }

    /// Pop an int in the form of an i32 off the stack
    fn pop_int_i(&mut self) -> i32 {
        let mut array = self.pop_int();
        array.reverse();
        let mut combined: u32 = 0;
        for v in array.iter() {
            combined = (combined << 8) | u32::from(*v);
        }
        combined as i32
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

    /// Run the program
    /// ```
    /// # use libvm::vm::Vm;
    /// # use libvm::consts::*;
    ///
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
					let rhs = self.pop_int_i();
					let lhs = self.pop_int_i();
					self.push_int_i(lhs $op rhs);
				}
            };
            (f$op: tt) => {
				{
					let rhs = self.pop_int_f();
                    let lhs = self.pop_int_f();
					self.push_int_f(lhs $op rhs);
				}
            };
			(ib$op: tt) => {
				{
					let rhs = self.pop_int_i();
					let lhs = self.pop_int_i();
					self.push((lhs $op rhs) as u8);
				}
            };
			(fb$op: tt) => {
				{
					let rhs = self.pop_int_f();
					let lhs = self.pop_int_f();
					self.push((lhs $op rhs) as u8);
				}
			};
		}
        while self.index < self.program.len() {
            match self.next() {
                PUSH_I => {
                    let val = self.next_int();
                    self.push_int(val);
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
                    let n = self.pop_int_i();
                    self.push_int_i(-n);
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
                    self.push_int(self.get_int());
                }
                GOTO => {
                    let location = self.next();
                    self.index = location as usize;
                }
                STO_I => {
                    let reg = self.next() as usize;
                    let val = self.pop_int();
                    if self.regs.len() <= reg + 3 {
                        for v in val.iter() {
                            self.regs.push(*v);
                        }
                    } else {
                        for (i, v) in val.iter().enumerate() {
                            *self.regs.get_mut(i).unwrap() = *v;
                        }
                    }
                }
                LOAD_I => {
                    let reg = self.next() as usize;
                    for i in 0..4 {
                        self.push(self.regs[reg + i]);
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
                        0x00 => println!("{}", self.pop_int_i()),
                        0x01 => println!("STACK: {:?}", self.stack),
                        0x02 => {
                            let len = self.pop();
                            let mut val = Vec::with_capacity(len as usize);
                            for _ in 0..len {
                                val.push(self.pop());
                            }
                            println!("{}", std::str::from_utf8(val.as_slice()).unwrap());
                        }
                        0x03 => println!("{}", self.pop_int_f()),
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
                RET_I => return Vec::from(&self.pop_int() as &[u8]), // TODO: fix return values
                CMP_I => {
                    let a = self.pop_int_i();
                    let b = self.pop_int_i();
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
        }
        vec![]
    }
}

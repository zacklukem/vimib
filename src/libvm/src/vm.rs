use crate::consts::*;
use std::cmp::Ordering;

pub struct Vm<'a> {
    program: &'a [u8],
    index: usize,
    regs: [u8; 0xff],
    stack: Vec<u8>,
}

impl Vm<'_> {
    pub fn new(program: &[u8]) -> Vm {
        Vm {
            program,
            index: 0,
            regs: [0; 0xff],
            stack: Vec::new(),
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

    /// Get an int in the form of an i32 from the stack
    fn get_int_i(&self) -> i32 {
        let mut array = self.get_int();
        array.reverse();
        let mut combined: u32 = 0;
        for v in array.iter() {
            combined = (combined << 8) | u32::from(*v);
        }
        combined as i32
    }

    /// Run the program
    pub fn run(&mut self) -> &[u8] {
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
			(ib$op: tt) => {
				{
					let rhs = self.pop_int_i();
					let lhs = self.pop_int_i();
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

                NE => binary_operator!(ib!=),
                EQ => binary_operator!(ib==),
                GT => binary_operator!(ib>),
                LT => binary_operator!(ib<),
                GE => binary_operator!(ib>=),
                LE => binary_operator!(ib<=),

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
                    for (i, v) in val.iter().enumerate() {
                        self.regs[reg + i] = *v;
                    }
                }
                LOAD_I => {
                    let reg = self.next() as usize;
                    for i in 0..4 {
                        self.push(self.regs[reg + i]);
                    }
                }
                VIRTUAL => {
                    let call = self.next();
                    match call {
                        0x00 => println!("{}", self.get_int_i()),
                        0x01 => println!("STACK: {:?}", self.stack),
                        _ => {}
                    }
                }
                RET => return self.stack.as_slice(),
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
        &[]
    }
}

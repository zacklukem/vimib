use crate::consts;
use crate::function::Function;
use std::collections::HashMap;
use crate::vm_type::Type;

#[derive(Default)]
pub struct Module {
    constants: Vec<u8>,
    functions: HashMap<usize, Function>,
}

impl Module {
    pub fn new_const(&mut self, val: &str) -> usize {
        let index = self.constants.len();
        let len = val.len();
        self.constants.push(len as u8);
        self.constants.extend(val.as_bytes().iter());
        index
    }

    pub fn get_main(&self) -> &Function {
        self.functions.iter().find(|(i, _)| {
            let mut buffer = String::new();
            let mut iter = self.constants.iter();
            let len = iter.nth(**i).unwrap();
            for _ in 0..*len {
                buffer.push(*iter.next().unwrap() as char);
            }
            buffer == String::from("main")
        }).unwrap().1
    }

    pub fn disassemble(&self) {
        println!("constants:");
        let mut buffer = String::new();
        let mut iter = self.constants.iter().enumerate();
        while let Some((i, byte)) = iter.next() {
            for _ in 0..*byte {
                buffer.push(*iter.next().unwrap().1 as char);
            }
            println!("{}: {}", i, buffer);
            buffer.clear();
        }
        println!();
        for (i, func) in self.functions.iter() {
            let mut name = String::new();
            let mut iter = self.constants.iter();
            let len = iter.nth(*i).unwrap();
            for _ in 0..*len {
                name.push(*iter.next().unwrap() as char);
            }
            println!(
                "{}({:?}):\n{}",
                name,
                func.params(),
                consts::disassemble(func.program().as_slice())
            );
        }
    }

    pub fn run_main(&self) {
        self.get_main().run(Vec::new());
    }

    pub fn push_fn(&mut self, name: String, function: Function) -> usize {
        let i = self.new_const(name.as_str());
        self.functions.insert(i, function);
        i
    }

    pub fn get_fn(&self, function: usize) -> &Function {
        self.functions
            .get(&function)
            .unwrap()
    }

    pub fn call(&self, function: usize, stack: &mut Vec<u8>) -> Vec<u8> {

        let func = self.get_fn(function);
        let mut params = Vec::new();
        for param in func.params().iter() {
            let len = match *param {
                Type::I32 => 4
            };
            for _ in 0..len {
                // params.push(0);
                params.push(stack.pop().unwrap());
            }
        }
        params.reverse();
        func.run(params)
    }

    pub fn constants(&self) -> &[u8] {
        self.constants.as_slice()
    }
    pub fn functions(&self) -> &HashMap<usize, Function> {
        &self.functions
    }
}

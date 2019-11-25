use crate::consts;
use crate::function::Function;
use std::collections::HashMap;

#[derive(Default)]
pub struct Module {
    constants: Vec<u8>,
    functions: HashMap<String, Function>,
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
        self.functions
            .get(&String::from("main"))
            .expect("No Main Function!") // TODO: Fix this error
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
        for (name, func) in self.functions.iter() {
            println!(
                "{}:\n{}",
                name,
                consts::disassemble(func.program().as_slice())
            );
        }
    }

    pub fn run_main(&self) {
        self.get_main().run();
    }

    pub fn push_fn(&mut self, name: String, function: Function) -> usize {
        self.functions.insert(name.clone(), function);
        self.new_const(name.as_str())
    }

    pub fn call(&self, function: usize) -> Vec<u8> {
        let len = self.constants[function];
        let string = &self.constants()[function + 1..=function + len as usize];

        self.functions
            .get(&String::from(std::str::from_utf8(string).unwrap()))
            .unwrap()
            .run()
    }

    pub fn constants(&self) -> &[u8] {
        self.constants.as_slice()
    }
}

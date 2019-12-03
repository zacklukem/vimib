use crate::consts;
use crate::function::Function;
use crate::vm_type::Type;
use std::collections::HashMap;

#[derive(Default, PartialEq, Debug)]
pub struct Module {
    constants: Vec<u8>,
    functions: HashMap<usize, Function>,
}

impl Module {
    /// Creates a new string constant and returns it's index
    /// ```
    /// # use libvm::module::*;
    ///
    /// let mut module: Module = Default::default();
    /// let index = module.new_const("Hello, World!");
    /// assert_eq!(module.constants()[index], 13);
    /// assert_eq!(module.constants()[index + 1], 'H' as u8);
    /// ```
    pub fn new_const(&mut self, val: &str) -> usize {
        let index = self.constants.len();
        let len = val.len();
        self.constants.push(len as u8);
        self.constants.extend(val.as_bytes().iter());
        index
    }

    /// Return the main function and panics if it doesn't exist
    /// ```
    /// # use libvm::module::*;
    /// # use libvm::function::Function;
    /// 
    /// let mut module: Module = Default::default();
    /// let func: Function = Default::default();
    /// module.push_fn("main".to_string(), func.clone());
    /// let main = module.get_main();
    /// assert_eq!(*main, func);
    /// ```
    pub fn get_main(&self) -> &Function {
        self.functions
            .iter()
            .find(|(i, _)| {
                let mut buffer = String::new();
                let mut iter = self.constants.iter();
                let len = iter.nth(**i).unwrap();
                for _ in 0..*len {
                    buffer.push(*iter.next().unwrap() as char);
                }
                buffer == "main"
            })
            .unwrap()
            .1
    }

    /// Disassembles the module and prints it out
    /// ```
    /// # use libvm::module::Module;
    /// let module: Module = Default::default();
    /// module.disassemble();
    /// ```
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
    
    /// Runs the main function and panics if it doesn't exist
    /// ```
    /// # use libvm::module::*;
    /// # use libvm::function::Function;
    /// 
    /// let mut module: Module = Default::default();
    /// let func: Function = Default::default();
    /// module.push_fn("main".to_string(), func.clone());
    /// module.run_main();
    /// ``` 
    pub fn run_main(&self) {
        self.get_main().run(Vec::new());
    }

    /// Pushes a function to the module
    /// 
    /// ```
    /// # use libvm::module::*;
    /// # use libvm::function::Function;
    /// 
    /// let mut module: Module = Default::default();
    /// let func: Function = Default::default();
    /// let index = module.new_const("main";
    /// module.push_fn(index, func.clone());
    /// let other = module.get_fn(index);
    /// assert_eq!(*other, func);
    /// ```
    pub fn push_fn(&mut self, index: usize, function: Function) {
        self.functions.insert(index, function);
    }

    /// Gets a function by it's id and returns a reference to it
    /// ```
    /// # use libvm::module::*;
    /// # use libvm::function::Function;
    /// 
    /// let mut module: Module = Default::default();
    /// let func: Function = Default::default();
    /// let index = module.push_fn("main".to_string(), func.clone());
    /// let other = module.get_fn(index);
    /// assert_eq!(*other, func);
    /// ```
    pub fn get_fn(&self, function: usize) -> &Function {
        self.functions.get(&function).unwrap()
    }

    /// Calls a function with a stack as parameters and return's its return
    /// results
    /// ```
    /// # use libvm::module::*;
    /// # use libvm::function::Function;
    /// 
    /// let mut module: Module = Default::default();
    /// let func: Function = Default::default();
    /// let index = module.push_fn("main".to_string(), func.clone());
    /// let mut stack = vec![];
    /// module.call(index, &mut stack);
    /// ```
    pub fn call(&self, function: usize, stack: &mut Vec<u8>) -> Vec<u8> {
        let func = self.get_fn(function);
        let mut params = Vec::new();
        for param in func.params().iter() {
            let len = match *param {
                Type::I32 => 4,
            };
            for _ in 0..len {
                // params.push(0);
                params.push(stack.pop().unwrap());
            }
        }
        params.reverse();
        func.run(params)
    }

    /// Returns this module's constants
    pub fn constants(&self) -> &[u8] {
        self.constants.as_slice()
    }

    // Return this module's functions
    pub fn functions(&self) -> &HashMap<usize, Function> {
        &self.functions
    }
}

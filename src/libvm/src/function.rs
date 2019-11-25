use crate::module::Module;
use crate::vm::Vm;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Function {
    program: Vec<u8>,
    module: Rc<RefCell<Module>>,
}

impl Function {
    pub fn new(program: Vec<u8>, module: Rc<RefCell<Module>>) -> Function {
        Function { program, module }
    }

    pub fn program(&self) -> &Vec<u8> {
        &self.program
    }
    pub fn run(&self) -> Vec<u8> {
        let mut vm = Vm::new(self.program.as_slice(), Rc::clone(&self.module));
        vm.run()
    }
}

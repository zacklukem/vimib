use crate::module::Module;
use crate::vm::Vm;
use crate::vm_type::Type;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Function {
    program: Vec<u8>,
    params: Vec<Type>,
    module: Rc<RefCell<Module>>,
}

impl Function {
    pub fn new(program: Vec<u8>, params: Vec<Type>, module: Rc<RefCell<Module>>) -> Function {
        Function {
            program,
            params,
            module,
        }
    }

    pub fn program(&self) -> &Vec<u8> {
        &self.program
    }
    pub fn params(&self) -> &Vec<Type> {
        &self.params
    }
    pub fn run(&self, params: Vec<u8>) -> Vec<u8> {
        let mut vm = Vm::new(self.program.as_slice(), params, Rc::clone(&self.module));
        vm.run()
    }
}

use crate::module::Module;
use crate::vm::Vm;
use crate::vm_type::Type;
use std::cell::RefCell;
use std::rc::Rc;

/// A vm function.  A function contains a program which is a set of instructions
/// run by the virtual machine.  A function also has typed params and a return
/// type.
#[derive(PartialEq, Debug, Default, Clone)]
pub struct Function {
    program: Vec<u8>,
    params: Vec<Type>,
    return_type: Type,
    module: Rc<RefCell<Module>>,
}

impl Function {
    /// Create a new function with these initial values
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use libvm::function::*;
    /// let function = Function::new(vec![], vec![], Default::default(), Default::default());
    /// ```
    /// This function is mostly useless however, as it has an empty module, and
    /// no parameters.
    pub fn new(
        program: Vec<u8>,
        params: Vec<Type>,
        return_type: Type,
        module: Rc<RefCell<Module>>,
    ) -> Function {
        Function {
            program,
            params,
            return_type,
            module,
        }
    }

    /// Returns a reference to the return type of the function
    pub fn return_type(&self) -> &Type {
        &self.return_type
    }

    /// Returns a reference to the program
    pub fn program(&self) -> &Vec<u8> {
        &self.program
    }

    /// Returns a reference the types of parameters of the function
    pub fn params(&self) -> &Vec<Type> {
        &self.params
    }

    /// Runs the program.  This creates a new [`Vm`](../vm/struct.Vm.html) each
    /// time the program is run with a fresh state and scope.  It returns the
    /// last `n` digits of the stack where `n` is the size of the function's
    /// return type.
    /// 
    /// # Examples
    /// ```
    /// # use libvm::function::*;
    /// # use libvm::consts::*;
    /// let func = Function::new(vec![
    ///     PUSH_I, 0, 0, 0, 5,
    ///     RET_I
    /// ], vec![], Default::default(), Default::default());
    /// let out = func.run(vec![]);
    /// assert_eq!(out, vec![5, 0, 0, 0]);
    /// ```
    pub fn run(&self, params: Vec<u8>) -> Vec<u8> {
        let mut vm = Vm::new(self.program.as_slice(), params, Rc::clone(&self.module));
        vm.run()
    }
}

use libvm::module::Module;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ObjBuilder {
    module: Rc<RefCell<Module>>,
    out: Vec<u8>,
}

impl ObjBuilder {
    pub fn new(module: Rc<RefCell<Module>>) -> ObjBuilder {
        ObjBuilder {
            module,
            out: Vec::new(),
        }
    }

    pub fn gen(&mut self) -> &[u8] {
        // Magic
        self.out.push(0xBB);
        self.out.push(0xBB);
        self.out.push(0xBB);
        self.out.push(0xBB);

        self.out.push(0x00); // TODO: Major version
        self.out.push(0x00); // TODO: Minor version

        self.out.push(self.module.borrow().constants().len() as u8); // Constants len
        self.out.extend(self.module.borrow().constants().iter());

        for (i, func) in self.module.borrow().functions().iter() {
            self.out.push(*i as u8);
            self.out.push(func.params().len() as u8); // Params Len
            self.out
                .extend(func.params().iter().map(|v| v.serialize()).flatten());
            self.out.push(func.program().len() as u8); // Program Len
            self.out.extend(func.program().iter());
        }
        self.out.as_slice()
    }
}

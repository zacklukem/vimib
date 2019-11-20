use libcodegen::*;
use libparser::*;
use libvm::consts::*;
use libvm::*;

fn main() {
    static INPUT: &str = r"
let i = 0

loop {
    println(i)
    if i >= 10 {
        println(1234123412)
        break
    }
    i = i + 1
}
    ";
    let mut gen = OpcodeGenerator::new(INPUT);
    let ctx = &parse_context::ParseContext::new(INPUT);
    let mut parser = parser::Parser::new(INPUT, ctx);
    let body = parser.parse();
    gen.gen_block(&body);

    let block = gen.gen();

    println!("root:\n{}", disassemble(block));

    let mut vm = vm::Vm::new(block);

    vm.run();
}

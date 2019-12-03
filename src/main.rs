use libcodegen::*;
use libparser::*;

fn main() {
    static INPUT: &str = include_str!("../example.vimib");
    let mut gen = OpcodeGenerator::new(INPUT);
    let ctx = &parse_context::ParseContext::new(INPUT);
    let mut parser = parser::Parser::new(INPUT, ctx);
    let body = parser.parse();
    gen.gen_module(&body);

    let module = gen.gen();
    module.borrow().disassemble();
    module.borrow().run_main();
}

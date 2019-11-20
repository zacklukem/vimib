use libcodegen::*;
use libparser::*;
use libvm::consts::*;
use libvm::*;

fn main() {
    static INPUT: &str = r#"
        let i = 0

        loop {
            print_int(i)
            if i >= 10 {
                print_str("Hello,")
                print_str("World!")
                break
            }
            i = i + 1
        }
    "#;
    let mut gen = OpcodeGenerator::new(INPUT);
    let ctx = &parse_context::ParseContext::new(INPUT);
    let mut parser = parser::Parser::new(INPUT, ctx);
    let body = parser.parse();
    gen.gen_block(&body);

    let (block, consts) = gen.gen();

    println!("root:\n{}", disassemble(block));

    let mut vm = vm::Vm::new(block, consts);

    vm.run();
}

use libcodegen::*;
use libparser::*;

fn main() {
    static INPUT: &str = r#"
    fn say_hi() {
        print_str("Hello,")
        print_str("World!")
    }

    fn main() {
        let i = 0

        loop {
            print_int(i)
            if i >= 10 {
                say_hi()
                break
            }
            i = i + 1
        }
    }
    "#;
    let mut gen = OpcodeGenerator::new(INPUT);
    let ctx = &parse_context::ParseContext::new(INPUT);
    let mut parser = parser::Parser::new(INPUT, ctx);
    let body = parser.parse();
    gen.gen_module(&body);

    let module = gen.gen();
    module.borrow().disassemble();
    module.borrow().run_main();
}

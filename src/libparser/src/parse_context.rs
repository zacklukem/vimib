use crate::span::Span;

pub struct ParseContext<'a> {
    input: &'a str,
}

impl ParseContext<'_> {
    pub fn new(input: &str) -> ParseContext {
        ParseContext { input }
    }

    pub fn error(&self, span: Span, message: &str) {
        // Count new lines
        let mut num_lines = 0;
        let mut covered = 0;
        let mut iter = self.input.chars();
        for i in 0..span.pos.0 {
            let c = iter.next().unwrap();
            if c == '\n' {
                covered = i;
                num_lines += 1;
            }
        }
        let line = self.input.split('\n').nth(num_lines).unwrap();
        eprintln!("\u{001b}[33merror: {}\u{001b}[0m", message);
        eprintln!("    \u{001b}[33m{} |\u{001b}[0m {}", num_lines + 1, line);
        eprintln!(
            "       {}^",
            (0..(span.pos.0 - covered)).map(|_| " ").collect::<String>()
        );
    }
}

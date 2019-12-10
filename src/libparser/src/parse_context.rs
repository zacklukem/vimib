use crate::span::Span;

/// Parsing context.  Manages printing out errors.
#[derive(Default)]
pub struct ParseContext<'a> {
    input: &'a str,
}

impl ParseContext<'_> {
    /// Creates a new parse context
    /// # Examples
    /// ```
    /// # use libparser::parse_context::*;
    /// let context = ParseContext::new("asd");
    /// ```
    pub fn new(input: &str) -> ParseContext {
        ParseContext { input }
    }

    /// Print an error for a span.
    /// # Examples
    /// ```
    /// # use libparser::parse_context::*;
    /// # use libparser::span::Span;
    /// let context = ParseContext::new("asd");
    /// context.error(Span::new(0, 3), "Error message");
    /// ```
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
            "       \u{001b}[34m{}{}\u{001b}[0m",
            (0..(span.pos.0 - covered)).map(|_| " ").collect::<String>(),
            (0..(span.pos.1 - span.pos.0))
                .map(|_| "^")
                .collect::<String>()
        );
    }
}

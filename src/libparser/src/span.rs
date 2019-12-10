/// Span represents a chunk of code with its starting index and ending index.
#[derive(Clone, Copy, PartialEq)]
pub struct Span {
    pub pos: (usize, usize),
    pub is_dummy: bool,
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_dummy {
            write!(f, "(dummy)")
        } else {
            write!(f, "{:?}", self.pos)
        }
    }
}

impl Span {
    /// Create a new span with pos `(start, end)`
    /// ```
    /// # use libparser::span::Span;
    /// let span = Span::new(0, 5);
    /// assert_eq!(span.pos, (0, 5));
    /// ```
    pub fn new(start: usize, end: usize) -> Span {
        Span {
            pos: (start, end),
            is_dummy: false,
        }
    }

    /// Create a new dummy span with pos `(0, 0)`
    /// ```
    /// # use libparser::span::Span;
    /// let span = Span::dummy();
    /// assert!(span.is_dummy);
    /// ```
    pub fn dummy() -> Span {
        Span {
            pos: (0, 0),
            is_dummy: true,
        }
    }
}

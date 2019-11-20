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
        // let s = include_str!("../../test.an");
        // write!(f, "{}", &s[self.pos.0..self.pos.1])
    }
}

impl Span {
    pub fn new(start: usize, end: usize) -> Span {
        Span {
            pos: (start, end),
            is_dummy: false,
        }
    }

    pub fn dummy() -> Span {
        Span {
            pos: (0, 0),
            is_dummy: true,
        }
    }
}

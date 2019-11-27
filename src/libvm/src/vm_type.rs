#[derive(Debug)]
pub enum Type {
    I32
}

impl Type {
    pub fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(0x00); // TODO: Make this right
        out
    }
}
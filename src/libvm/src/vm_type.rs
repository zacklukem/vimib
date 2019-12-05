#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    I32,
    Void,
    String,
    F32,
}

impl std::default::Default for Type {
    fn default() -> Type {
        Type::Void
    }
}

impl Type {
    /// Convert this type into a vector of u8 representing it's type
    /// ```
    /// # use libvm::vm_type::Type;
    /// let t = Type::I32;
    /// assert_eq!(t.serialize(), vec![0x00]);
    pub fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(0x00); // TODO: Make this right
        out
    }
}

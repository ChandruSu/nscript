pub mod vm {
    #[derive(PartialEq, Debug)]
    pub enum Value {
        Null,
        Int(i32),
        Float(f32),
        Bool(bool),
        Func(u32, u16),
        String(String),
    }
}

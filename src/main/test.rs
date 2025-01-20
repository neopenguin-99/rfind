pub use self::test::Test;
pub mod test {
    use crate::main::*;
    #[derive(Debug)]
    pub enum Test {
        Name(String),
        Types(String)
    }
}

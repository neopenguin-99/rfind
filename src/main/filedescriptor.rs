pub use self::filedescriptor::FileDescriptor;
pub mod filedescriptor {
    use crate::main::*;
    #[derive(Clone, Debug, PartialEq, Copy)]
    pub enum FileDescriptor {
        StdIn = 0,
        StdOut = 1,
        StdErr = 2
    }
}

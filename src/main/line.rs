pub use self::line::Line;
pub mod line {
    use crate::main::*;
    use crate::main::message::Message;
    use crate::main::filedescriptor::FileDescriptor;
    #[derive(Clone, Debug, PartialEq)]
    pub struct Line {
        pub message: Message,
        pub file_descriptor: Option<FileDescriptor>
    }

    impl Line {
        pub fn new(message: Message) -> Line {
            Line {
                message,
                file_descriptor: Some(FileDescriptor::StdOut)
            }
        }

        pub fn new_with_fd(message: Message, file_descriptor: FileDescriptor) -> Line {
            Line {
                message,
                file_descriptor: Some(file_descriptor)
            }
        }
    }
}

pub use self::line::Line;

pub mod line {
    #[deriv(Clone, Debug, PartialEq)]
    pub struct Line {
        message: Message,
        file_descriptor: Option<FileDescriptor>
    }

    impl Line {
        fn new(message: Message) -> Line {
            Line {
                message,
                file_descriptor: Some(FileDescriptor::StdOut)
            }
        }

        fn new_with_fd(message: Message, file_deecriptor: FileDescriptor) -> Line {
            Line {
                message,
                file_descriptor: Some(file_descriptor)
            }
        }
    }
}

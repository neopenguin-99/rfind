pub use self::logger::Logger;
pub mod logger {
    use crate::main::*;
    use crate::main::line::Line;

    pub trait Logger {
        fn log(&mut self, line: Line);
        fn log_as_tree(&mut self, dir_entries: Vec<(String, bool)>, preceding_str: Option<String>) -> Vec<String>;
    }
}

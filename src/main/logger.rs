pub use self::logger::Logger;
pub mod logger {
    use crate::main::*;

    pub trait Logger {
        fn log(&mut self, line: Line);
        fn log_as_tree(&mut self, dir_entries: Vec<(String, bool)>, preceding_str: Option<String>) -> Vec<String>;
    }

    #[derive(Debug)]
    pub struct StandardLogger { }

    impl StandardLogger {
        pub fn new() -> StandardLogger {
            StandardLogger {
            }
        }
    }

    impl Logger for StandardLogger {
        fn log(&mut self, line: Line) { 
            let str_message = line.message.get_contained_message();
            _ = match line.file_descriptor {
                Some(fd) if (fd as i32 == 1) => {
                    println!("{}", str_message);
                }
                Some(fd) if (fd as i32 == 2) => {
                    eprintln!("{}", str_message);
                }
                Some(fd) => {
                    let x = fd as i32;
                    let mut f = unsafe { File::from_raw_fd(x) };
                    write!(&mut f, "{}", str_message).unwrap();
                }
                None => {
                    println!("{}", 1);
                    let mut f = unsafe { File::from_raw_fd(1) };
                    write!(&mut f, "{}", str_message).unwrap();
                }
            }
        }

        fn log_as_tree(&mut self, dir_entries: Vec<(String, bool)>, preceding_str: Option<String>) -> Vec<String> {
            let result = Vec::<String>::new();
            for entry in dir_entries {
                if entry.1 {
                    println!("{}{}", preceding_str.clone().unwrap_or(String::new()), entry.0.green())
                }
                else {
                    println!("{}{}", preceding_str.clone().unwrap_or(String::new()), entry.0.red())
                }
            }
            result
        }
    }
}

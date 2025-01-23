pub use self::testlogger::TestLogger;
pub mod testlogger {
    use crate::main::*;
    use crate::main::line::Line;
    use crate::main::filedescriptor::FileDescriptor;
    use crate::main::logger::Logger;
    pub struct TestLogger {
        logs: Vec<Line>
    }

    impl TestLogger {
        pub fn new() -> TestLogger {
            TestLogger {
                logs: Vec::new(),
            }
        }

        pub fn print(&self) {
            let logs = &self.logs;
            let logs_iter = logs.into_iter();
            for log in logs_iter {
                let b = log.message.clone();
                match b {
                    self::message::Message::Standard(msg) => println!("print: {}", msg),
                    _ => continue
                }
            }

        }

        pub fn get_logs(&self) -> Vec<&Line> {
            let logs = &self.logs;
            let logs_iter = logs.into_iter();
            logs_iter.filter(|_| {
                true
            }).collect()
        }

        pub fn get_logs_by_file_descriptor(&self, file_descriptor: FileDescriptor) -> Vec<&Line> {
            let logs = &self.logs;
            let logs_iter = logs.into_iter();
            logs_iter.filter(move |&x| {
                x.file_descriptor == Some(file_descriptor)
            }).collect()
        }

        pub fn get_lines_from_logs_where_logs_contains_provided_value(lines: Vec<&Line>, line_to_find: String) -> bool {
            for line in lines.into_iter() {
                let message = line.message.get_contained_message().clone().to_string();
                if message.contains(&line_to_find) {
                    return true;
                }
            };
            false
        }
    }

    impl Logger for TestLogger {
        fn log(&mut self, line: Line) {
            self.logs.push(line);
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

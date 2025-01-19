pub mod main {

    use std::borrow::Borrow;
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::{borrow::BorrowMut, cell::RefCell, fmt::Debug, ptr, rc::Rc, cell::Ref};
    use std::io;
    use std::mem::{discriminant, Discriminant};
    use std::io::{Write, Read, Seek, SeekFrom};
    use std::fs::{self, FileType, ReadDir};
    use std::os::unix::fs::FileTypeExt;
    use std::path::Path;
    use std::process::exit;
    use std::env;
    use clap::{arg, crate_authors, crate_version, value_parser, Arg, ArgAction, ArgMatches, Command, ValueEnum};
    use libc::write;
    use std::sync::Mutex;
    use std::fs::File;
    use speculoos::prelude::*;
    use test_case::test_case;
    use assert_fs::prelude::*;
    use colored::Colorize;
    use predicates::prelude::*;
    use std::thread;
    use std::time::Duration;

    #[derive(Debug)]
    pub enum Test {
        Name(String),
        Types(String)
    }

    #[derive(Clone, Debug, PartialEq, Copy)]
    pub enum FileDescriptor {
        StdIn = 0,
        StdOut = 1,
        StdErr = 2
    }





    #[derive(Debug, Clone, PartialEq)]
    pub enum SymLinkSetting {
        Never,
        Follow,
        OnlyCommandLineArgs
    }

    #[derive(Debug)]
    pub struct Params {
        pub symlink_setting: SymLinkSetting,
        pub debug_opts: Option<DebugOpts>,
        pub optimisation_level: Option<u8>
        
    }

    #[derive(Debug)]
    pub struct Searcher<T: Logger> {
        min_depth: Option<u32>,
        max_depth: Option<u32>,
        params: Params,
        logger: Rc<Mutex<T>>,
        pub starting_path: String
    }




    impl<T: Logger> Searcher<T> {
        pub fn new(params: Params, max_depth: Option<u32>, min_depth: Option<u32>, logger: Rc<Mutex<T>>, starting_path: String) -> Searcher<T> {
            Searcher {
                logger,
                params,
                max_depth,
                min_depth,
                starting_path
            }
        }

        pub fn search_directory_path(&self, directory_path: &Path, test: &Test, preceding_str: Option<String>, current_depth: Option<u32>) {
            let current_depth = current_depth.unwrap_or(0);
            let read_dir = match fs::read_dir(directory_path) {
                Ok(res) => {
                    res
                }
                Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                    let line = format!("rfind: Permission denied for directory name {}", directory_path.to_str().unwrap());
                    self.logger.lock().unwrap().log(Line::new_with_fd(Message::Standard(line), FileDescriptor::StdErr));
                    return;
                }
                Err(_) => {
                    let line = format!("rfind: An error occurred when attempting to read the {} directory", directory_path.to_str().unwrap());
                    self.logger.lock().unwrap().log(Line::new_with_fd(Message::Standard(line), FileDescriptor::StdErr));
                    return;
                }
            };
            let mut read_dir_iter = read_dir.peekable();
            while let Some(ele) = read_dir_iter.next() {
                let mut preceding_str = preceding_str.clone().unwrap_or(String::new()).clone();
                if self.params.debug_opts.is_some() {
                    let debug_opts = self.params.debug_opts.as_ref().unwrap();
                    if read_dir_iter.peek().is_some() && *debug_opts == DebugOpts::Tree {
                        preceding_str.push_str("├── ")
                    }
                    else if read_dir_iter.peek().is_none() && *debug_opts == DebugOpts::Tree {
                        preceding_str.push_str("└── ") 
                    }
                }
                let ele = ele.unwrap();
                let file_name = ele.file_name();
                let file_type = ele.file_type().unwrap();

                if file_type.is_symlink() && self.params.symlink_setting == SymLinkSetting::Follow {
                    // navigate to the file pointed to by the symlink
                    let file_referred_to_by_symlink = fs::read_link(ele.path());
                    _ = match file_referred_to_by_symlink {
                        Ok(file_referred_to_by_symlink_unwrapped) => {
                            let line = format!("{}{}", preceding_str, file_referred_to_by_symlink_unwrapped.to_str().unwrap());
                            self.logger.lock().unwrap().log(Line::new_with_fd(Message::Standard(line), FileDescriptor::StdOut));
                            continue;
                        }
                        Err(error) if error.kind() == io::ErrorKind::NotFound && read_dir_iter.peek().is_some() => {
                            let line = format!("{}Broken symlink: {}", preceding_str, ele.path().to_str().unwrap());
                            self.logger.lock().unwrap().log(Line::new_with_fd(Message::Standard(line), FileDescriptor::StdErr));
                            continue;
                        }
                        Err(_) => {
                            unreachable!("We have handled both cases where read_link would result in an error, so this should be unreachable");
                        }
                    }
                }
        
                // todo make operator logic, by default all tests have to pass to return a find.
                // todo
                // self.logger.lock().unwrap().log(LogLine::StdOut(directory_path.join(file_name.clone()).to_str().unwrap().to_string()));
                let line_to_log;
                line_to_log = match test {
                    Test::Name(name) if file_name.to_str().unwrap() == name => true,
                    Test::Types(provided_file_type) if 
                    (file_type.is_block_device() && provided_file_type.contains('b')) &&
                    (file_type.is_char_device() && provided_file_type.contains('c')) &&
                    (file_type.is_dir() && provided_file_type.contains('d')) &&
                    (file_type.is_file() && provided_file_type.contains('f')) &&
                    (file_type.is_fifo() && provided_file_type.contains('p')) &&
                    (file_type.is_symlink() && provided_file_type.contains('l') && self.params.symlink_setting != SymLinkSetting::Follow) &&
                    (file_type.is_socket() && provided_file_type.contains('s')) => true,
                    _ => false,
                };
                if line_to_log && (self.min_depth.is_some() && current_depth >= self.min_depth.unwrap()) || self.min_depth.is_none() {
                    self.logger.lock().unwrap().log(Line::new_with_fd(Message::Standard(directory_path.join(file_name).to_str().unwrap().to_string()), FileDescriptor::StdOut));
                    continue;
                }

                if file_type.is_dir() && (self.max_depth.is_some() && current_depth < self.max_depth.unwrap() || self.max_depth.is_none()) {
                    let file_name = ele.file_name();
                    let file_name: &str = file_name.to_str().unwrap();
                    let directory_path = directory_path.join(file_name);
                    let directory_path = directory_path.as_path();

                    let preceding_str_2: String;
                    match read_dir_iter.peek() {
                        Some(_) => preceding_str_2 = format!("{}| ", preceding_str),
                        None => preceding_str_2 = format!("{}  ", preceding_str)
                    }
                    self.search_directory_path(directory_path, test, Some(preceding_str_2), Some(current_depth + 1));
                }
            }
        }
    }

    pub struct TestLogger {
        logs: Vec<Line>
    }

    impl TestLogger {
        pub fn new() -> TestLogger {
            TestLogger {
                logs: Vec::new(),
            }
        }

        fn get_logs(&self) -> Vec<&Line> {
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

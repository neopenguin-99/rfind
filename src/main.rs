use std::borrow::Borrow;
use std::os::fd::FromRawFd;
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
use std::sync::Mutex;
use std::fs::File;
use speculoos::prelude::*;
use test_case::test_case;
use assert_fs::prelude::*;
use colored::Colorize;
use predicates::prelude::*;

fn main() {
    let logger = StandardLogger::new();


    let mut matches: ArgMatches = Command::new("MyApp")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .arg(Arg::new("symlink_never")
            .short('P')
            .action(ArgAction::SetTrue)
            .help("Never follow symbolic links")
        )
        .arg(Arg::new("symlink_follow")
            .short('L')
            .action(ArgAction::SetTrue)
            .help("Follow symbolic links")
        )
        .arg(Arg::new("symlink_only_command_line_args")
            .short('H')
            .action(ArgAction::SetTrue)
            .help("Do not follow symbolic links, except when processing command line arguments")
        )
        .arg(Arg::new("debug_opts")
            .short('D')
            .action(ArgAction::Set)
            .help("Set debug opts"))
        .arg(Arg::new("optimisation_level")
            .short('O')
            .value_parser(value_parser!(u8))
            .action(ArgAction::Set)
            .help("Set optimisation level")
        )
        .arg(Arg::new("max_depth")
            .value_parser(value_parser!(u32))
            .long("maxdepth")
            .action(ArgAction::Set)
            .help("Descend at most the provided number of levels, this value must be a non-negative integer.
            Using max depth of 0 will apply the expression 
            for files only in the current directory, and will not search subdirectories")
        )
        .arg(Arg::new("name")
            .long("name")
            .help("The name of the file to find")
        )
        .arg(Arg::new("type")
            .long("type")
            .help("The file type of the file to find")
        )
        .allow_missing_positional(true)
        .arg(Arg::new("starting_path").default_value("."))
        .arg(Arg::new("expression").default_value("--true").num_args(0..).value_parser(value_parser!(String)))
        .try_get_matches().unwrap();

    // parse the cmd arguments
    let mut symlink_setting: SymLinkSetting = SymLinkSetting::Never;

    if matches.remove_one::<bool>("symlink_only_command_line_args").is_some() {
        symlink_setting = SymLinkSetting::OnlyCommandLineArgs;
    }

    if matches.remove_one::<bool>("symlink_follow").is_some() {
        symlink_setting = SymLinkSetting::Follow;
    }

    if matches.remove_one::<bool>("symlink_never").is_some() {
        symlink_setting = SymLinkSetting::Never;
    }

    let debug_opts: Option<DebugOpts> = match matches.remove_one::<String>("debug_opts") {
        Some(x) if x == "exec" => Some(DebugOpts::Exec),
        _ => None
    };

    let optimisation_level: Option<u8> = match matches.remove_one::<u8>("optimisation_level") {
        Some(x) if x == 0 || x == 1 || x == 2 || x == 3 => Some(x),
        Some(_) => panic!("Invalid optimisation level provided"),
        _ => None
    };

    let params = Params {
        symlink_setting,
        debug_opts,
        optimisation_level
    };




    let max_depth = matches.remove_one::<u32>("max_depth");

    let starting_path = matches.remove_one::<String>("starting_path");
    
    let expression = match matches.remove_many::<String>("expression") {
        Some(expression) => {
            let mut a: Vec<String> = Vec::new();
            for token in expression {
                a.push(token.to_string())
            }
            a
        }
        _ => vec!["--true".to_string()]
    };
    
    println!("EXPRESSION: {:#?}", expression);
    eval(expression, Searcher::<StandardLogger>::new(params, max_depth, Rc::new(Mutex::new(logger)), starting_path.unwrap_or(format!("."))));
}

struct Params {
    symlink_setting: SymLinkSetting,
    debug_opts: Option<DebugOpts>,
    optimisation_level: Option<u8>
    
}

#[derive(PartialEq)]
enum DebugOpts {
    Exec,
    Opt,
    Rates,
    Search,
    Stat,
    Tree,
    All,
    Help
}

struct Expression {
    expression_str: Option<Box<Vec<String>>>,
    sub_expression: Option<Box<Vec<Expression>>>
}

fn some_test_returns_true(input: Vec<String>) -> bool {
    _ = input;
    true
}

fn some_test_returns_false(input: Vec<String>) -> bool {
    _ = input;
    false
}

fn eval<T: Logger>(tokens: Vec<String>, searcher: Searcher<T>) -> bool {
    let iter = tokens.iter();

    let mut ex = Expression {
        expression_str: Some(Box::new(tokens.clone())),
        sub_expression: None
    };

    for (i, el) in iter.clone().enumerate() {
        if el == ")" {
            panic!(") should not be here!");
        }
        if el == "(" {
            let iter2 = tokens[i+1..].iter();
            for (i2, el2) in iter2.enumerate() {
                if el2 == ")" {
                    return eval(tokens[i+1..i2-1].to_vec(), searcher);
                }
            }            
            panic!("Could not find enclosing )");

        }
    }

    let mut expression_result: bool = false;
    for (i, el) in iter.enumerate() {
        if el == "--or" {
            if expression_result {
                return true;
            } else {
                return expression_result || eval(tokens[i+1..].to_vec(), searcher);
            }
        }
        if el == "--and" {
            if !expression_result {
                return false;
            } else {
                return expression_result && eval(tokens[i+1..].to_vec(), searcher);
            }
        }
        if el == "--not" {
            return !eval(tokens[i+1..].to_vec(), searcher);
        }
        // tests logic
        let directory_path = Path::new(&searcher.starting_path);
        if el == "--name" { // todo maybe make one if statement for all tests?
            let name: String = tokens.get(i + 1).expect("--name expects a file name, but found nothing").clone();
            
            ex.expression_str = Some(Box::new(vec![el.to_string(), name.clone()]));
            let test = Test::Name(name.clone());
            searcher.search_directory_path(directory_path, &test, None, None, None);
            expression_result = some_test_returns_true(*ex.expression_str.unwrap());
        }
        else if el == "--type" {
            let r#type: String = tokens.get(i + 1).expect("--type expects a file type, but found nothing").clone();
            
            ex.expression_str = Some(Box::new(vec![el.to_string(), r#type.clone()]));
            let test = Test::Types(r#type);
            searcher.search_directory_path(directory_path, &test, None, None, None);
            expression_result = some_test_returns_true(*ex.expression_str.unwrap());
        }
    }
    expression_result
}

#[derive(Debug, Clone, PartialEq)]
enum SymLinkSetting {
    Never,
    Follow,
    OnlyCommandLineArgs
}

#[derive(Clone, Debug, PartialEq)]
struct Line {
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

    fn new_with_fd(message: Message, file_descriptor: FileDescriptor) -> Line {
        Line {
            message,
            file_descriptor: Some(file_descriptor)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
enum FileDescriptor {
    StdIn = 0,
    StdOut = 1,
    StdErr = 2
}

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Standard(String),
    Tree(String)
}

impl Message {
    fn get_contained_message(&self) -> &String {
        match self {
            Self::Standard(x) | Self::Tree(x) => x, 
        }
        //todo fix so that this works so that we don't
//have to update this method every time a new type of message is added to the message
//enum.
    }
}

trait Logger {
    fn log(&mut self, line: Line);
    fn log_as_tree(&mut self, dir_entries: Vec<(String, bool)>, preceding_str: Option<String>) -> Vec<String>;
}

struct StandardLogger { }

impl StandardLogger {
    fn new() -> StandardLogger {
        StandardLogger {
        }
    }
}

impl Logger for StandardLogger {
    fn log(&mut self, line: Line) {
        let str_message = line.message.get_contained_message();
        _ = match line.file_descriptor {
            Some(fd) => {
                let x = fd as i32;
                let mut f = unsafe { File::from_raw_fd(x) };
                write!(&mut f, "{}", str_message).unwrap();
            }
            None => {
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

#[derive(Debug)]
enum Test {
    Name(String),
    Types(String)
}

struct Searcher<T: Logger> {
    max_depth: Option<u32>,
    params: Params,
    logger: Rc<Mutex<T>>,
    starting_path: String
}

impl<T: Logger> Searcher<T> {
    pub fn new(params: Params, max_depth: Option<u32>, logger: Rc<Mutex<T>>, starting_path: String) -> Searcher<T> {
        Searcher {
            logger,
            params,
            max_depth,
            starting_path
        }
    }

    pub fn search_directory_path(&self, directory_path: &Path, test: &Test, debug_opts: Option<&DebugOpts>, preceding_str: Option<String>, current_depth: Option<u32>) {
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
            if read_dir_iter.peek().is_some() && debug_opts.is_some() && *debug_opts.unwrap() == DebugOpts::Tree {
                preceding_str.push_str("├── ")
            }
            else if read_dir_iter.peek().is_none() && debug_opts.is_some() && *debug_opts.unwrap() == DebugOpts::Tree {
                preceding_str.push_str("└── ") 
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
            if line_to_log {
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
                self.search_directory_path(directory_path, test, debug_opts, Some(preceding_str_2), Some(current_depth + 1));
            }
        }
    }
}

#[cfg(test)]
use mockall::{automock, mock, predicate::*};
use tempfile::Builder;
use tempfile::TempDir;
use tempfile::NamedTempFile;


struct TestLogger {
    logs: Vec<Line>
}

impl TestLogger {
    fn new() -> TestLogger {
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

    fn get_logs_by_file_descriptor(&self, file_descriptor: FileDescriptor) -> Vec<&Line> {
        let logs = &self.logs;
        let logs_iter = logs.into_iter();
        logs_iter.filter(move |&x| {
            x.file_descriptor == Some(file_descriptor)
        }).collect()
    }

    fn get_lines_from_logs_where_logs_contains_provided_value(lines: Vec<&Line>, line_to_find: String) -> bool {
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



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_file_in_same_directory() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir()); 
        let params = Params {
            symlink_setting: SymLinkSetting::Never,
            debug_opts: None,
            optimisation_level: None
        };
        let logger = Rc::new(Mutex::new(TestLogger::new()));
        let searcher = Searcher::new(params, None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());

        // Create a file inside of `env::temp_dir()`.
        let file = NamedTempFile::new()?;
        let file_name_with_extension = file.path().file_name().unwrap().to_str().unwrap().to_string();
        let test_by_name = Test::Name(file_name_with_extension.clone());

        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), &test_by_name, None, None, None);
        
        // Assert  
        let logs = logger.lock().unwrap();
        let stdout_logs = logs.get_logs_by_file_descriptor(FileDescriptor::StdOut);
        assert!(TestLogger::get_lines_from_logs_where_logs_contains_provided_value(stdout_logs.clone(), file.path().to_str().unwrap().to_string()),
            "{}", format!("expected to find {} in logs, but the string could not be found. Full logs: \n{:#?}", file_name_with_extension, stdout_logs));

        // Teardown
        drop(file);
        Ok(())
    }

    #[test]
    fn find_file_in_child_directory() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        const FILE_NAME_WITH_EXTENSION: &'static str = "find_file_in_child_directory.txt";
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());
        let params = Params {
            symlink_setting: SymLinkSetting::Never,
            debug_opts: None,
            optimisation_level: None
        };
        let logger = Rc::new(Mutex::new(TestLogger::new()));
        let searcher = Searcher::new(params, None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());
        
        // Create a directory inside of `env::temp_dir()`
        let directory = TempDir::new()?;
        let file_path = directory.path().join(FILE_NAME_WITH_EXTENSION);
        
        // Create a file inside of the newly created directory
        let tmp_file = File::create(file_path.clone())?;
        let test_by_name = Test::Name(FILE_NAME_WITH_EXTENSION.to_string());

        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), &test_by_name, None, None, None);

        // Assert
        let logs = logger.lock().unwrap();
        let stdout_logs = logs.get_logs_by_file_descriptor(FileDescriptor::StdOut);
        assert!(TestLogger::get_lines_from_logs_where_logs_contains_provided_value(stdout_logs, FILE_NAME_WITH_EXTENSION.to_string()));

        // Teardown
        drop(tmp_file);
        directory.close()?;
        Ok(())        
    }

    #[test]
    fn find_file_in_child_child_directory() -> Result<(), Box<dyn std::error::Error>> {
        const FILE_NAME: &'static str = "find_file_in_child_child_directory";
        const FILE_NAME_WITH_EXTENSION: &'static str = "find_file_in_child_child_directory.txt";

        // Arrange
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());
        let params = Params {
            symlink_setting: SymLinkSetting::Never,
            debug_opts: None,
            optimisation_level: None
        };
        let logger = Rc::new(Mutex::new(TestLogger::new()));
        let searcher = Searcher::new(params, None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());

        let directory = Builder::new().prefix(FILE_NAME).tempdir().unwrap();
        let temp_dir_child = Builder::new().prefix(FILE_NAME).tempdir_in(directory.path()).unwrap();
        let file_path = temp_dir_child.path().join(FILE_NAME_WITH_EXTENSION);
        let tmp_file = File::create(file_path.clone());
        let test_by_name = Test::Name(FILE_NAME_WITH_EXTENSION.to_string());
        
        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), &test_by_name, None, None, None);

        // Assert
        let logs = logger.lock().unwrap();
        let stdout_logs = logs.get_logs_by_file_descriptor(FileDescriptor::StdOut);
        assert!(TestLogger::get_lines_from_logs_where_logs_contains_provided_value(stdout_logs.clone(), FILE_NAME_WITH_EXTENSION.to_string()),
            "{}", format!("expected to find {} in logs, but the string could not be found. Full logs: \n{:#?}", FILE_NAME_WITH_EXTENSION, stdout_logs));

        // Teardown
        drop(tmp_file);
        temp_dir_child.close()?;
        directory.close()?;
        Ok(())
    }

    #[test]
    fn does_not_find_file_in_child_directory_when_max_depth_is_set_to_zero() -> Result<(), Box<dyn std::error::Error>> {
        const FILE_NAME_WITH_EXTENSION: &'static str = "find_file_in_child_directory.txt";

        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());
        let params = Params {
            symlink_setting: SymLinkSetting::Never,
            debug_opts: None,
            optimisation_level: None
        };
        let logger = Rc::new(Mutex::new(TestLogger::new()));
        let searcher = Searcher::new(params, Some(0), logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());

        // Create a directory inside of `env::temp_dir()`
        let directory = TempDir::new()?;
        let file_path = directory.path().join(FILE_NAME_WITH_EXTENSION);
        // Create a file inside of the newly created directory
        let tmp_file = File::create(file_path.clone())?;
        let test_by_name = Test::Name(FILE_NAME_WITH_EXTENSION.to_string());

        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), &test_by_name, None, None, None);

        // Assert
        let logs = logger.lock().unwrap();
        let stdout_logs = logs.get_logs_by_file_descriptor(FileDescriptor::StdOut);
        assert!(!TestLogger::get_lines_from_logs_where_logs_contains_provided_value(stdout_logs.clone(), FILE_NAME_WITH_EXTENSION.to_string()),
            "{}", format!("expected to find {} in logs, but the string could not be found. Full logs: \n{:#?}", FILE_NAME_WITH_EXTENSION, stdout_logs));

        // Teardown
        drop(tmp_file);
        directory.close()?;
        Ok(())
    }

    #[test]
    fn does_not_find_file_in_child_child_directory_when_max_depth_is_set_to_one() -> Result<(), Box<dyn std::error::Error>> {
        const FILE_NAME: &'static str = "find_file_in_child_child_directory";
        const FILE_NAME_WITH_EXTENSION: &'static str = "find_file_in_child_child_directory.txt";
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());
        let params = Params {
            symlink_setting: SymLinkSetting::Never,
            debug_opts: None,
            optimisation_level: None
        };
        let logger = Rc::new(Mutex::new(TestLogger::new()));
        let searcher = Searcher::new(params, Some(1), logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());

        let directory = Builder::new().prefix(FILE_NAME).tempdir().unwrap();
        let temp_dir_child = Builder::new().prefix(FILE_NAME).tempdir_in(directory.path()).unwrap();
        let file_path = temp_dir_child.path().join(FILE_NAME_WITH_EXTENSION);
        let tmp_file = File::create(file_path.clone());
        let test_by_name = Test::Name(FILE_NAME_WITH_EXTENSION.to_string());
        
        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), &test_by_name, None, None, None);

        // Assert
        let logs = logger.lock().unwrap();
        let stdout_logs = logs.get_logs_by_file_descriptor(FileDescriptor::StdOut);
        assert!(!TestLogger::get_lines_from_logs_where_logs_contains_provided_value(stdout_logs.clone(), FILE_NAME_WITH_EXTENSION.to_string()),
            "{}", format!("expected to find {} in logs, but the string could not be found. Full logs: \n{:#?}", FILE_NAME_WITH_EXTENSION, stdout_logs));

        // Teardown
        drop(tmp_file);
        temp_dir_child.close()?;
        directory.close()?;
        Ok(())
    }

    #[test]
    fn does_not_follow_symbolic_links_by_default() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange 
        const FILE_NAME_WITH_EXTENSION: &'static str = "does_not_follow_symbolic_links_by_default.txt";
        let current_directory = TempDir::new()?;
        let directory_of_link = TempDir::new()?;
        let working_directory_before_test = std::env::current_dir().unwrap();
        assert!(std::env::set_current_dir(current_directory.path()).is_ok());

        let original_file_path = current_directory.path().join(FILE_NAME_WITH_EXTENSION);
        let original_file = File::create(original_file_path.clone())?;

        let directory_of_link_path = directory_of_link.path().join("symlink");
        std::os::unix::fs::symlink(&original_file_path, directory_of_link_path.clone())?;
        
        let logger = Rc::new(Mutex::new(TestLogger::new()));

        let params = Params {
            symlink_setting: SymLinkSetting::Never,
            debug_opts: None,
            optimisation_level: None
        };

        let searcher = Searcher::new(params, None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string()); 
        let test_by_name = Test::Name(FILE_NAME_WITH_EXTENSION.to_string());
        
        // Act
        searcher.search_directory_path(current_directory.path(), &test_by_name, None, None, None);

        // Assert
        let logs = logger.lock().unwrap();
        let stdout_logs = logs.get_logs_by_file_descriptor(FileDescriptor::StdOut);
        assert!(TestLogger::get_lines_from_logs_where_logs_contains_provided_value(stdout_logs.clone(), FILE_NAME_WITH_EXTENSION.to_string()),
            "{}", format!("expected to find {} in logs, but the string could not be found. Full logs: \n{:#?}", FILE_NAME_WITH_EXTENSION, stdout_logs));

        // Teardown
        current_directory.close()?;
        directory_of_link.close()?;
        let _ = std::env::set_current_dir(working_directory_before_test)?;
        drop(original_file);
        Ok(())
    }

    #[test]
    fn follows_symlink_when_set_to_follow() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        const FILE_NAME_WITH_EXTENSION: &'static str = "follows_symlink_when_set_to_follow.txt";
        let directory_of_file = TempDir::new()?;
        let directory_of_link = TempDir::new()?;
        // let working_directory_before_test = std::env::current_dir().unwrap();
        assert!(std::env::set_current_dir(directory_of_link.path()).is_ok());

        let original_file_path = directory_of_file.path().join(FILE_NAME_WITH_EXTENSION);
        let original_file = File::create(original_file_path.clone())?;

        let directory_of_link_path = directory_of_link.path().join("symlink");
        std::os::unix::fs::symlink(&original_file_path, directory_of_link_path.clone())?;
        
        let logger = Rc::new(Mutex::new(TestLogger::new()));

        let params = Params {
            symlink_setting: SymLinkSetting::Follow,
            debug_opts: None,
            optimisation_level: None
        };

        let searcher = Searcher::new(params, None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());
        let test_by_name = Test::Name(FILE_NAME_WITH_EXTENSION.to_string());
        
        // Act
        searcher.search_directory_path(directory_of_link.path(), &test_by_name, None, None, None);

        // Assert
        let logs = logger.lock().unwrap();
        let stdout_logs = logs.get_logs_by_file_descriptor(FileDescriptor::StdOut);
        assert!(TestLogger::get_lines_from_logs_where_logs_contains_provided_value(stdout_logs.clone(), FILE_NAME_WITH_EXTENSION.to_string()),
            "{}", format!("expected to find {} in logs, but the string could not be found. Full logs: \n{:#?}", FILE_NAME_WITH_EXTENSION, stdout_logs));

        // Teardown
        directory_of_file.close()?;
        directory_of_link.close()?;
        // let _ = std::env::set_current_dir(working_directory_before_test)?;
        drop(original_file);
        Ok(())
    }

    #[test]
    fn handle_broken_symlink() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        const FILE_NAME_WITH_EXTENSION: &'static str = "handle_broken_symlink.txt";
        let directory_of_file = TempDir::new()?;
        let directory_of_link = TempDir::new()?;
        // let working_directory_before_test = std::env::current_dir().unwrap();
        assert!(std::env::set_current_dir(directory_of_link.path()).is_ok());

        let original_file_path = directory_of_file.path().join(FILE_NAME_WITH_EXTENSION);
        let original_file = File::create(original_file_path.clone())?;

        let directory_of_link_path = directory_of_link.path().join("symlink");
        std::os::unix::fs::symlink(&original_file_path, directory_of_link_path.clone())?;

        //delete the original file to create a broken symlink
        std::fs::remove_file(original_file_path.clone())?;
        
        let logger = Rc::new(Mutex::new(TestLogger::new()));

        let params = Params {
            symlink_setting: SymLinkSetting::Follow,
            debug_opts: None,
            optimisation_level: None
        };

        let searcher = Searcher::new(params, None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());
        let test_by_name = Test::Name(FILE_NAME_WITH_EXTENSION.to_string());
        
        // Act
        searcher.search_directory_path(directory_of_link.path(), &test_by_name, None, None, None);

        // Assert
        let logs = logger.lock().unwrap();
        let stdout_logs = logs.get_logs_by_file_descriptor(FileDescriptor::StdOut);
        assert!(TestLogger::get_lines_from_logs_where_logs_contains_provided_value(stdout_logs.clone(), FILE_NAME_WITH_EXTENSION.to_string()),
            "{}", format!("expected to find {} in logs, but the string could not be found. Full logs: \n{:#?}", FILE_NAME_WITH_EXTENSION, stdout_logs));

        // Teardown
        directory_of_file.close()?;
        directory_of_link.close()?;
        // let _ = std::env::set_current_dir(working_directory_before_test)?;
        drop(original_file);
        Ok(())

    }

    #[test]
    fn check_debug_opts_tree_logs_correctly() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        let temp = assert_fs::TempDir::new().unwrap();
        let inputs_files = vec![temp.child("foo1.txt"), temp.child("foo2.txt"), temp.child("check_debug_opts_tree_logs_correctly_sub_dir/foo3.txt"), temp.child("check_debug_opts_tree_logs_correctly_sub_dir/foo4.txt")];
        for input_file in inputs_files {
            input_file.touch()?;
        }

        let logger = Rc::new(Mutex::new(TestLogger::new()));

        let params = Params {
            symlink_setting: SymLinkSetting::Never,
            debug_opts: None,
            optimisation_level: None
        };

        let debug_opts = DebugOpts::Exec;

        let searcher = Searcher::new(params, None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());
        let test = Test::Name("foo4.txt".to_string());

        // Act
        searcher.search_directory_path(temp.path(), &test, Some(&debug_opts), None, None);

        // Assert
        // let stdout_logs = logger.lock().unwrap().get_logs_by_type(discriminant(&LogLine::StdOut(String::new())));
        let logs = logger.lock().unwrap();
        let stdout_logs = logs.get_logs_by_file_descriptor(FileDescriptor::StdOut);
        assert!(TestLogger::get_lines_from_logs_where_logs_contains_provided_value(stdout_logs, "foo4.txt".to_string()));
        
        // logs.into_iter().filter(|predicate| {
            // *predicate.line.
        // });
        // assert!(logs.ends_with(&LogLine::StdOut(format!("foo4.txt"))));
        // Teardown
        Ok(())
    }
}

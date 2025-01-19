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

use rfind::main::*;

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
        .arg(Arg::new("min_depth")
            .value_parser(value_parser!(u32))
            .long("mindepth")
            .action(ArgAction::Set)
            .help("Do not apply any tests or actions at levels less than levels (a  non-negative  integer).
              Using -mindepth 1 means process all files except the starting-points."))
        .arg(Arg::new("name")
            .long("name")
            .help("The name of the file to find")
        )
        .arg(Arg::new("type")
            .long("type")
            .help("The file type of the file to find")
        )
        .allow_missing_positional(true)
        .arg(Arg::new("starting_path")
            .default_value(".")
            .value_parser(value_parser!(String))
        )
        .arg(Arg::new("expression")
            .default_value("--true")
            .num_args(0..)
            .value_parser(value_parser!(String))
        )
        .get_matches();

    // parse the cmd arguments
    let mut symlink_setting: SymLinkSetting = SymLinkSetting::Never;

    if matches.get_flag("symlink_only_command_line_args") {
        symlink_setting = SymLinkSetting::OnlyCommandLineArgs;
    }

    if matches.get_flag("symlink_follow") {
        symlink_setting = SymLinkSetting::Follow;
    }

    if matches.get_flag("symlink_never") {
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

    let min_depth = matches.remove_one::<u32>("min_depth");

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
    
    let searcher = Searcher::<StandardLogger>::new(params, max_depth, min_depth, Rc::new(Mutex::new(logger)), starting_path.unwrap_or(format!(".")));
    println!("{:#?}", searcher);
    eval(expression, searcher);
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
    println!("TOKENS: {:#?}", tokens);

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
    let directory_path = Path::new(&searcher.starting_path);
    for (i, el) in iter.enumerate() {
        if el == "--true" {
            expression_result = true;
        }
        if el == "--false" {
            expression_result = false;
        }
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
        if el == "--name" { // todo maybe make one if statement for all tests?
            let name: String = tokens.get(i + 1).expect("--name expects a file name, but found nothing").clone();
            
            ex.expression_str = Some(Box::new(vec![el.to_string(), name.clone()]));
            let test = Test::Name(name.clone());
            searcher.search_directory_path(directory_path, &test, None, None);
            expression_result = some_test_returns_true(*ex.expression_str.unwrap());
        }
        else if el == "--type" {
            let r#type: String = tokens.get(i + 1).expect("--type expects a file type, but found nothing").clone();
            
            ex.expression_str = Some(Box::new(vec![el.to_string(), r#type.clone()]));
            let test = Test::Types(r#type);
            searcher.search_directory_path(directory_path, &test, None, None);
            expression_result = some_test_returns_true(*ex.expression_str.unwrap());
        }
    }
    expression_result
}

#[cfg(test)]
use mockall::{automock, mock, predicate::*};
use tempfile::Builder;
use tempfile::TempDir;
use tempfile::NamedTempFile;




#[cfg(test)]
mod tests {
    use super::*;
    use self::test_case;

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
        let searcher = Searcher::new(params, None, None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());

        // Create a file inside of `env::temp_dir()`.
        let file = NamedTempFile::new()?;
        let file_name_with_extension = file.path().file_name().unwrap().to_str().unwrap().to_string();
        let test_by_name = Test::Name(file_name_with_extension.clone());

        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), &test_by_name, None, None);
        
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
        let searcher = Searcher::new(params, None, None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());
        
        // Create a directory inside of `env::temp_dir()`
        let directory = TempDir::new()?;
        let file_path = directory.path().join(FILE_NAME_WITH_EXTENSION);
        
        // Create a file inside of the newly created directory
        let tmp_file = File::create(file_path.clone())?;
        let test_by_name = Test::Name(FILE_NAME_WITH_EXTENSION.to_string());

        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), &test_by_name, None, None);

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
    fn find_file_in_child_child_directory_2() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        const CHILD_DIR: &'static str = "child_dir";
        const CHILD_FILE: &'static str = "child_file.txt";
        const CHILD_FILE_REL_PATH: &'static str = "child_dir/child_file.txt";


        let temp_dir = assert_fs::TempDir::new()?;
        std::env::set_current_dir(temp_dir.path())?;

        let logger = Rc::new(Mutex::new(TestLogger::new()));

        let params = Params {
            symlink_setting: SymLinkSetting::Never,
            debug_opts: None,
            optimisation_level: None
        };

        let searcher = Searcher::new(params, None, None, logger.clone(), temp_dir.path().to_str().unwrap().to_string());
        let temp_file = temp_dir.child(CHILD_FILE_REL_PATH).touch();

        let test_by_name = Test::Name(CHILD_FILE.to_string());
        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), &test_by_name, None, None);

        // Assert
        let logs = logger.lock().unwrap();
        let stdout_logs = logs.get_logs_by_file_descriptor(FileDescriptor::StdOut);

        println!("logs: {:#?}", stdout_logs);
        assert!(TestLogger::get_lines_from_logs_where_logs_contains_provided_value(stdout_logs.clone(), CHILD_FILE.to_string()),
            "{}", format!("expected to find {} in logs, but the string could not be found. Full logs: \n{:#?}", CHILD_FILE, stdout_logs));
        // Teardown
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
        let searcher = Searcher::new(params, None, None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());

        let directory = Builder::new().prefix(FILE_NAME).tempdir().unwrap();
        let temp_dir_child = Builder::new().prefix(FILE_NAME).tempdir_in(directory.path()).unwrap();
        let file_path = temp_dir_child.path().join(FILE_NAME_WITH_EXTENSION);
        let tmp_file = File::create(file_path.clone());
        let test_by_name = Test::Name(FILE_NAME_WITH_EXTENSION.to_string());
        
        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), &test_by_name, None, None);

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
        let searcher = Searcher::new(params, Some(0), None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());

        // Create a directory inside of `env::temp_dir()`
        let directory = TempDir::new()?;
        let file_path = directory.path().join(FILE_NAME_WITH_EXTENSION);
        // Create a file inside of the newly created directory
        let tmp_file = File::create(file_path.clone())?;
        let test_by_name = Test::Name(FILE_NAME_WITH_EXTENSION.to_string());

        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), &test_by_name, None, None);

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
        let searcher = Searcher::new(params, Some(1), None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());

        let directory = Builder::new().prefix(FILE_NAME).tempdir().unwrap();
        let temp_dir_child = Builder::new().prefix(FILE_NAME).tempdir_in(directory.path()).unwrap();
        let file_path = temp_dir_child.path().join(FILE_NAME_WITH_EXTENSION);
        let tmp_file = File::create(file_path.clone());
        let test_by_name = Test::Name(FILE_NAME_WITH_EXTENSION.to_string());
        
        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), &test_by_name, None, None);

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
    fn does_not_find_file_in_current_directory_when_min_depth_is_set_to_one() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        const FILE_NAME_WITH_EXTENSION: &'static str = "does_not_find_file_in_current_directory_when_min_depth_is_set_to_one.txt";
        let temp = assert_fs::TempDir::new()?;
        std::env::set_current_dir(temp.path())?;

        let logger = Rc::new(Mutex::new(TestLogger::new()));

        let params = Params {
            symlink_setting: SymLinkSetting::Never,
            debug_opts: None,
            optimisation_level: None
        };

        let searcher = Searcher::new(params, None, Some(1), logger.clone(), temp.path().to_str().unwrap().to_string());
        temp.child(FILE_NAME_WITH_EXTENSION).touch()?;
        let test_by_name = Test::Name(FILE_NAME_WITH_EXTENSION.to_string());

        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), &test_by_name, None, None);

        // Assert
        let logs = logger.lock().unwrap();
        let stdout_logs = logs.get_logs_by_file_descriptor(FileDescriptor::StdOut);
        assert!(!TestLogger::get_lines_from_logs_where_logs_contains_provided_value(stdout_logs.clone(), FILE_NAME_WITH_EXTENSION.to_string()),
            "{}", format!("expected to find {} in logs, but the string could not be found. Full logs: \n{:#?}", FILE_NAME_WITH_EXTENSION, stdout_logs));

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

        let searcher = Searcher::new(params, None, None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string()); 
        let test_by_name = Test::Name(FILE_NAME_WITH_EXTENSION.to_string());
        
        // Act
        searcher.search_directory_path(current_directory.path(), &test_by_name, None, None);

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

        let searcher = Searcher::new(params, None, None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());
        let test_by_name = Test::Name(FILE_NAME_WITH_EXTENSION.to_string());
        
        // Act
        searcher.search_directory_path(directory_of_link.path(), &test_by_name, None, None);

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

        let searcher = Searcher::new(params, None, None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());
        let test_by_name = Test::Name(FILE_NAME_WITH_EXTENSION.to_string());
        
        // Act
        searcher.search_directory_path(directory_of_link.path(), &test_by_name, None, None);

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
            debug_opts: Some(DebugOpts::Exec),
            optimisation_level: None
        };


        let searcher = Searcher::new(params, None, None, logger.clone(), tempfile::env::temp_dir().to_str().unwrap().to_string());
        let test = Test::Name("foo4.txt".to_string());

        // Act
        searcher.search_directory_path(temp.path(), &test, None, None);

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

    #[test]
    fn checks_only_files_and_directories_that_are_empty() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        let temp = assert_fs::TempDir::new()?;
        std::env::set_current_dir(temp.path())?;

        let empty_dir = temp.child("empty_dir/").touch();
        let empty_file = temp.child("empty_file.txt").touch();

        let populated_dir = temp.child("populated_dir/");
        let populated_file = temp.child("populated_dir/populated_file.txt").write_str("some data");

        let logger = Rc::new(Mutex::new(TestLogger::new()));

        let params = Params {
            symlink_setting: SymLinkSetting::Never,
            debug_opts: None,
            optimisation_level: None
        };

        // todo add more
        let test = Test::Name("empty_file.txt".to_string());
        
        let searcher = Searcher::new(params, None, None, logger.clone(), std::env::current_dir().unwrap().to_str().unwrap().to_string());
        searcher.search_directory_path(temp.path(), &test, None, None);
        
        Ok(())
    }

    #[test_case("--false", "--false", false ; "Expect false when both operands are false")]
    #[test_case("--false", "--true", false ; "Expect false when first operand is false and second operand is true")]
    #[test_case("--true", "--false", false ; "Expect false when first operand is true and second operand is false")]
    #[test_case("--true", "--true", true ; "Expect true when both operands are true")]
    fn check_and_operator_works(first_operand: &str, second_operand: &str, expected: bool) -> Result<(), Box<dyn std::error::Error>> {
        let temp = assert_fs::TempDir::new()?;
        std::env::set_current_dir(temp.path())?;

        let logger = Rc::new(Mutex::new(TestLogger::new()));

        let params = Params {
            symlink_setting: SymLinkSetting::Never,
            debug_opts: None,
            optimisation_level: None
        };
        let searcher = Searcher::new(params, None, None, logger.clone(), temp.path().to_str().unwrap().to_string());

        let operator = format!("--and");
        let tokens = [ first_operand.to_owned(), operator, second_operand.to_owned() ].to_vec();

        assert_eq!(eval(tokens, searcher), expected);
        Ok(())
    }

    #[test_case("--false", "--false", false ; "Expect false when both operands are false")]
    #[test_case("--false", "--true", true ; "Expect true when first operand is false and second operand is true")]
    #[test_case("--true", "--false", true ; "Expect true when first operand is true and second operand is false")]
    #[test_case("--true", "--true", true ; "Expect true when both operands are true")]
    fn check_or_operator_works(first_operand: &str, second_operand: &str, expected: bool) -> Result<(), Box<dyn std::error::Error>> {
        let temp = assert_fs::TempDir::new()?;
        std::env::set_current_dir(temp.path())?;

        let logger = Rc::new(Mutex::new(TestLogger::new()));

        let params = Params {
            symlink_setting: SymLinkSetting::Never,
            debug_opts: None,
            optimisation_level: None
        };
        let searcher = Searcher::new(params, None, None, logger.clone(), temp.path().to_str().unwrap().to_string());

        let operator = format!("--or");
        let tokens = [ first_operand.to_owned(), operator, second_operand.to_owned() ].to_vec();

        assert_eq!(eval(tokens, searcher), expected);
        Ok(())
    }

    #[test_case("--true", false ; "Expect false when operand is true")]
    #[test_case("--false", true ; "Expect true when operand is false")]
    fn check_not_operator_works(operand: &str, expected: bool) -> Result<(), Box<dyn std::error::Error>> {
        let temp = assert_fs::TempDir::new()?;
        std::env::set_current_dir(temp.path())?;

        let logger = Rc::new(Mutex::new(TestLogger::new()));
        
        let params = Params {
            symlink_setting: SymLinkSetting::Never,
            debug_opts: None,
            optimisation_level: None
        };
        let searcher = Searcher::new(params, None, None, logger.clone(), temp.path().to_str().unwrap().to_string());

        let operator = format!("--not");
        let tokens = [operator, operand.to_owned()].to_vec();

        assert_eq!(eval(tokens, searcher), expected);
        Ok(())
    }
}

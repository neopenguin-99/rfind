use std::io;
use std::mem::{discriminant, Discriminant};
use std::io::{Write, Read, Seek, SeekFrom};
use std::fs;
use std::path::Path;
use std::process::exit;
use std::env;
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command, ValueEnum};
use std::fs::File;
use speculoos::prelude::*;

fn main() {
    let mut logger = StandardLogger::new();

    let matches: ArgMatches = Command::new("MyApp")
        // .arg(Arg::new("debugopts")
            // .value_parser(value_parser!(SymLinkSetting))
            // .short('D')
            // .action(ArgAction::Set)
            // .help("")
        // )
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
        .arg(Arg::new("version")
            .short('v')
            .long("version")
            .action(ArgAction::SetTrue)
            .help("Gets the current version of rfind")
        )
        .arg(Arg::new("starting_path")
            .action(ArgAction::Set)
        )
        .arg(Arg::new("maxdepth")
            .value_parser(value_parser!(u32))
            .long("maxdepth")
            .action(ArgAction::Set)
            .help("Descend at most the provided number of levels, this value must be a non-negative integer.
            Using max depth of 0 will apply the expression 
            for files only in the current directory, and will not search subdirectories")
        )
        .arg(Arg::new("name")
            .long("name")
            .action(ArgAction::Set)
            .help("Base of file name (the path with the leading directories removed) matches shell pattern")
        ).get_matches();

    // parse the cmd arguments
    let mut symlink_setting: SymLinkSetting = SymLinkSetting::Never;

    if matches.get_one::<bool>("symlink_only_command_line_args").is_some() {
        symlink_setting = SymLinkSetting::OnlyCommandLineArgs;
    }

    if matches.get_one::<bool>("symlink_follow").is_some() {
        symlink_setting = SymLinkSetting::Follow;
    }

    if matches.get_one::<bool>("symlink_never").is_some() {
        symlink_setting = SymLinkSetting::Never;
    }

    match matches.get_one::<bool>("version") {
        Some(c) if *c => {
            let line = format!("Version: {}", env!("CARGO_PKG_VERSION"));
            logger.log(LogLine::StdOut(line));
            exit(0);
        },
        _ => ()
    }

    let max_depth = matches.get_one::<u32>("maxdepth");

    let starting_path: &str = match matches.get_one::<String>("starting_path") {
        Some(x) => x,
        _ => "."
    };

    let name: &str = match matches.get_one::<String>("name") {
        Some(x) => x,
        _ => "*"
    };

    // todo log as verbose:
    // println!("Starting_path: {}", starting_path);
    // println!("Name: {}", name);

    let searcher = Searcher::new(max_depth.copied(), symlink_setting);
    searcher.search_directory_path(Path::new(starting_path), name, &mut logger, None);
}

#[derive(Debug, Clone, PartialEq)]
enum SymLinkSetting {
    Never,
    Follow,
    OnlyCommandLineArgs
}

// impl ValueEnum for SymLinkSetting {
    // fn value_variants<'a>() -> &'a [Self] {
    // }
// }


#[derive(Clone, Debug, PartialEq)]
enum LogLine {
    StdOut(String),
    StdErr(String)
}

trait Logger {
    fn log(&mut self, line_to_log: LogLine);
}

struct StandardLogger { }

impl StandardLogger {
    fn new() -> StandardLogger {
        StandardLogger {
        }
    }
}

impl Logger for StandardLogger {
    fn log(&mut self, line_to_log: LogLine) {
        _ = match line_to_log {
            LogLine::StdOut(line) => println!("{}", line),
            LogLine::StdErr(line) => eprintln!("{}", line),
        }
    }
}

struct Searcher {
    max_depth: Option<u32>,
    symlink_setting: SymLinkSetting
}

impl Searcher {
    pub fn new(max_depth: Option<u32>, symlink_setting: SymLinkSetting) -> Searcher {
        Searcher {
            max_depth,
            symlink_setting
        }
    }

    pub fn search_directory_path<T: Logger>(&self, directory_path: &Path, name: &str, logger: &mut T, current_depth: Option<u32>) {
        let current_depth = current_depth.unwrap_or(0);
        let read_dir = match fs::read_dir(directory_path) {
            Ok(res) => {
                res
            }
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied => {
                println!("find: Permission denied for dir name {:#?}", directory_path);
                let line = format!("find: Permission denied for directory name {}", directory_path.to_str().unwrap());
                logger.log(LogLine::StdErr(line));
                return;
            }
            Err(_) => {
                let line = format!("An error occurred when attempting to read the {} directory", directory_path.to_str().unwrap());
                logger.log(LogLine::StdErr(line));
                return;
            }
        };
        for ele in read_dir.into_iter() {
            let ele = ele.unwrap();
            let file_name = ele.file_name();
            let file_type = ele.file_type().unwrap();

            if file_type.is_symlink() && self.symlink_setting == SymLinkSetting::Follow {
                // navigate to the file pointed to by the symlink
                let file_referred_to_by_symlink = fs::read_link(ele.path());
                
                _ = match file_referred_to_by_symlink {
                    Ok(file_referred_to_by_symlink_unwrapped) => {
                        logger.log(LogLine::StdOut(file_referred_to_by_symlink_unwrapped.to_str().unwrap().to_string()));
                        continue;
                    }
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {
                        logger.log(LogLine::StdErr(format!("Broken symlink: {}", ele.path().to_str().unwrap().to_string())));
                        continue;
                    }
                    Err(_) => {
                        unreachable!("We have handled both cases where read_link would result in an error, so this should be unreachable");
                    }
                }
            }
            if file_name == name {
                logger.log(LogLine::StdOut(directory_path.join(name).to_str().unwrap().to_string()));
                continue;
            }
            if file_type.is_dir() && (self.max_depth.is_some() && current_depth < self.max_depth.unwrap() || self.max_depth.is_none()) {
                let file_name = ele.file_name();
                let file_name: &str = file_name.to_str().unwrap();
                let directory_path = directory_path.join(file_name);
                let directory_path = directory_path.as_path();
                self.search_directory_path(directory_path, name, logger, Some(current_depth + 1));
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
    log: Vec<LogLine>
}

impl TestLogger {
    fn new() -> TestLogger {
        TestLogger {
            log: Vec::new()
        }
    }

    fn get_logs(&self) -> &Vec<LogLine> {
        &self.log
    }

    fn is_enum_variant(value: &LogLine, d: Discriminant<LogLine>) -> bool {
        if discriminant(value) == d {
            return true;
        }
        return false;
    }

    fn get_logs_by_type(&self, d: Discriminant<LogLine>) -> Vec<LogLine> { //use some
        //rust wizardry to make this function better (remove the .clone)
        let log_iter = self.log.clone();
        log_iter.into_iter().filter(|x| Self::is_enum_variant(x, d)).collect::<Vec<LogLine>>()
    }
}

impl Logger for TestLogger {
    fn log(&mut self, line_to_log: LogLine) {
        self.log.push(line_to_log);
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_file_in_same_directory() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());
        let searcher = Searcher::new(None, SymLinkSetting::Never);

        // Create a file inside of `env::temp_dir()`.
        let file = NamedTempFile::new()?;
        let file_name = file.path().file_name().unwrap().to_str().unwrap();
        let mut logger = TestLogger::new();

        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), file_name, &mut logger, None);
        
        // Assert 
        let a = logger.get_logs_by_type(discriminant(&LogLine::StdOut(String::new()))); //todo why
        // do we need to pass in String::new here to get it to compile?????????????
        assert!(a.contains(&LogLine::StdOut(file.path().to_str().unwrap().to_string())));

        // Teardown
        drop(file);
        Ok(())
    }

    #[test]
    fn find_file_in_child_directory() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());
        let searcher = Searcher::new(None, SymLinkSetting::Never);
        
        // Create a directory inside of `env::temp_dir()`
        let directory = TempDir::new()?;
        let file_path = directory.path().join("find_file_in_child_directory.txt");
        
        // Create a file inside of the newly created directory
        let tmp_file = File::create(file_path.clone())?;
        let mut logger = TestLogger::new();

        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), "find_file_in_child_directory.txt", &mut logger, None);

        // Assert
        let stdout_logs = logger.get_logs_by_type(discriminant(&LogLine::StdOut(String::new())));
        assert_that(stdout_logs.first().unwrap()).is_equal_to(&LogLine::StdOut(file_path.to_str().unwrap().to_string()));
        //assert!(stdout_logs.contains(&LogLine::StdOut(file_path.to_str().unwrap().to_string())));

        // Teardown
        drop(tmp_file);
        directory.close()?;
        Ok(())        
    }

    #[test]
    fn find_file_in_child_child_directory() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());
        let searcher = Searcher::new(None, SymLinkSetting::Never); 

        let directory = Builder::new().prefix("find_file_in_child_child_directory").tempdir().unwrap();
        let temp_dir_child = Builder::new().prefix("find_file_in_child_child_directory").tempdir_in(directory.path()).unwrap();
        let file_path = temp_dir_child.path().join("find_file_in_child_child_directory.txt");
        let tmp_file = File::create(file_path.clone());
        let mut logger = TestLogger::new();
        
        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), "find_file_in_child_child_directory.txt", &mut logger, None);

        // Assert
        let stdout_logs = logger.get_logs_by_type(discriminant(&LogLine::StdOut(String::new())));
        assert!(stdout_logs.contains(&LogLine::StdOut(file_path.to_str().unwrap().to_string())));


        // Teardown
        drop(tmp_file);
        temp_dir_child.close()?;
        directory.close()?;
        Ok(())
    }

    #[test]
    fn does_not_find_file_in_child_directory_when_max_depth_is_set_to_zero() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());
        let searcher = Searcher::new(Some(0), SymLinkSetting::Never);

        // Create a directory inside of `env::temp_dir()`
        let directory = TempDir::new()?;
        let file_path = directory.path().join("find_file_in_child_directory.txt");
        // Create a file inside of the newly created directory
        let tmp_file = File::create(file_path.clone())?;
        let mut logger = TestLogger::new();

        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), "find_file_in_child_directory.txt", &mut logger, None);

        // Assert
        let stdout_logs = logger.get_logs_by_type(discriminant(&LogLine::StdOut(String::new())));
        assert!(!stdout_logs.contains(&LogLine::StdOut(file_path.to_str().unwrap().to_string())));

        // Teardown
        drop(tmp_file);
        directory.close()?;
        Ok(())
    }

    #[test]
    fn does_not_find_file_in_child_child_directory_when_max_depth_is_set_to_one() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());
        let searcher = Searcher::new(Some(1), SymLinkSetting::Never);

        let directory = Builder::new().prefix("find_file_in_child_child_directory").tempdir().unwrap();
        let temp_dir_child = Builder::new().prefix("find_file_in_child_child_directory").tempdir_in(directory.path()).unwrap();
        let file_path = temp_dir_child.path().join("find_file_in_child_child_directory.txt");
        let tmp_file = File::create(file_path.clone());
        let mut logger = TestLogger::new();
        
        // Act
        searcher.search_directory_path(tempfile::env::temp_dir().as_path(), "find_file_in_child_child_directory.txt", &mut logger, None);

        // Assert
        let stdout_logs = logger.get_logs_by_type(discriminant(&LogLine::StdOut(String::new())));
        assert!(!stdout_logs.contains(&LogLine::StdOut(file_path.to_str().unwrap().to_string())));

        // Teardown
        drop(tmp_file);
        temp_dir_child.close()?;
        directory.close()?;
        Ok(())
    }

    #[test]
    fn does_not_follow_symbolic_links_by_default() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        let current_directory = TempDir::new()?;
        let directory_of_link = TempDir::new()?;
        let working_directory_before_test = std::env::current_dir().unwrap();
        assert!(std::env::set_current_dir(current_directory.path()).is_ok());

        let original_file_path = current_directory.path().join("does_not_follow_symbolic_links_by_default.txt");
        let original_file = File::create(original_file_path.clone())?;

        let directory_of_link_path = directory_of_link.path().join("symlink");
        std::os::unix::fs::symlink(&original_file_path, directory_of_link_path.clone())?;
        
        let mut logger = TestLogger::new();
        let searcher = Searcher::new(None, SymLinkSetting::Never);
        
        // Act
        searcher.search_directory_path(current_directory.path(), "does_not_follow_symbolic_links_by_default.txt", &mut logger, None);

        // Assert
         let stdout_logs = logger.get_logs_by_type(discriminant(&LogLine::StdOut(String::new())));
         assert!(!stdout_logs.contains(&LogLine::StdOut(directory_of_link_path.to_str().unwrap().to_string())));

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
        let directory_of_file = TempDir::new()?;
        let directory_of_link = TempDir::new()?;
        // let working_directory_before_test = std::env::current_dir().unwrap();
        assert!(std::env::set_current_dir(directory_of_link.path()).is_ok());

        let original_file_path = directory_of_file.path().join("follows_symlink_when_set_to_follow.txt");
        let original_file = File::create(original_file_path.clone())?;

        let directory_of_link_path = directory_of_link.path().join("symlink");
        std::os::unix::fs::symlink(&original_file_path, directory_of_link_path.clone())?;
        
        let mut logger = TestLogger::new();
        let searcher = Searcher::new(None, SymLinkSetting::Follow);
        
        // Act
        searcher.search_directory_path(directory_of_link.path(), "follows_symlink_when_set_to_follow.txt", &mut logger, None);

        // Assert
        let stdout_logs = logger.get_logs_by_type(discriminant(&LogLine::StdOut(String::new())));

        eprintln!("stdout logs {:#?}", stdout_logs);
        eprintln!("file path {:#?}", original_file_path.to_str().unwrap().to_string());
        eprintln!("link path {:#?}", directory_of_link_path.to_str().unwrap().to_string());
        eprintln!("env: {}", std::env::current_dir().unwrap().to_str().unwrap().to_string());
        assert!(stdout_logs.contains(&LogLine::StdOut(original_file_path.to_str().unwrap().to_string())));

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
        let directory_of_file = TempDir::new()?;
        let directory_of_link = TempDir::new()?;
        // let working_directory_before_test = std::env::current_dir().unwrap();
        assert!(std::env::set_current_dir(directory_of_link.path()).is_ok());

        let original_file_path = directory_of_file.path().join("follows_symlink_when_set_to_follow.txt");
        let original_file = File::create(original_file_path.clone())?;

        let directory_of_link_path = directory_of_link.path().join("symlink");
        std::os::unix::fs::symlink(&original_file_path, directory_of_link_path.clone())?;

        //delete the original file to create a broken symlink
        std::fs::remove_file(original_file_path.clone())?;
        
        let mut logger = TestLogger::new();
        let searcher = Searcher::new(None, SymLinkSetting::Follow);
        
        // Act
        searcher.search_directory_path(directory_of_link.path(), "follows_symlink_when_set_to_follow.txt", &mut logger, None);

        // Assert
        let stdout_logs = logger.get_logs_by_type(discriminant(&LogLine::StdOut(String::new())));

        eprintln!("stdout logs {:#?}", stdout_logs);
        eprintln!("file path {:#?}", original_file_path.to_str().unwrap().to_string());
        eprintln!("link path {:#?}", directory_of_link_path.to_str().unwrap().to_string());
        eprintln!("env: {}", std::env::current_dir().unwrap().to_str().unwrap().to_string());
        assert!(stdout_logs.contains(&LogLine::StdOut(original_file_path.to_str().unwrap().to_string())));

        // Teardown
        directory_of_file.close()?;
        directory_of_link.close()?;
        // let _ = std::env::set_current_dir(working_directory_before_test)?;
        drop(original_file);
        Ok(())

    }
}

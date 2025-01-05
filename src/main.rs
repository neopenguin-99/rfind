use std::io;
use std::mem::{discriminant, Discriminant};
use std::io::{Write, Read, Seek, SeekFrom};
use std::fs;
use std::path::Path;
use std::process::exit;
use std::env;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::fs::File;

fn main() {
    let mut logger = StandardLogger::new();

    let matches: ArgMatches = Command::new("MyApp")
        .arg(Arg::new("symlink")
            .short('P')
            .action(ArgAction::SetTrue)
            .help("Never follow symbolic links.")
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
        .arg(Arg::new("name")
            .long("name")
            .action(ArgAction::Set)
            .help("Base of file name (the path with the leading directories removed) matches shell pattern")
        ).get_matches();
    if let Some(c) = matches.get_one::<bool>("symlink") {
        println!("Value for -c: {c}");
        // todo implement
    }
    match matches.get_one::<bool>("version") {
        Some(c) if *c => {
            let line = format!("Version: {}", env!("CARGO_PKG_VERSION"));
            logger.log(LogLine::StdOut(line));
            exit(0);
        },
        _ => ()
    }

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

    search_directory_path(Path::new(starting_path), name, &mut logger);
}

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


fn search_directory_path<T: Logger>(directory_path: &Path, name: &str, logger: &mut T) {
// fn search_directory_path(directory_path: &str, name: &str) {
    // Check the contents of the current working directory.

    // println!("{:#?}", directory_path);

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
        

        
        if file_name == name {
            logger.log(LogLine::StdOut(directory_path.join(name).to_str().unwrap().to_string()));
            continue;
        }
        let file_type = ele.file_type().unwrap();
        if file_type.is_dir() {

            // let directory_path: &str = directory_path.to_str().unwrap();
            // let file_name = ele.file_name().to_str().unwrap();
            
            // let directory_path = format!("{directory_path}/{file_name}").as_str();
            // search_directory_path(&Path::new(directory_path), name);
            
            let file_name = ele.file_name();
            let file_name: &str = file_name.to_str().unwrap();
            let directory_path = directory_path.join(file_name);
            let directory_path = directory_path.as_path();
            search_directory_path(directory_path, name, logger);

        }
    }
    // If nothing found, get all of the directories in this working directory, and then call this function recursively, but pass in different directory each time
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
        TestLogger::new();

        // Create a file inside of `env::temp_dir()`.
        let file = NamedTempFile::new()?;
        let file_name = file.path().file_name().unwrap().to_str().unwrap();
        let mut logger = TestLogger::new();

        // Act
        search_directory_path(tempfile::env::temp_dir().as_path(), file_name, &mut logger);
        
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
        TestLogger::new();
        
        // Create a directory inside of `env::temp_dir()`
        let directory = TempDir::new()?;
        let file_path = directory.path().join("find_file_in_child_directory.txt");
        // Create a file inside of the newly created directory
        let tmp_file = File::create(file_path.clone())?;
        let mut logger = TestLogger::new();

        // Act
        search_directory_path(tempfile::env::temp_dir().as_path(), "find_file_in_child_directory.txt", &mut logger);

        // Assert
        let stdout_logs = logger.get_logs_by_type(discriminant(&LogLine::StdOut(String::new())));
        assert!(stdout_logs.contains(&LogLine::StdOut(file_path.to_str().unwrap().to_string())));

        // Teardown
        drop(tmp_file);
        directory.close()?;
        Ok(())        
    }

    #[test]
    fn find_file_in_child_child_directory() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());
        let mut logger = TestLogger::new();
        

        let directory = Builder::new().prefix("find_file_in_child_child_directory").tempdir().unwrap();
        let temp_dir_child = Builder::new().prefix("find_file_in_child_child_directory").tempdir_in(directory.path()).unwrap();
        let file_path = temp_dir_child.path().join("find_file_in_child_child_directory.txt");
        let tmp_file = File::create(file_path.clone());
        
        // Act
        search_directory_path(tempfile::env::temp_dir().as_path(), "find_file_in_child_child_directory.txt", &mut logger);

        // Assert
        let stdout_logs = logger.get_logs_by_type(discriminant(&LogLine::StdOut(String::new())));
        assert!(stdout_logs.contains(&LogLine::StdOut(file_path.to_str().unwrap().to_string())));


        // Teardown
        drop(tmp_file);
        temp_dir_child.close()?;
        directory.close()?;
        Ok(())
    }
}

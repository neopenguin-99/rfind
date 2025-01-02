use std::io::{Write, Read, Seek, SeekFrom};
use std::fs;
use std::os::linux::net::SocketAddrExt;
use std::path::Path;
use std::process::exit;
use clap::Parser;
use std::env;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::fs::File;

fn main() {
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
            // expressions
        .arg(Arg::new("name")
            .long("name")
            .action(ArgAction::Set)
            .help("Base of file name (the path with the leading directories removed) matches shell pattern")
        ).get_matches();
    if let Some(c) = matches.get_one::<bool>("symlink") {
        println!("Value for -c: {c}");
    }
    match matches.get_one::<bool>("version") {
        Some(c) if *c => {
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
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

    println!("Starting_path: {}", starting_path);
    println!("Name: {}", name);

    search_working_directory(Path::new(starting_path), name);
}


fn search_working_directory(working_directory: &Path, name: &str) {
// fn search_working_directory(working_directory: &str, name: &str) {
    // Check the contents of the current working directory.
    println!("{:#?}", working_directory);
    let a = fs::read_dir(working_directory).unwrap(); //todo fix unwrap
    for ele in a.into_iter() {
        let ele = ele.unwrap();
        let file_name = ele.file_name();
        println!("In loop! file_name: {}", file_name.to_str().unwrap());
        let metadata = ele.metadata();

        if file_name == name {
            println!("{:#?}/{}", working_directory, name);
        }
        else if ele.file_type().unwrap().is_dir() {

            // let working_directory: &str = working_directory.to_str().unwrap();
            // let file_name = ele.file_name().to_str().unwrap();
            
            // let working_directory = format!("{working_directory}/{file_name}").as_str();
            // search_working_directory(&Path::new(working_directory), name);
            
            let file_name = ele.file_name();
            let file_name: &str = file_name.to_str().unwrap();
            let working_directory = working_directory.join(file_name);
            let working_directory = working_directory.as_path();
            search_working_directory(working_directory, name);

        }
    }
    // If nothing found, get all of the directories in this working directory, and then call this function recursively, but pass in different directory each time
}

#[cfg(test)]
use mockall::{automock, mock, predicate::*};
use tempfile::Builder;
use tempfile::TempDir;
#[cfg_attr(test, automock)]
trait MyTrait {

}

struct SetupInfo {
    path: String
}

impl SetupInfo {
    fn new() -> SetupInfo {
        SetupInfo {
            path: tempfile::env::temp_dir().to_str().unwrap().to_string() // todo remove unwrap
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_file_in_same_directory_2() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());

        // create new temp file
        Ok(())
    }

    #[test]
    fn find_file_in_same_directory() -> Result<(), Box<dyn std::error::Error>> {
        // Arrange
        assert_eq!(tempfile::env::temp_dir(), std::env::temp_dir());
        let test_dir = TempDir::new()?;
        let working_directory = tempfile::env::temp_dir().clone().join(test_dir);

        // let file_name = working_directory.clone();
        // let file_name = file_name.join("find_file_in_same_directory");
        // let file_name = file_name.to_str().unwrap();
        let mut file = File::create(working_directory.clone().join("find_file_in_same_directory"));

        // println!("Working directory: {:#?}", working_directory);
        // println!("file name: {:#?}", file_name);

        // Act
        search_working_directory(working_directory.as_path(), "find_file_in_same_directory");


        println!("{:#?}", tempfile::env::temp_dir());
        let tmp_dir = TempDir::new()?;

        let mut tmpfile: File = tempfile::tempfile().unwrap();
        println!("Metadata: {:#?}", tmpfile.metadata());
        println!("Name: {:#?}", tmpfile);
        write!(tmpfile, "Hello World!").unwrap();
        
        // Seek to start
        tmpfile.seek(SeekFrom::Start(0)).unwrap();

        // Read
        let mut buf = String::new();
        tmpfile.read_to_string(&mut buf).unwrap();
        assert_eq!("Hello World!", buf);

        drop(file);
        // test_dir.close();
        Ok(())
    }
}


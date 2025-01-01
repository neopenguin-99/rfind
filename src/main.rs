use std::io;
use std::fs;
use std::os::linux::net::SocketAddrExt;
use std::path::Path;
use std::process::exit;
use clap::Parser;
use std::env;
use clap::{Arg, ArgAction, ArgMatches, Command};

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
    //
    let a = fs::read_dir(working_directory).unwrap(); //todo fix unwrap
    for ele in a.into_iter() {
        let ele = ele.unwrap();
        let file_name = ele.file_name();
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


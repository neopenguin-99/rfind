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

    pub mod line;
    pub mod logger;
    pub mod symlinksetting;
    pub mod params;
    pub mod standardlogger;
    pub mod testlogger;
    pub mod test;
    pub mod filedescriptor;
    pub mod debugopts;
    pub mod message;
    pub mod searcher;
    pub mod threadpool;
    pub mod worker;
    pub mod fnbox;
    pub mod job;
    pub mod multithreadmessage;
}

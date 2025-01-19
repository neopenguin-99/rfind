pub use self::debugopts::DebugOpts;
pub mod debugopts {
    #[derive(Debug, PartialEq)]
    pub enum DebugOpts {
        Exec,
        Opt,
        Rates,
        Search,
        Stat,
        Tree,
        All,
        Help
    }
}

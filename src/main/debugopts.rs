pub use self::debugopts::DebugOpts;
pub mod debugopts {
    use crate::main::*;
    #[derive(Clone, Copy, Debug, PartialEq)]
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

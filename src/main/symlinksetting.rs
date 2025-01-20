pub use self::symlinksetting::SymLinkSetting;
pub mod symlinksetting {
    use crate::main::*;
    #[derive(Debug, Clone, PartialEq)]
    pub enum SymLinkSetting {
        Never,
        Follow,
        OnlyCommandLineArgs
    }
}

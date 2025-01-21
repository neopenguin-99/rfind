pub use self::params::Params;
pub mod params {
    use crate::main::*;
    use crate::main::symlinksetting::SymLinkSetting;
    use crate::main::debugopts::DebugOpts;
    #[derive(Debug)]
    pub struct Params {
        pub symlink_setting: SymLinkSetting,
        pub debug_opts: Option<DebugOpts>,
        pub optimisation_level: Option<u8> 
    }
}

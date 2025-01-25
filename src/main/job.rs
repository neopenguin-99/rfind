pub use self::job::Job;
pub mod job {
    pub use crate::main::fnbox::FnBox;
    pub type Job = Box<dyn FnBox + Send + 'static>;
}

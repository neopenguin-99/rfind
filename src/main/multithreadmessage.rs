pub use self::multithreadmessage::MultiThreadMessage;
pub mod multithreadmessage {
    use crate::main::job::Job;
    pub enum MultiThreadMessage {
        NewJob(Job),
        Terminate
    }
}

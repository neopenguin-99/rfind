pub use self::worker::Worker;
pub mod worker {
    use std::sync::mpsc;
    use std::thread;
    use crate::main::job::Job;
    use std::sync::{Arc, Mutex};
    pub struct Worker {
        id: usize,
        thread: thread::JoinHandle<()>,
    }

    impl Worker {
        pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
            let thread = thread::spawn(move || {
                loop {
                    let job = receiver.lock().unwrap().recv().unwrap();

                    println!("Worker {} got a job; executing.", id);

                    job.call_box();
                }
            });

            Worker {
                id,
                thread,
            }
        }
    }
}

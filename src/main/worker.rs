pub use self::worker::Worker;
pub mod worker {
    use std::sync::mpsc;
    use std::thread;
    use crate::main::multithreadmessage::MultiThreadMessage;
    use std::sync::{Arc, Mutex};
    #[derive(Debug)]
    pub struct Worker {
        pub id: usize,
        pub thread: Option<thread::JoinHandle<()>>
    }

    impl Worker {
        pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<MultiThreadMessage>>>) -> Worker {
            let thread = thread::spawn(move || {
                loop {
                    let message = receiver.lock().unwrap().recv().unwrap();

                    match message {
                        MultiThreadMessage::NewJob(job) => {
                            println!("Worker {} got a job; executing.", id);
                            job.call_box();
                        },
                        MultiThreadMessage::Terminate => {
                            println!("Worker {} was told to terminate", id);
                            break;
                        }
                    }
                }
            });

            Worker {
                id,
                thread: Some(thread)
            }
        }
    }
}

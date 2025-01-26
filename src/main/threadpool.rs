pub use self::threadpool::ThreadPool;
pub mod threadpool {
    use std::sync::mpsc;
    use crate::main::worker::Worker;
    use crate::main::job::Job;
    use std::sync::{Arc, Mutex};

    pub struct ThreadPool {
        workers: Vec<Worker>,
        sender: mpsc::Sender<Job>,
    }

    impl ThreadPool {
        pub fn new(size: usize) -> ThreadPool {
            assert!(size > 0);

            let (sender, receiver) = mpsc::channel();

            let receiver = Arc::new(Mutex::new(receiver));

            let mut workers = Vec::with_capacity(size);

            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)));
            }
            ThreadPool {
                workers,
                sender,
            }
        }
        pub fn execute<F>(&self, f: F)
            where
                F: FnOnce() + Send + 'static
        {
            let job = Box::new(f);

            self.sender.send(job).unwrap()
        }
    }
}

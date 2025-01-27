pub use self::threadpool::ThreadPool;
pub mod threadpool {
    use std::sync::mpsc;
    use crate::main::worker::Worker;
    use crate::main::multithreadmessage::MultiThreadMessage;
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    pub struct ThreadPool {
        workers: Vec<Worker>,
        sender: mpsc::Sender<MultiThreadMessage>,
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

            self.sender.send(MultiThreadMessage::NewJob(job)).unwrap()
        }
    }

    impl Drop for ThreadPool {
        fn drop(&mut self) {
            println!("Sending terminate message to all workers.");

            for _ in &mut self.workers {
                self.sender.send(MultiThreadMessage::Terminate).unwrap();
            }

            println!("Shutting down all workers.");

            for worker in &mut self.workers {
                println!("Shutting down worker {}", worker.id);

                if let Some(thread) = worker.thread.take() {
                    thread.join().unwrap();
                }
            }
        }
    }
}

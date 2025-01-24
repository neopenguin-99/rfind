pub use self::threadpool::ThreadPool;
pub mod threadpool {
    use std::thread;

    pub struct ThreadPool {
        threads: Vec<thread::JoinHandle<()>>
    }

    impl ThreadPool {
        fn new(size: usize) -> ThreadPool {
            assert!(size > 0);

            let mut threads = Vec::with_capacity(size);

            for _ in 0..size {

            }
            ThreadPool {
                threads
            }
        }
    }
}

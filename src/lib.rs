use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

// This is the same type the `execute` expects for its closure 🤓☝️
type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    // In our case, the closures we’re passing to the thread pool will handle the connection and not return anything, so <T> will be the unit type ().
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job; executing.");

            job();
        });

        Worker { id, thread }
    }
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            // For each new worker, we clone the Arc to bump the reference count so the workers can share ownership of the receiver
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    /// `execute` will send a job from the `ThreadPool` to the `Worker` instances
    /// said job will be sent through the `sender`.
    pub fn execute<F>(&self, f: F)
    where
        // * Why FnOnce ?
        // FnOnce is the trait we want to use because the thread for running a request will only execute that request’s closure one time
        // We still use the () after FnOnce because this FnOnce represents a closure that takes no parameters and returns the unit type ().
        // * Why Send ?
        // We need Send to transfer the closure from one thread to another
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

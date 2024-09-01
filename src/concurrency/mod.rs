use std::{
    sync::{mpsc, Arc, Mutex, Condvar},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
    job_counter: Arc<(Mutex<usize>, Condvar)>
}

type Job = Box<dyn FnOnce() + Send + 'static>;

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
        let job_counter = Arc::new((Mutex::new(0), Condvar::new()));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver), job_counter.clone()));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
            job_counter,
        }
    }

    pub fn join_all(&self){
        let mut counter = (*self.job_counter.0.lock().unwrap()).clone();
        if counter != 0 {
            counter = *self.job_counter.1.wait(self.job_counter.0.lock().unwrap()).unwrap();
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {

        let job = Box::new(f);

        let (lock, cvar) = &*self.job_counter;
        *lock.lock().unwrap() += 1;
 
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>, job_counter: Arc<(Mutex<usize>, Condvar)>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();

                    *job_counter.0.lock().unwrap() -= 1;

                    job_counter.1.notify_all();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

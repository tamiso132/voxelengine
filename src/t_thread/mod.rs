use lazy_static::lazy_static;
use std::{
    cell::UnsafeCell,
    collections::HashMap,
    sync::{mpsc, Arc, Condvar, Mutex},
    thread,
};

lazy_static! {
    pub static ref THREAD_POOL: Mutex<ThreadPool> = Mutex::new(ThreadPool::new(20));
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
    job_counter: Arc<(Mutex<usize>, Condvar)>,

    stages: StageHandles,

    join_handles: Vec<mpsc::Receiver<()>>,
    free_join_handles: Vec<usize>,
}

pub struct JoinHandle {
    reciever: mpsc::Receiver<()>,
}

pub struct StageHandles {
    recievers: HashMap<Stage, Arc<(Mutex<usize>, Condvar)>>,
}
#[derive(Hash, Debug, PartialEq, PartialOrd, Eq)]
pub struct Stage {
    id: u32,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
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
            stages: StageHandles { recievers: HashMap::new() },
            join_handles: vec![],
            free_join_handles: vec![],
        }
    }

    pub fn join_tasks() {
        let thread_pool = THREAD_POOL.lock().unwrap();
        let mut counter = (thread_pool.job_counter.0.lock().unwrap()).clone();
        if counter != 0 {
            for i in 0..counter {
                thread_pool.job_counter.1.wait(thread_pool.job_counter.0.lock().unwrap()).unwrap();
            }
        }
    }

    pub fn wait_on_stage<F>(stage: Stage) {
        let thread_pool = THREAD_POOL.lock().unwrap();

        let stage_val = thread_pool.stages.recievers.get(&stage).unwrap();

        let mut count = stage_val.0.lock().unwrap().clone();
        if count > 0 {
            for i in 0..count {
                *stage_val.1.wait(stage_val.0.lock().unwrap()).unwrap();
            }
        }
    }

    pub fn execute_stage<F>(f: F, stage: Stage)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut thread_pool = THREAD_POOL.lock().unwrap();

        let stage_value = thread_pool.stages.recievers.entry(stage).or_insert(Arc::new((Mutex::new(0 as usize), Condvar::new()))).clone();

        // increase number of tasks for this stage
        *stage_value.0.lock().unwrap() += 1;

        let mut job_counter = thread_pool.job_counter.clone();
        *job_counter.0.lock().unwrap() += 1;
        let job = Box::new(move || {
            f();
            *stage_value.0.lock().unwrap() -= 1;
            stage_value.1.notify_one();
        });

        // Send the task to the workers
        thread_pool.sender.as_ref().unwrap().send(job).unwrap();
    }

    pub fn execute<F>(f: F) -> usize
    where
        F: FnOnce() + Send + 'static,
    {
        let mut thread_pool = THREAD_POOL.lock().unwrap();

        let (done_sender, reciever) = mpsc::channel();

        // increase amount of jobs by 1
        let mut job_counter = thread_pool.job_counter.clone();
        *job_counter.0.lock().unwrap() += 1;

        let job = Box::new(move || {
            f();
            done_sender.send(()).unwrap();
            job_counter.1.notify_one();
        });

        thread_pool.sender.as_ref().unwrap().send(job).unwrap();

        let index;
        if thread_pool.free_join_handles.len() > 0 {
            index = thread_pool.free_join_handles.pop().unwrap();
            thread_pool.join_handles[index] = reciever;
        } else {
            thread_pool.join_handles.push(reciever);
            index = thread_pool.join_handles.len() - 1;
        }
        index
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
                    job();
                    *job_counter.0.lock().unwrap() -= 1;
                    job_counter.1.notify_one();
                }
                Err(_) => {
                    break;
                }
            }
        });

        Worker { id, thread: Some(thread) }
    }
}

pub struct Ptr<T> {
    pub data: *const T,
}

impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        Self { data: self.data }
    }
}

impl<T> Ptr<T> {
    pub fn new(data: *const T) -> Self {
        Self { data }
    }
}

pub struct MutPtr<T> {
    pub data: *mut T,
}

impl<T> Clone for MutPtr<T> {
    fn clone(&self) -> Self {
        Self { data: self.data }
    }
}

impl<T> MutPtr<T> {
    pub fn new(data: *mut T) -> Self {
        Self { data }
    }
}

unsafe impl<T> Send for MutPtr<T> {}
unsafe impl<T> Send for Ptr<T> {}

// This thread module is based on the rust book

use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Job>>>,
        num_jobs: Arc<Mutex<usize>>,
    ) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let job = receiver.lock().unwrap().recv().unwrap();
                println!("Worker {} received a job", id);
                job();
                let mut numb_jobs = num_jobs.lock().unwrap();
                *numb_jobs -= 1;
            }
        });
        Self { id, thread }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
    num_jobs: Arc<Mutex<usize>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let num_jobs = Arc::new(Mutex::new(0));

        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
                Arc::clone(&num_jobs),
            ));
        }
        Self {
            workers,
            sender,
            num_jobs,
        }
    }

    pub fn create_with_max_threads() -> Self {
        let size = thread::available_parallelism().map_or(1, |n| n.get());
        Self::new(size)
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        let mut num_jobs = self.num_jobs.lock().unwrap();
        *num_jobs += 1;
        self.sender.send(job).unwrap();
    }

    pub fn block_until_finished(&self) {
        loop {
            let num_jobs = *self.num_jobs.lock().unwrap();
            if num_jobs == 0 {
                break;
            }
            // thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

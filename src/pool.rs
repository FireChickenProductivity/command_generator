// This thread module is based on the rust book

use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

type Job<JobResult> = Box<dyn FnOnce() -> JobResult + Send + 'static>;

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new<JobResult>(
        id: usize,
        receiver: Arc<Mutex<mpsc::Receiver<Job<JobResult>>>>,
        result_sender: mpsc::Sender<JobResult>,
    ) -> Self
    where
        JobResult: Send + 'static,
    {
        let thread = thread::spawn(move || {
            loop {
                let job = receiver.lock().unwrap().recv().unwrap();
                let result: JobResult = job();
                result_sender.send(result).unwrap();
            }
        });
        Self { id, thread }
    }
}

pub struct ThreadPool<JobResult> {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job<JobResult>>,
    receiver: mpsc::Receiver<JobResult>,
}

impl<JobResult: Send + 'static> ThreadPool<JobResult> {
    pub fn new(size: usize) -> Self {
        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let (result_sender, result_receiver) = mpsc::channel();

        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
                result_sender.clone(),
            ));
        }
        Self {
            workers,
            sender,
            receiver: result_receiver,
        }
    }

    pub fn create_with_max_threads() -> Self {
        let size = thread::available_parallelism().map_or(1, |n| n.get());
        Self::new(size)
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() -> JobResult + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }

    pub fn join(&self) -> Vec<JobResult> {
        // I need a mechanism for knowing when all jobs are done
        let mut results = Vec::new();
        while let Ok(value) = self.receiver.recv() {
            results.push(value);
        }
        results
    }
}

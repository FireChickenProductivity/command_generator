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
        receiver: Arc<Mutex<mpsc::Receiver<(usize, Job<JobResult>)>>>,
        result_sender: mpsc::Sender<(usize, JobResult)>,
    ) -> Self
    where
        JobResult: Send + 'static,
    {
        let thread = thread::spawn(move || {
            loop {
                let (number, job) = receiver.lock().unwrap().recv().unwrap();
                let result: JobResult = job();
                result_sender.send((number, result)).unwrap();
            }
        });
        Self { id, thread }
    }
}

pub struct ThreadPool<JobResult> {
    workers: Vec<Worker>,
    sender: mpsc::Sender<(usize, Job<JobResult>)>,
    receiver: mpsc::Receiver<(usize, JobResult)>,
    job_number: usize,
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
            job_number: 0,
        }
    }

    pub fn create_with_max_threads() -> Self {
        let size = thread::available_parallelism().map_or(1, |n| n.get());
        Self::new(size)
    }

    pub fn execute<F>(&mut self, f: F)
    where
        F: FnOnce() -> JobResult + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send((self.job_number, job)).unwrap();
        self.job_number += 1;
    }

    pub fn join(&mut self) -> Vec<JobResult> {
        let mut results: Vec<Option<JobResult>> = Vec::with_capacity(self.job_number);
        results.resize_with(self.job_number, || None);
        let mut received = 0;
        while let Ok((number, value)) = self.receiver.recv() {
            results[number] = Some(value);
            received += 1;
            if received == self.job_number {
                break;
            }
        }
        self.job_number = 0;
        results.into_iter().map(|r| r.unwrap()).collect()
    }
}

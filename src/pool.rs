use std::thread;

struct Worker {
	id: usize,
	thread: thread::JoinHandle<()>,
}

impl Worker {
	fn new(id: usize) -> Self {
		let thread = thread::spawn(move || {
			
		});
		Self {
			id,
			thread,
		}
	}
}

pub struct ThreadPool {
	workers: Vec<Worker>,
}

impl ThreadPool {
	pub fn new(size: usize) -> Self {
		let mut workers = Vec::with_capacity(size);
		for id in 0..size {
			workers.push(Worker::new(id));
		}
		Self {
			workers
		}
	}

	pub fn execute<F>(&self, _f: F) where F: FnOnce() + Send + 'static {
		// Placeholder for thread pool execution logic
	}
}

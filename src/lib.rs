pub struct ThreadPool {
    workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id));
        }

        ThreadPool { workers }
    }
}

pub struct Worker {
    pub id: usize,
    pub thread: std::thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize) -> Worker {
        Self {
            id,
            thread: std::thread::spawn(|| {}),
        }
    }
}

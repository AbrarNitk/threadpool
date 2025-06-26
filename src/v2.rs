use std::thread::JoinHandle;

pub type Job = Box<dyn Fn() + Send + 'static>;

pub enum Message {
    Job(Job),
    Exit,
}

pub struct ThreadPool {
    pub workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn new(count: usize) -> Self {
        let mut workers = vec![];
        for id in 0..count {
            let worker = Worker::new(id + 1);
            workers.push(worker);
        }

        Self { workers }
    }
}

pub struct Worker {
    pub id: String,
    pub thread: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize) -> Self {
        let thread = std::thread::spawn(|| {});

        Worker {
            id: format!("worker-{}", id),
            thread,
        }
    }
}

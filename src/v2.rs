use crossbeam::queue::SegQueue;
use std::sync::Arc;
use std::thread::JoinHandle;

pub type Job = Box<dyn Fn() + Send + 'static>;

pub enum Message {
    Job(Job),
    Exit,
}

pub struct ThreadPool {
    /// Worker Thread
    pub workers: Vec<Worker>,

    /// Job queue
    pub queue: Arc<SegQueue<Message>>,
}

impl ThreadPool {
    pub fn new(count: usize) -> Self {
        let queue = Arc::new(SegQueue::new());

        let q = std::sync::Arc::clone(&queue);

        let mut workers = vec![];
        for id in 0..count {
            let worker = Worker::new(id + 1, std::sync::Arc::clone(&queue));
            workers.push(worker);
        }

        Self { workers, queue }
    }
}

pub struct Worker {
    pub id: String,
    pub thread: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, queue: Arc<SegQueue<Message>>) -> Self {
        let thread = std::thread::spawn(move || loop {
            match queue.pop() {
                Some(Message::Exit) => break,
                Some(Message::Job(job)) => {
                    job();
                }
                None => {}
            }
        });

        Worker {
            id: format!("worker-{}", id),
            thread,
        }
    }
}

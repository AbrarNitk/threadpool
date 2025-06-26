use crossbeam::queue::SegQueue;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

pub type Job = Box<dyn FnOnce() + Send + 'static>;

pub enum Message {
    Job(Job),
    Exit,
}

pub struct ThreadPool {
    /// Worker Thread
    pub workers: Vec<Worker>,

    /// Job queue
    pub queue: Arc<SegQueue<Message>>,

    /// Job Available
    pub job_signal: Arc<(Mutex<bool>, Condvar)>,
}

impl ThreadPool {
    pub fn new(count: usize) -> Self {
        let queue = Arc::new(SegQueue::new());
        let job_signal = Arc::new((Mutex::new(false), Condvar::new()));
        let mut workers = vec![];
        for id in 0..count {
            let worker = Worker::new(
                id + 1,
                std::sync::Arc::clone(&queue),
                Arc::clone(&job_signal),
            );
            workers.push(worker);
        }

        Self {
            workers,
            queue,
            job_signal,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let message = Message::Job(Box::new(f));
        self.queue.push(message);

        // mark job is available
        let (lock, condvar) = &*self.job_signal;
        let mut job_available = lock.lock().unwrap();
        *job_available = true;
        condvar.notify_all();
    }
}

pub struct Worker {
    pub id: String,
    pub thread: JoinHandle<()>,
}

impl Worker {
    fn new(
        id: usize,
        queue: Arc<SegQueue<Message>>,
        job_signal: Arc<(Mutex<bool>, Condvar)>,
    ) -> Self {
        let thread = std::thread::spawn(move || loop {
            println!("running");

            match queue.pop() {
                Some(Message::Exit) => break,
                Some(Message::Job(job)) => {
                    job();
                }
                None => {
                    let (signal, condvar) = &*job_signal;

                    let mut job_available = signal.lock().unwrap();

                    // need while loop because cpu can wake the thread without even
                    // condvar waking it up, that's why it needs the loop and not if
                    while !*job_available {
                        job_available = condvar.wait(job_available).unwrap();
                    }

                    *job_available = false;

                    // Note: here it must sleep till we don't have any jobs in the queue
                    // once it has jobs in the queue, queue must be wake-up again
                    // and starts the execution
                }
            }
        });

        Worker {
            id: format!("worker-{}", id),
            thread,
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn t() {
        let pool = ThreadPool::new(5);

        pool.execute(|| println!("hello1"));
        std::thread::sleep(std::time::Duration::from_secs(1));

        pool.execute(|| println!("hello2"));
        std::thread::sleep(std::time::Duration::from_secs(1));

        pool.execute(|| println!("hello3"));
        std::thread::sleep(std::time::Duration::from_secs(1));

        pool.execute(|| println!("hello4"));

        std::thread::sleep(std::time::Duration::from_secs(100));
        // if workers have some jobs it should not get shutdown
    }
}

use std::sync::{mpsc, Arc, Mutex};
// use std::sync::mpmc::RecvError;

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Message>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Message::NewJob(Box::new(f));
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // self.sender.send(Message::Terminate).unwrap();

        for _ in &self.workers {
            let _ = self.sender.as_ref().unwrap().send(Message::Terminate);
        }

        drop(self.sender.take());
    }
}

pub struct Worker {
    id: usize,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, rc: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let handle = std::thread::spawn(move || loop {
            let job = rc.lock().unwrap().recv();
            match job {
                Ok(Message::NewJob(job)) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                // Ok(Message::Terminate) => break,
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
                _ => {
                    println!("Worker {} got a message that wasn't a job; ignoring.", id);
                }
            }
        });

        Self {
            id,
            thread: Some(handle),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(th) = self.thread.take() {
            // println!("Shutting down worker {}", self.id);
            th.join().unwrap();
            println!("Worker {} shut down.", self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn if_works() {
        let pool = ThreadPool::new(4);
        pool.execute(|| {
            std::thread::sleep(std::time::Duration::from_secs(1));
            println!("hello threadpool t1")
        });
        pool.execute(|| println!("hello threadpool t2"));
        pool.execute(|| {
            std::thread::sleep(std::time::Duration::from_secs(1));
            println!("hello threadpool t3")
        });
        pool.execute(|| println!("hello threadpool t4"));
    }

    #[test]
    fn with_time() {
        let pool = ThreadPool::new(2);
        pool.execute(|| {
            println!("hello threadpool t1");
            std::thread::sleep(std::time::Duration::from_secs(3));
            println!("done: hello threadpool t1");
        });
        pool.execute(|| {
            println!("hello threadpool t2");
            std::thread::sleep(std::time::Duration::from_secs(3));
            println!("done: hello threadpool t2");
        });
        pool.execute(|| {
            println!("hello threadpool t3");
            std::thread::sleep(std::time::Duration::from_secs(3));
            println!("done: hello threadpool t3");
        });
        pool.execute(|| {
            println!("hello threadpool t4");
            std::thread::sleep(std::time::Duration::from_secs(3));
            println!("done: hello threadpool t4");
        });

        pool.execute(|| {
            println!("hello threadpool t5");
            std::thread::sleep(std::time::Duration::from_secs(3));
            println!("done: hello threadpool t5");
        });
    }
}

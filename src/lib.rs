use std::sync::{Arc, Mutex, mpsc};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

pub struct Worker {
    id: usize,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, rc: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let handle = std::thread::spawn(move || {
            loop {
                let job = rc.lock().unwrap().recv().unwrap();
                println!("Worker {} got a job; executing.", id);
                job();
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
        println!("Shutting down worker {}", self.id);
        self.thread.take().expect("").join().unwrap();
        println!("Worker {} shut down.", self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn if_works() {
        let pool = ThreadPool::new(4);
        pool.execute(|| println!("hello threadpool t1"));
        pool.execute(|| println!("hello threadpool t2"));
        pool.execute(|| println!("hello threadpool t3"));
        pool.execute(|| println!("hello threadpool t4"));
    }


    #[test]
    fn if_works_2() {
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

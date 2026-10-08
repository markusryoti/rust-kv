use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, SendError, Sender},
    },
    thread,
};

#[derive(Debug)]
pub struct ThreadPool {
    tx: Sender<Job>,
    workers: Vec<Worker>,
}

#[derive(Debug, Clone)]
struct Worker {
    rx: Arc<Mutex<Receiver<Job>>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn run(self) {
        thread::spawn(move || {
            loop {
                let job = {
                    let rx = self.rx.lock().expect("to work");

                    match rx.recv() {
                        Ok(job) => job,
                        Err(_) => return,
                    }
                };

                job();
            }
        });
    }
}

impl ThreadPool {
    pub fn new(num_workers: usize) -> Self {
        let (tx, rx) = mpsc::channel::<Job>();
        let rx = Arc::new(Mutex::new(rx));

        let mut workers = vec![];
        for _ in 0..num_workers {
            workers.push(Worker { rx: rx.clone() });
        }

        ThreadPool { tx, workers }
    }

    pub fn start(&self) {
        self.workers.iter().for_each(|w| w.clone().run());
    }

    pub fn enqueue(&self, job: Job) -> Result<(), SendError<Job>> {
        self.tx.send(job)?;
        Ok(())
    }
}

impl Default for ThreadPool {
    fn default() -> Self {
        Self::new(1)
    }
}

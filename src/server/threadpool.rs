use std::sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}};
use std::thread::{self, JoinHandle};

type Job = Box<dyn FnOnce(usize) + Send + 'static>;

pub struct ThreadPool{
  workers: Vec<Worker>,
  tx: Option<Sender<Job>>,
}

impl ThreadPool {

  // Initialises with number of threads to create
  pub fn new(ports: Vec<usize>) -> ThreadPool {

    let (tx, rx) = mpsc::channel();

    // Only one instance of rx, will be shared between threads
    let rx :Arc<Mutex<Receiver<Job>>> = Arc::new(Mutex::new(rx));

    // Initialise worker threads
    let mut workers: Vec<Worker> = Vec::with_capacity(ports.len());
    for port in ports{
      println!("Creating worker for port {}...", port);
      workers.push(Worker::new(port, rx.clone()));
    }

    ThreadPool{
      workers,
      tx: Some(tx),
    }
  }
   
  // Adds given closure to queue, available workers will execute it
  pub fn execute<F>(&self, f: F) 
  where 
    F: FnOnce(usize) -> () + Send + 'static
  {
    let job: Job = Box::new(f);
    self.tx.as_ref().unwrap().send(job).unwrap();
  }

}

impl Drop for ThreadPool{
  fn drop(&mut self){

    drop(self.tx.take());

    for worker in &mut self.workers{
      println!("Shutting down worker with port {}...", worker.port);
      
      if let Some(handle) = worker.handle.take(){
        handle.join().unwrap();
      }
    }
  }
}

struct Worker{
  port: usize,
  handle: Option<JoinHandle<()>>
}

impl Worker{
  fn new(port: usize, rx: Arc<Mutex<Receiver<Job>>>) -> Worker {

    // Loop waiting for task till Terminate signal comes in
    let handle: JoinHandle<()> = thread::spawn(move || loop {
        let job_result = rx.lock().unwrap().recv();

        match job_result{
          Ok(job) => {
            println!("Worker for port {} has received instructions.", port);
            job(port);
          }
          Err(_) => {
            println!("Worker {} is terminating. Initiating self-destruct sequence...", port);
            break;
          }
        }
      });

    Worker {
      port, 
      handle: Some(handle),
    }
  }
}
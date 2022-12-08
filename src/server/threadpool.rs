use std::sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}};
use std::thread::{self, JoinHandle};

trait FnBox {
  fn call_box(self: Box<Self>, port: usize);
}

impl<F: FnOnce()> FnBox for F {
  fn call_box(self: Box<F>, port: usize) {
    (*self)(port)
  }
}

type Job = Box<dyn FnBox + Send + 'static>;

enum Instruction{
  Task(Job),
  Terminate,
}

pub struct ThreadPool{
  workers: Vec<Worker>,
  tx: Sender<Instruction>
}

impl ThreadPool {

  // Initialises with number of threads to create
  pub fn new(ports: Vec<usize>) -> ThreadPool {

    let (tx, rx) = mpsc::channel();

    // Only one instance of rx, will be shared between threads
    let rx :Arc<Mutex<Receiver<Instruction>>> = Arc::new(Mutex::new(rx));

    // Initialise worker threads
    let mut workers: Vec<Worker> = Vec::with_capacity(ports.len());
    for port in ports{
      println!("Creating worker for port {}...", port);
      workers.push(Worker::new(port, rx.clone()));
    }

    ThreadPool{
      workers,
      tx,
    }
  }
   
  // Adds given closure to queue, available workers will execute it
  pub fn execute<F>(&self, f: F) 
  where 
    F: FnOnce() + Send + 'static
  {
    let job = Box::new(f);
    self.tx.send(Instruction::Task(job)).unwrap();
  }

}

impl Drop for ThreadPool{
  fn drop(&mut self){

    // Send one terminate for each existing worker
    for _ in &mut self.workers {
      self.tx.send(Instruction::Terminate).unwrap();
    }

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
  fn new(port: usize, rx: Arc<Mutex<mpsc::Receiver<Instruction>>>) -> Worker {

    let handle: JoinHandle<()> = thread::spawn(move || {
      // Loop waiting for task till Terminate signal comes in
      loop{
        let instruction: Instruction = rx.lock().unwrap().recv().unwrap();

        match instruction{
          Instruction::Task(job) => {
            println!("Worker for port {} has received instructions.", port);
            job.call_box(port);
          }
          Instruction::Terminate => {
            println!("Worker {} is terminating. Initiating self-destruct sequence...", port);
            break;
          }
        }
      }
    });

    Worker {
      port, 
      handle: Some(handle),
    }
  }
}
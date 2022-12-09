extern crate parfs;
use parfs::server::threadpool::ThreadPool;

fn main() {
  let ports: Vec<usize> = Vec::from([1,2,3,4]);
  let pool = ThreadPool::new(ports);

  for run in 0..5{
    pool.execute(|port: usize| {print_port(port);});
  }

}

fn print_port(port: usize){
  println!("Port number is {}", port);
}
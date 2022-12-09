extern crate parfs;
use std::{net::TcpListener, process::exit};
use std::io::Error;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use parfs::server::fsrw_mutex::FsrwMutex;
use parfs::server::handler::ConnectionHandler;
use parfs::server::threadpool::ThreadPool;
use parfs::server::utilities::MAX_REQUEST_NO;

fn main() {
  let args: Vec<String> =env::args().collect();
  let addr_to_listen: &str = &args[1];
  let home_folder: PathBuf = PathBuf::from(&args[2]);

  let first_free_port: usize = 12801;

  let listener_result: Result<TcpListener, Error> = TcpListener::bind(addr_to_listen);

  // Prints error if unable to listen on address
  if listener_result.is_err() {
    let e: Error = listener_result.err().unwrap();
    print!("Error listening on address {}", addr_to_listen);
    print!("{}", e.to_string());
    exit(1);
  }
  
  // Initialize file system reader writer mutex
  let fsrw_mutex = Arc::new(FsrwMutex::new());

  // Creates a threadpool
  let ports: Vec<usize> = (first_free_port..(first_free_port + MAX_REQUEST_NO)).collect();
  let threadpool = ThreadPool::new(ports);

  // Listens for incoming connection requests
  let listener: TcpListener = listener_result.unwrap();
  for stream in listener.incoming() {
    let stream = stream.unwrap();

    // Create handler for incoming stream
    let handle = 
      ConnectionHandler::new(
        stream, 
        home_folder.clone(), 
        fsrw_mutex.clone(), 
        addr_to_listen.to_string()
      ).unwrap();

    // Pass handler off to threadpool to initialise new ports and handle requests
    threadpool.execute(|port: usize| {handle.handle_connection(port);});
  }
}

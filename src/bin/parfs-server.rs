extern crate parfs;
use std::net::TcpStream;
use std::{net::TcpListener, process::exit};
use std::io::{Error, Write};
use std::env;
use std::path::PathBuf;
use parfs::server::handler::ConnectionHandler;

fn main() {
  let args: Vec<String> =env::args().collect();
  let home_folder: PathBuf = PathBuf::from(&args[0]);
  let addr_to_listen: &str = "127.0.0.1:12800";
  let mut free_port: i32 = 12801;
  let listener_result: Result<TcpListener, Error> = TcpListener::bind(addr_to_listen);

  // Prints error if unable to listen on address
  if listener_result.is_err() {
    let e: Error = listener_result.err().unwrap();
    print!("Error listening on address {}", addr_to_listen);
    print!("{}", e.to_string());
    exit(1);
  }
  
  let listener: TcpListener = listener_result.unwrap();
  
  for stream in listener.incoming() {
    let stream = stream.unwrap();
      ConnectionHandler::new(stream).unwrap();
  }
  

  // for stream in listener.incoming() {
  //   // Prints error if problem with incoming stream
  //   if stream.is_err() {
  //     let e: Error = stream.err().unwrap();
  //     print!("{}", e.to_string());
  //     continue;
  //   }

  //   let mut tcp_stream: TcpStream = stream.unwrap();
    
  //   // Send next available free port to client to establish connection
  //   let bytes: [u8; 4] = free_port.to_le_bytes();
  //   free_port += 1;

  //   let write_result: Result<usize, Error> = tcp_stream.write(&bytes);
  //   if let Err(e) = write_result{
  //     println!("{}", e);
  //   }

  // }
}

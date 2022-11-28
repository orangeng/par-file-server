use std::net::TcpStream;
use std::{net::TcpListener, process::exit};
use std::io::{Error, Write};

fn main() {
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
    // Prints error if problem with incoming stream
    if stream.is_err() {
      let e: Error = stream.err().unwrap();
      print!("{}", e.to_string());
      continue;
    }

    let mut tcp_stream: TcpStream = stream.unwrap();
    
    // Send next available free port to client to establish connection
    let bytes: [u8; 4] = free_port.to_le_bytes();
    free_port += 1;

    let write_result: Result<usize, Error> = tcp_stream.write(&bytes);
    if let Err(e) = write_result{
      println!("{}", e);
    }

  }
}
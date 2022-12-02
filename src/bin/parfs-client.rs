extern crate parfs;
<<<<<<< HEAD
use std::{io::{self, Write, Error, Read}, process::exit, net::TcpStream};
use parfs::client::connection::Connection;
=======
use parfs::client::connection::*;
use std::env;
use std::{io::{self, Write, Error, Read}, process::exit, net::TcpStream};
use regex::Regex;

>>>>>>> 79bf828 (Updated messagereceiver to use a reference to tcpstream instead)

fn main() {
  let args: Vec<String> =env::args().collect();
  let ip_address  = args[0].clone().to_string()+":12800";
  // Indicator for user requested quit
  let mut quit: bool = false;

  // Connection that will be used by this client program
  let mut conn: Connection = Connection{
    connected: false,
    stream: None,
    addr: "".to_string()
  };
  
  while !quit {
    // Print out prompt
    print!("parfs-client>");
    let flush = io::stdout().flush();
    if let Err(err) = flush {
      eprintln!("{}", err);
    }

    // Read in line from stdin
    let mut input = String::new();
    let read = io::stdin().read_line(&mut input);
    if read.is_err() {
      println!("There was an error reading the command. Please try again...");
      continue;
    }
    
    // No commands issued
    let tokens: Vec<&str> = input.split_ascii_whitespace().collect();
    if tokens.len() == 0{
      println!("No commands were issued! Type 'help' for a list of commands.");
      continue;
    }

    // User asks to quit
    if tokens[0] == "quit"{
      quit = true;
      // Call some cleanup function
      continue;
    }

    // Process the command
    let result: Result<(), String> = conn.process_command(&tokens);
    if let Err(err) = result {
      println!("{}", err);
      continue;
    }

  }

  exit(0);

<<<<<<< HEAD
}
=======
}

pub fn process_command(conn: &mut Connection, tokens: &Vec<&str>) -> Result<(), String>{

  // uses the conn class to process commands once connected. im not a fan of this.
  if conn.stream.is_some() {
    conn.process_command(tokens);
  }
  // "connect" command
  if tokens[0] == "connect"{
    // disabled
    // let stream_result: TcpStream = connect(&tokens)?;
    // conn.stream = Some(stream_result);
    // conn.addr = tokens[1].to_string();
    let mut stream = TcpStream::connect(&conn.addr).unwrap();
    conn.stream = Some(stream);
    return Ok(());
  }
  
  let invalid: String = String::from("Command was invalid. Type 'help' for a list of commands.");
  return Err(invalid);
}


fn connect(tokens: &Vec<&str>) -> Result<TcpStream, String>{
  
  // Some return strings
  let help: String = "Help:\n\tconnect [socket-addr]\n\t[socket-addr]: 'ip-addr:port' e.g. 127.0.0.1:12800".to_string();
  let wrong_addr: String = "There was an error connecting to the given socket address.".to_string();

  // Insufficient / wrong no of arguments
  if tokens.len() != 2{
    return Err(help);
  }
  
  // Validate socket address
  let addr: &str = tokens[1];
  let socket_addr_re = Regex::new(r"^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d{1,5}$").unwrap();
  let valid: bool = socket_addr_re.is_match(addr);
  if !valid{
    return Err("Socket address is invalid.\n".to_string() + &help);
  }

  let addr_split: Vec<&str> = addr.split(":").collect();
  let ip_addr = addr_split[0];

  let stream_result: Result<TcpStream, Error> = TcpStream::connect(tokens[1]);
  if stream_result.is_err(){
    return Err(wrong_addr);
  }
  
  // If connection opened successfully
  let mut stream: TcpStream = stream_result.unwrap();
  let mut buf: [u8; 4]= [0; 4];
  let port_read_result = stream.read(&mut buf);
  if port_read_result.is_err(){
    return Err(wrong_addr);
  }

  let new_port: i32 = i32::from_le_bytes(buf);
  let new_addr: &str = &(ip_addr.to_string() + ":" + new_port.to_string().as_str());
  println!("New address to connect to: {}", new_addr);
  
  return Ok(stream);
}

>>>>>>> 79bf828 (Updated messagereceiver to use a reference to tcpstream instead)

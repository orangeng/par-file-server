use std::net::TcpStream;
use std::io::Error;
use std::io::Read;
use regex::Regex;
use crate::client::utilities::Command;

pub struct Connection {
  pub stream: Option<TcpStream>,
  pub addr: String,
}

impl Connection {
  
  // This function returns a Result. The Err(String) contains the string that will be
  // printed on the user interface
  pub fn process_command(&mut self, tokens: &Vec<&str>) -> Result<(), String>{
    
    // Check if command is valid
    let invalid: String = "Command was invalid. Type 'help' for a list of commands.".to_string();
    let command_type: Command = match tokens[0] {
      "connect" => Command::Connect,
      "help" => Command::Help,
      "login" => Command::Login,
      "mkdir" => Command::Mkdir,
      "cd" => Command::Cd,
      "ls" => Command::Ls,
      "up" => Command::Up,
      "down" => Command::Down,
      "status" => Command::Status,
      _ => {return Err(invalid)}
    };

    // "connect" command
    if let Command::Connect = command_type {
      let stream_result: TcpStream = self.connect(tokens)?;
      self.stream = Some(stream_result);
      self.addr = tokens[1].to_string();
      return Ok(());
    }
    
    // "help" command
    if let Command::Help = command_type {
      self.help();
      return Ok(());
    }
    // "status" command
    else if let Command::Status = command_type{
      self.status();
      return Ok(());
    }

    // Check if stream is established
    if self.stream.is_none(){
      return Err("Connection has not been established yet. Type 'help' for a list of commands.".to_string());
    }

    return Ok(());

  }

  fn connect(&mut self, tokens: &Vec<&str>) -> Result<TcpStream, String>{
  
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

  fn help(&self){
    for command in Command::iterator() {
      let command_str: String = command.get_str();
      println!("{:?}{} {}", command_str, " ".repeat(20 - command_str.len()),command.get_desc());
    }
  }

  fn status(&self){
    if self.stream.is_none() {
      println!("Connection has not been established. Use \"connect\" to connect to a parfs server.");
      return;
    }
    println!("Connected to server at {}.", self.addr);
  }

}

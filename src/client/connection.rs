use std::io::Error;
use regex::Regex;
use crate::client::utilities::*;
use std::{
  io::{BufReader, BufWriter},
  net::TcpStream,
};

use crate::message::*;

pub struct Connection {
  pub stream: Option<TcpStream>,
  pub addr: String,
  pub cwd: String,
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

    /******************** COMMANDS FROM HERE ONWARDS REQUIRE A WORKING TCPSTREAM ********************/

    // Check if stream is established
    if self.stream.is_none(){
      return Err(ERR_NO_STREAM.to_string());
    }

    // "cd" command
    if let Command::Cd = command_type{
      let ok: () = self.cd(&tokens)?;
      if ok == () {
        self.cwd = tokens[1].to_string();
      }
    }

    else if let Command::Ls = command_type{
      self.ls(&tokens)?;
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
  
    //let addr_split: Vec<&str> = addr.split(":").collect();
    //let ip_addr = addr_split[0];
  
    let stream_result: Result<TcpStream, Error> = TcpStream::connect(tokens[1]);
    if stream_result.is_err(){
      return Err(wrong_addr);
    }
    
    // If connection opened successfully
    let stream: TcpStream = stream_result.unwrap();

    // Code commented out, is for use when server demands a connection on a new port
    /* let mut buf: [u8; 4]= [0; 4];
    let port_read_result = stream.read(&mut buf);
    if port_read_result.is_err(){
      return Err(wrong_addr);
    }
  
    let new_port: i32 = i32::from_le_bytes(buf);
    let new_addr: &str = &(ip_addr.to_string() + ":" + new_port.to_string().as_str());
    println!("New address to connect to: {}", new_addr); */
    
    // Receives welcome message from server
    let tcp_reader: BufReader<&TcpStream> = BufReader::new(&stream);
    let confirmation_message: MessageReceiver = match MessageReceiver::new(tcp_reader) {
      Ok(server_message) => server_message,
      Err(e) => {
        return Err(e.to_string());
      }
    };

    // Check welcome message from server and print it out
    match confirmation_message.command {
      MessageKind::Success => {
        println!("{}", confirmation_message.arguments);
        return Ok(stream);
      },
      _ => {
        return Err(wrong_addr);
      }
    };
  }

  fn cd(&mut self, tokens: &Vec<&str>) -> Result<(), String> {
    let help: String = "Help:\n\tcd [file path]".to_string();
    
    // currently only supports non-spaced file paths
    // TODO: support quotation file paths
    if tokens.len() != 2 {
      return Err(help);
    }
    let tcp_stream: &TcpStream = match &self.stream {
      Some(tcp) => &tcp,
      None => {
        return Err(ERR_NO_STREAM.to_string());
      }
    };

    // Sends cd request
    let message_sender: MessageSender = MessageSender::new(
        MessageKind::Cd,
        tokens[1].to_string(),
        None,
    );
    let tcp_writer: BufWriter<&TcpStream> = BufWriter::new(&tcp_stream);
    let ms_result: Result<(), Error> = message_sender.send_message(tcp_writer);
    if ms_result.is_err(){
      return Err(ERR_NON_SERVER.to_string());
    }

    // Read in request output from server
    let tcp_reader: BufReader<&TcpStream> = BufReader::new(tcp_stream);
    let confirmation_message: MessageReceiver = match MessageReceiver::new(tcp_reader) {
      Ok(server_message) => server_message,
      Err(_) => {
        return Err(ERR_NON_SERVER.to_string());
      }
    };

    match  confirmation_message.command {
      MessageKind::Success => {
        return Ok(());
      },
      MessageKind::Error => {
        println!("{}", &confirmation_message.arguments);
        return Ok(());
      },
      _ => Err(ERR_SERVER.to_string()),
    }
  }

  fn ls(&self, tokens: &Vec<&str>) -> Result<(), String> {
    // Some return strings
    let help: String = "Help:\n\tls".to_string();
  
    // Insufficient / wrong no of arguments
    if tokens.len() != 1{
      return Err(help);
    }
    
    // Borrow the TcpStream
    let tcp_stream: &TcpStream = match &self.stream {
      Some(tcp) => &tcp,
      None => {
        return Err(ERR_NO_STREAM.to_string());
      }
    };

    // Sends ls request
    let message_sender: MessageSender = MessageSender::new(
        MessageKind::Ls,
        "".to_string(),
        None,
      );
    let tcp_writer: BufWriter<&TcpStream> = BufWriter::new(&tcp_stream);
    let ms_result: Result<(), Error> = message_sender.send_message(tcp_writer);
    if ms_result.is_err(){
      return Err(ERR_NON_SERVER.to_string());
    }

    // Read in request output from server
    let tcp_reader: BufReader<&TcpStream> = BufReader::new(tcp_stream);
    let confirmation_message: MessageReceiver = match MessageReceiver::new(tcp_reader) {
      Ok(server_message) => server_message,
      Err(_) => {
        return Err(ERR_NON_SERVER.to_string());
      }
    };

    match  confirmation_message.command {
      MessageKind::Success => {
        println!("{}", confirmation_message.arguments);
        return Ok(());
      },
      MessageKind::Error => {
        println!("{}", &confirmation_message.arguments);
        return Ok(());
      },
      _ => Err(ERR_SERVER.to_string()),
    }

  }

  fn help(&self){
    for command in Command::iterator() {
      let command_str: String = command.get_str();
      println!("{:?}{} {}", command_str, " ".repeat(20 - command_str.len()),command.get_desc());
    }
  }

  fn status(&self){
    if self.stream.is_none() {
      println!("{}", ERR_NO_STREAM);
      return;
    }
    println!("Connected to server at {}.", self.addr);
    println!("Current working directory is '{}'", self.cwd);
  }

}

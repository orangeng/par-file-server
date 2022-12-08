extern crate parfs;
use std::{io::{self, Write}, process::exit, env};
use parfs::{client::connection::Connection, errors::ClientError};

fn main() {
  let args: Vec<String> = env::args().collect();
  // Indicator for user requested quit
  let mut quit: bool = false;

  // Connection that will be used by this client program
  let mut conn: Connection = Connection{
    stream: None,
    addr: "".to_string(),
    cwd: "".to_string(),
  };

  // If server address was provided in program args, try to connect
  if args.len() > 1 {
    let connect_instruction = vec!["connect", &args[1]];
    match conn.process_command(&connect_instruction) {
      Ok(_) => println!("Connected"),
      Err(e) => println!("{e}"),
    }
  }

  
  while !quit {
    // Print out prompt
    // println!("");
    if conn.cwd.is_empty() {
      print!("parfs-client>");
    }
    else{
      print!("{}>", &conn.cwd);
    }
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
    let result: Result<(), ClientError> = conn.process_command(&tokens);
    if let Err(err) = result {
      println!("{}", err.to_string());
      continue;
    }

  }

  exit(0);

}


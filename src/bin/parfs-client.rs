extern crate parfs;
use std::{io::{self, Write}, process::exit};
use parfs::client::connection::Connection;

fn main() {

  // Indicator for user requested quit
  let mut quit: bool = false;

  // Connection that will be used by this client program
  let mut conn: Connection = Connection{
    stream: None,
    addr: "".to_string(),
    cwd: "".to_string(),
  };
  
  while !quit {
    // Print out prompt
    println!("");
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
    let result: Result<(), String> = conn.process_command(&tokens);
    if let Err(err) = result {
      println!("{}", err);
      continue;
    }

  }

  exit(0);

}

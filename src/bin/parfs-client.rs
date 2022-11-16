use std::{io::{self, Write}, process::exit};

fn main() {
  let mut quit: bool = false;
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
    }

    // Process the command
    let result: Result<(), String> = process_command(tokens);
    if let Err(err) = result {
      println!("{}", err);
    }

  }

  exit(0);

}

fn process_command(tokens: Vec<&str>) -> Result<(), String>{
  return Ok(());
}

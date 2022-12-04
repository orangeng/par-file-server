use std::slice::Iter;

#[derive(Debug)]
pub enum Command {
  Connect,
  Help,
  Login, 
  Mkdir,
  Cd,
  Ls,
  Up,
  Down,
  Status
}

impl Command {
  pub fn get_desc(&self) -> String {
    match self {
      Command::Connect => "Establishes a connection to a file server. Usage: connect [socket-addr]".to_string(),
      Command::Login => "Logs in. Usage: (fill in)".to_string(),
      Command::Mkdir => "Makes a folder in the current working directory. Usage: mkdir [name]".to_string(),
      Command::Cd => "Changes the current working directory. Usage: cd [path]".to_string(),
      Command::Ls => "Lists the files in the current working directory. Usage: ls".to_string(),
      Command::Up => "Uploads a file from the local computer to the server. Usage: up [path-to-file]".to_string(),
      Command::Down => "Downloads a file from the server to the local computer. Usage: (placeholder)".to_string(),
      _ => "this should never show up".to_string()
    }
  }

  pub fn get_str(&self) -> String {
    match self {
      Command::Connect => "connect".to_string(),
      Command::Login => "login".to_string(),
      Command::Mkdir => "mkdir".to_string(),
      Command::Cd => "cd".to_string(),
      Command::Ls => "ls".to_string(),
      Command::Up => "up".to_string(),
      Command::Down => "down".to_string(),
      _ => "this should never show up".to_string()
    }
  }

  pub fn iterator() -> Iter<'static, Command> {
    static COMMANDS: [Command; 7] = [Command::Connect, Command::Login, Command::Mkdir, Command::Cd, Command::Ls, Command::Up, Command::Down];
    COMMANDS.iter()
  }
}
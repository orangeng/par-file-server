use std::{
    io,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    net::TcpStream,
    path::{PathBuf, Path},
    fs, mem,
};

use crate::message::{*, self};

pub struct ConnectionHandler {
    tcpstream: TcpStream,
    home_directory: PathBuf,
    current_directory: PathBuf,
}

impl ConnectionHandler {
    //make a new connectionhandler which encapsulates the connection from the server's side! wow!
    pub fn new(stream: TcpStream, home_directory: PathBuf) -> io::Result<Self> {
        let handler = Self {
            tcpstream: stream,
            home_directory,
            current_directory: PathBuf::new(),
        };
        let welcome_message =
            MessageSender::new(MessageKind::Success, "Welcome to parfs".to_string(), None, &handler.tcpstream);
        welcome_message.send_message()?;
        return Ok(handler);
    }

    // main loop of the handler 
    pub fn handle_connection(mut self) {
        loop {
            let tcp_reader = BufReader::new(&self.tcpstream);
            let client_request = match MessageReceiver::new(tcp_reader) {
                Ok(message) => message,
                Err(e) => panic!("message forming failed :( {}", e),
            };
            print!("Recieved request..");
            print!("{}", &client_request.command_string);
            
            // match statements to process different commands
            let result = match &client_request.command {
                MessageKind::Mkdir => {
                    self.mkdir(client_request.command_string)
                },
                MessageKind::Cd => {
                    self.cd(client_request.command_string)
                },
                MessageKind::Ls => {
                    self.ls(client_request.command_string)
                },
                // MessageKind::Up => {

                // },
                // MessageKind::Down => {

                // },
                _ => {Ok("ok".to_string())},
            };
            match result {
                Ok(success_message) => {
                    let response = MessageSender::new( MessageKind::Success,  success_message,  None,  &self.tcpstream );
                    response.send_message();
                }
                Err(e) => {
                    let response = MessageSender::new( MessageKind::Error,  e.to_string(),  None,  &self.tcpstream );
                    response.send_message();
                }
            }
        }
    }

    fn mkdir(&self, dir_name:String) -> io::Result<String>{
        let mut new_dir = Path::new(&dir_name);
        if new_dir.starts_with("/") {
            new_dir = new_dir.strip_prefix("/").expect("This really shouldnt fail");
        }

        fs::create_dir(Path::new(&self.home_directory).join(&self.current_directory).join(new_dir))?;
        return Ok("".to_string());
    }

    fn cd(&mut self, path_name:String) -> io::Result<String> {
        let mut new_path = PathBuf::from(path_name);
        if new_path.starts_with("/") {
            new_path = PathBuf::from(new_path.strip_prefix("/").expect("This really shouldnt fail"));
        }
        if Path::new(&self.home_directory).join(&new_path).exists() {
            self.current_directory = new_path;
            return Ok("".to_string());
        } else {
            return Err(std::io::Error::new( io::ErrorKind::NotFound,"that path doesnt exist m8"))
        }
    }

    fn ls(&self, path_name:String) -> io::Result<String> {
        return Ok("this aint working yet".to_string()) 
        // let mut new_path = PathBuf::from(path_name);
        // if new_path.starts_with("/") {
        //     new_path = PathBuf::from(new_path.strip_prefix("/").expect("This really shouldnt fail"));
        // }
        // if new_path.exists() {
        //     self.current_directory = new_path;
        //     return Ok("".to_string());
        // } else {
        //     return Err(std::io::Error::new( io::ErrorKind::NotFound,"that path doesnt exist m8"))
        // }
    }

}

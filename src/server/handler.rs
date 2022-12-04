use std::{
    fs, io,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    mem,
    net::TcpStream,
    path::{Path, PathBuf},
};

use crate::message::{self, *};

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
        let welcome_message = MessageSender::new(
            MessageKind::Success,
            "Welcome to parfs".to_string(),
            None,
            &handler.tcpstream,
        );
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
                MessageKind::Mkdir => self.mkdir(client_request.command_string),
                MessageKind::Cd => self.cd(client_request.command_string),
                MessageKind::Ls => self.ls(),
                // MessageKind::Up => {

                // },
                // MessageKind::Down => {

                // },

                //place holder
                _ => Ok(self.error_message("Command not implemented".to_string())),
            };
            match result {
                Ok(message) => {
                    message.send_message();
                }
                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
    }

    fn mkdir(&self, dir_name: String) -> io::Result<MessageSender> {
        let mut new_dir = Path::new(&dir_name);
        if new_dir.starts_with("/") {
            new_dir = new_dir
                .strip_prefix("/")
                .expect("This really shouldnt fail");
        }

        fs::create_dir(
            Path::new(&self.home_directory)
                .join(&self.current_directory)
                .join(new_dir),
        )?;
        return Ok(self.success_message(None));
    }

    fn cd(&mut self, path_name: String) -> io::Result<MessageSender> {
        let mut new_path = PathBuf::from(path_name);
        if new_path.starts_with("/") {
            new_path = PathBuf::from(
                new_path
                    .strip_prefix("/")
                    .expect("This really shouldnt fail"),
            );
        }
        if Path::new(&self.home_directory).join(&new_path).exists() {
            self.current_directory = new_path;
            return Ok(self.success_message(None));
        } else {
            return Err(std::io::Error::new(
                io::ErrorKind::NotFound,
                "that path doesnt exist m8",
            ));
        }
    }

    fn ls(&self) -> io::Result<MessageSender> {
        let paths = match fs::read_dir(&self.home_directory.join(&self.current_directory)) {
            Ok(result) => result,
            Err(e) => panic!("{}", e),
        };

        // joins the paths in the the iterator paths with "\n" 
        let output: String = paths
            .into_iter()
            .map(|x| x.unwrap().path().to_string_lossy().into_owned())
            .collect::<Vec<String>>()
            .join("\n");

        return Ok(self.success_message(Some(output)));
    }

    fn success_message(&self, message_string: Option<String>) -> MessageSender {
        let message_string = match message_string {
            Some(string) => string,
            None => "".to_string(),
        };
        return MessageSender::new(MessageKind::Success, message_string, None, &self.tcpstream);
    }

    fn error_message(&self, message_string: String) -> MessageSender {
        return MessageSender::new(MessageKind::Error, message_string, None, &self.tcpstream);
    }
}

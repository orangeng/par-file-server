use std::{
    fs, io::{self},
    io::{BufReader, Error},
    net::TcpStream,
    path::{Path, PathBuf},
};

use crate::message::*;

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
            "Welcome to parfs!".to_string(),
            None,
            &handler.tcpstream,
        );
        welcome_message.send_message()?;
        return Ok(handler);
    }

    // main loop of the handler
    pub fn handle_connection(mut self) {
        loop {
            // Creates a MessageReceiver and waits for incoming messages
            let tcp_reader: BufReader<&TcpStream> = BufReader::new(&self.tcpstream);
            let client_request: MessageReceiver = match MessageReceiver::new(tcp_reader) {
                Ok(message) => message,
                Err(e) => panic!("message forming failed :( {}", e),
            };

            // Confirms received request
            println!("Received request..");
            println!("{:?}", &client_request.command);
            println!("{}", &client_request.arguments);

            // Group of match statements to process different commands
            let message_kind: MessageKind = client_request.command;
            let arguments: String = client_request.arguments;
            let result: Result<(), Error> = match message_kind {
                MessageKind::Mkdir => self.mkdir(arguments),
                MessageKind::Cd => self.cd(arguments),
                MessageKind::Ls => self.ls(),
                // MessageKind::Up => {

                // },
                // MessageKind::Down => {

                // },

                //place holder
                _ => Ok(()),
            };

            if let Err(e) = result{
                println!("{}", e);
            }

        }
    }

    fn mkdir(&self, dir_name: String) -> io::Result<()> {
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
        let success_message: MessageSender = self.success_message(None);
        success_message.send_message()?;
        return Ok(());
    }

    fn cd(&mut self, path_name: String) -> io::Result<()> {
        let mut new_path = PathBuf::from(path_name);
        // TODO: implement some error checking for filename
        if new_path.starts_with("/") {
            new_path = PathBuf::from(
                new_path
                    .strip_prefix("/")
                    .expect("This really shouldnt fail"),
            );
        }

        // Sends success message if path exists
        if Path::new(&self.home_directory).join(&new_path).exists() {
            self.current_directory = new_path;
            let success_message: MessageSender = self.success_message(None);
            success_message.send_message()?;
            return Ok(());
        } 
        
        // Sends error message if file does not exists
        else {
            let error_message: MessageSender = self.error_message("File path does not exist!".to_string());
            error_message.send_message()?;
            return Ok(());
        }
    }

    fn ls(&self) -> io::Result<()> {
        let paths = match fs::read_dir(&self.home_directory.join(&self.current_directory)) {
            Ok(result) => result,
            Err(e) => panic!("{}", e),
        };

        // Joins the paths in the the iterator paths with "\n" 
        let output: String = paths
            .into_iter()
            .map(|x| x.unwrap().path().to_string_lossy().into_owned())
            .collect::<Vec<String>>()
            .join("\n");

            let success_message: MessageSender = self.success_message(Some(output));
            success_message.send_message()?;
        
        return Ok(());
    }

    // Creates a MessageSender of MessageKind::Success
    fn success_message(&self, message_string: Option<String>) -> MessageSender {
        let message_string = match message_string {
            Some(string) => string,
            None => "".to_string(),
        };
        return MessageSender::new(MessageKind::Success, message_string, None, &self.tcpstream);
    }

    // Creates a MessageSender of MessageKind::Error
    fn error_message(&self, message_string: String) -> MessageSender {
        return MessageSender::new(MessageKind::Error, message_string, None, &self.tcpstream);
    }

}

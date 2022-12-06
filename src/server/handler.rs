use std::{
    fs, io::{self},
    io::{BufReader, BufWriter, Error, ErrorKind},
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
            handler.home_directory.to_str().unwrap().to_string(),
            None,
        );
        let final_msg_result = welcome_message.send_message(&handler.tcpstream);

        if let Err(e) = final_msg_result{
            println!("{}", e);
        }

        return Ok(handler);
    }

    // main loop of the handler
    pub fn handle_connection(&mut self) {
        loop {

            println!("New iteration of handle_connection()...");

            // Creates a MessageReceiver and waits for incoming messages
            let client_request: MessageReceiver = match MessageReceiver::new(&self.tcpstream) {
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
            let result: Result<MessageSender, Error> = match message_kind {
                MessageKind::Mkdir => self.mkdir(arguments),
                MessageKind::Cd => self.cd(arguments),
                MessageKind::Ls => self.ls(),
                MessageKind::Down => self.down(arguments),
                // MessageKind::Up => {

                // },

                //place holder
                _ => Err(Error::new(ErrorKind::Other, "Message type couldn't be found :<")),
            };

            // Ok() will be the MessageSender created by the individual functions, be it Success / Error
            // Err() will be errors propagated by ? in other parts of the function
            let final_msg_result : Result<(), Error> = match result{
                Ok(message) => {
                    message.send_message(&self.tcpstream)
                },
                Err(e) => {
                    println!("{}", e);
                    let generic_server_err: String = "There was an error at the server. Please try again!".to_string();
                    let error_message: MessageSender = self.error_message(generic_server_err);
                    error_message.send_message(&self.tcpstream)
                },
            };

            if let Err(e) = final_msg_result{
                println!("{}", e);
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
            let mut full_path = PathBuf::from(&self.home_directory);
            full_path.push(&self.current_directory);
            return Ok(self.success_message(Some(full_path.to_str().unwrap().to_string())));
        }
        
        // Sends error message if file does not exists
        else {
            return Ok(self.error_message("File path does not exist!".to_string()));
        }
    }

    fn ls(&self) -> io::Result<MessageSender> {
        let paths = fs::read_dir(&self.home_directory.join(&self.current_directory))?;

        // Joins the paths in the the iterator paths with "\n" 
        let output: String = paths
            .into_iter()
            .map(|x| x.unwrap().path().to_string_lossy().into_owned())
            .collect::<Vec<String>>()
            .join("\n");
        
        return Ok(self.success_message(Some(output)));
    }

    fn down(&self, file_name: String) -> io::Result<MessageSender> {
        let mut file_path: PathBuf = PathBuf::from(&self.home_directory);
        file_path.push(&self.current_directory);
        file_path.push(file_name.as_str());
        println!("{}", file_path.to_str().unwrap());
        let file_sender: MessageSender = MessageSender::new(
            MessageKind::File,
            "".to_string(),
            Some(file_path),
        );

        return Ok(file_sender);
        // match file_sender.send_message(BufWriter::new(&self.tcpstream)){
        //     Ok(()) => Ok(self.success_message(None)),
        //     Err(..) => Ok(self.error_message("File could not be sent out from server!".to_string())),
        // }
    }

    fn up(&self, file_name: String) -> io::Result<MessageSender> {
        let mut file_path: PathBuf = PathBuf::from(&self.home_directory);
        file_path.push(&self.current_directory);
        file_path.push(file_name.as_str());
        println!("{}", file_path.to_str().unwrap());
        let file_sender: MessageSender = MessageSender::new(
            MessageKind::File,
            "".to_string(),
            Some(file_path),
        );

        return Ok(file_sender);
    }

    // Creates a MessageSender of MessageKind::Success
    fn success_message(&self, message_string: Option<String>) -> MessageSender {
        let message_string = match message_string {
            Some(string) => string,
            None => "".to_string(),
        };
        return MessageSender::new(MessageKind::Success, message_string, None);
    }

    // Creates a MessageSender of MessageKind::Error
    fn error_message(&self, message_string: String) -> MessageSender {
        return MessageSender::new(MessageKind::Error, message_string, None);
    }

}

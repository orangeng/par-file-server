use std::fs;
use std::io::{self, Error, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use crate::message::MessageKind;
use crate::server::message::receiver::MessageReceiver;
use crate::server::message::sender::MessageSender;
use crate::server::utilities::*;
use crate::utilities::format_error;

use super::fsrw_mutex::FsrwMutex;

pub struct ConnectionHandler {
    tcpstream: TcpStream,
    home_directory: PathBuf,
    current_directory: PathBuf,
    connection_dropped: bool,
    fsrw_mutex: Arc<FsrwMutex>,
    addr: String,
}

// To do:: Have a proper way to indicate when the connection is dropped

impl ConnectionHandler {
    //make a new connectionhandler which encapsulates the connection from the server's side! wow!
    pub fn new(
        stream: TcpStream,
        home_directory: PathBuf,
        fsrw_mutex: Arc<FsrwMutex>,
        addr: String
    ) -> io::Result<Self> {
        println!("New connection started");
        let handler = Self {
            tcpstream: stream,
            home_directory: home_directory.clone(),
            current_directory: home_directory,
            connection_dropped: false,
            fsrw_mutex,
            addr
        };

        return Ok(handler);
    }

    // main loop of the handler
    pub fn handle_connection(mut self, port: usize) {
        
        // Shift request to new port
        println!("Request now being shifted to port {}", port);
        let bytes: [u8; 4] = (port as i32).to_le_bytes();
        self.tcpstream.write(&bytes);

        // Listen and capture incoming connection on new port
        let addr_split: Vec<&str> = self.addr.split(":").collect();
        let ip_addr = addr_split[0];
        let new_addr: &str = &(ip_addr.to_string() + ":" + port.to_string().as_str());
        println!("New address to connect to: {}", new_addr);
        let listener: TcpListener = TcpListener::bind(new_addr).unwrap();

        for stream in listener.incoming() {
            self.tcpstream = stream.unwrap();
            break;
        }
        
        let welcome_message =
            MessageSender::new(MessageKind::Success, self.get_display_path(&self.current_directory), None);
        let final_msg_result =
            welcome_message.send_message(&self.tcpstream, &self.fsrw_mutex);

        if let Err(e) = final_msg_result {
            println!("{}", e);
        }

        loop {
            println!("New iteration of handle_connection()...");

            // Creates a MessageReceiver and waits for incoming messages
            let client_request = self.receive_message();
            if self.connection_dropped {
                self.exit();
                return;
            }
            let client_request = client_request.unwrap();

            // Confirms received request
            println!("Received request..");
            println!("{:?}", &client_request.command);
            println!("{}", &client_request.arguments);

            // Group of match statements to process different commands
            // Validation of command is done within each command
            let message_kind: MessageKind = client_request.command;
            let arguments: String = client_request.arguments;
            let result: Result<MessageSender, Error> = match message_kind {
                MessageKind::Mkdir => self.mkdir(arguments),
                MessageKind::Cd => self.cd(arguments),
                MessageKind::Ls => self.ls(),
                MessageKind::Down => self.down(arguments),
                MessageKind::Up => self.up(arguments),
                //place holder
                _ => Err(Error::new(
                    ErrorKind::Other,
                    "Message type couldn't be found :<",
                )),
            };

            // Ok() will be the MessageSender created by the individual functions, be it Success / Error
            // Err() will be errors propagated by ? in other parts of the function
            let final_msg_result: Result<(), Error> = match result {
                Ok(message) => message.send_message(&self.tcpstream, &self.fsrw_mutex),
                Err(e) => {
                    if e.kind() == ErrorKind::UnexpectedEof {
                        self.exit();
                        return;
                    }
                    println!("{}", e);
                    let generic_server_err: String =
                        "There was an error at the server. Please try again!".to_string();
                    let error_message: MessageSender = self.error_message(generic_server_err);
                    error_message.send_message(&self.tcpstream, &self.fsrw_mutex)
                }
            };

            if let Err(e) = final_msg_result {
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

        let file_path = PathBuf::from(&self.current_directory).join(new_dir);
        if self.is_valid_file(&file_path) {
            return Ok(self.error_message(format_error(ERR_NO_DIR,&dir_name)))
        }
        fs::create_dir(file_path)?;
        return Ok(self.success_message(None));
    }

    fn cd(&mut self, path_name: String) -> io::Result<MessageSender> {
        let mut new_path = PathBuf::from(&path_name);
        if new_path.starts_with("/") {
            new_path = PathBuf::from(
                new_path
                    .strip_prefix("/")
                    .expect("This really shouldnt fail"),
            );
        }

        new_path = Path::new(&self.current_directory).join(&new_path);
        if self.is_valid_directory(&new_path) {
            // Sends success message if path exists
            new_path = Path::canonicalize(&Path::new(&self.current_directory).join(new_path))?;
            self.current_directory = new_path;
            return Ok(self.success_message(Some(self.get_display_path(&self.current_directory))));
        }
        // Sends error message if file does not exists
        else {
            return Ok(self.error_message(format_error(
                ERR_NO_DIR,
                &path_name,
            )));
        }
    }

    fn ls(&self) -> io::Result<MessageSender> {
        let paths = fs::read_dir(&self.current_directory)?;

        // Joins the paths in the the iterator paths with "\n". If the entry is a directory, append a "/" to the end
        let output: String = paths
            .into_iter()
            .map(|x| -> String {
                let path = x.unwrap().path();
                if path.is_dir() {
                    let mut new_str = String::from(path.iter().last().unwrap().to_str().unwrap());
                    new_str.push_str("/");
                    new_str
                } else {
                    String::from(path.iter().last().unwrap().to_str().unwrap())
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        return Ok(self.success_message(Some(output)));
    }

    fn down(&self, file_name: String) -> io::Result<MessageSender> {
        let mut file_path: PathBuf = PathBuf::from(&self.current_directory);
        file_path.push(file_name.as_str());
        println!("{}", file_path.to_str().unwrap());
        if self.is_valid_file(&file_path) {
            let file_sender: MessageSender =
                MessageSender::new(MessageKind::File, "".to_string(), Some(file_path));

            return Ok(file_sender);
        } else {
            return Ok(self.error_message(format_error(ERR_NO_PATH, &file_name)));
        }
    }

    // For the server to handle an up, it will first send a success to the client
    // to indicate that it is ready to receive a file.
    fn up(&mut self, file_name: String) -> io::Result<MessageSender> {
        let mut file_path: PathBuf = PathBuf::from(&self.current_directory);
        file_path.push(file_name.as_str());

        // Check if file to be written to is a file. If not, check if the parent is a directory. If not, send an error message.
        if !self.is_valid_file(&file_path) {
            if file_path.parent().is_none() {
                return Ok(
                    self.error_message(format_error(ERR_NO_PATH,&file_name)) 
                );
            } else {
                if !self.is_valid_directory(&PathBuf::from(file_path.parent().unwrap())){
                    return Ok(
                        self.error_message(format_error(ERR_NO_PATH, &file_name))
                    );
                }
            }
        }
        println!("Ready to receive {:?}", file_path);
        self.success_message(None)
            .send_message(&self.tcpstream, &self.fsrw_mutex)?;
        let file_message = match self.receive_message() {
            Some(message) => message,
            None => return Err(Error::new(ErrorKind::UnexpectedEof, "Connection closed")),
        };
        if file_message.command != MessageKind::File {
            panic!("Received wrong message kind from client!");
        } else {
            file_message.write_to(&self.tcpstream, file_path, &self.fsrw_mutex)?;
        };
        return Ok(self.success_message(None));
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

    fn receive_message(&mut self) -> Option<MessageReceiver> {
        match MessageReceiver::new(&self.tcpstream) {
            Ok(message) => Some(message),
            Err(e) => {
                // checks for connection being dropped
                if e.kind() == ErrorKind::UnexpectedEof {
                    self.connection_dropped = true;
                    return None;
                } else {
                    panic!("message forming failed :( {:?}", e);
                }
            }
        }
    }

    fn exit(&self) {
        println!("Connection shutdown");
        self.tcpstream.shutdown(std::net::Shutdown::Both);
    }

    fn get_display_path(&self, path: &PathBuf) -> String {
        let home_folder_name = &self.home_directory.iter().last().unwrap();
        let display_path: PathBuf = 
            path
            .clone()
            .iter()
            .skip_while(|s| **s != **home_folder_name)
            .skip(1)
            .collect();
        let mut display_string = String::from(
            display_path
                .to_str()
                .expect("Converting display path to string failed"),
        );
        if display_string.is_empty() {
            display_string.push_str("~/");
        } else {
            display_string = "~/".to_owned() + &display_string + "/";
        }
        return display_string;
    }

    // Checks if new path is within the sandboxed folder
    fn is_valid_directory(&self, path: &PathBuf) -> bool {
        if path.exists() && path.is_dir() {
            // JANK WAY TO CHECK: Merely checks if the home folder name is within the new path. Can obviously be bypassed if there are other folders with the same name as the home folder.
            let simplified_path = path.canonicalize().unwrap();
            println!("Checking valid dir: {:?}",&simplified_path);
            let home_folder_name = &self.home_directory.iter().last().unwrap();
            let path_from_current_directory: PathBuf = simplified_path
                .clone()
                .iter()
                .skip_while(|s| **s != **home_folder_name)
                .collect();
            if !path_from_current_directory.as_os_str().is_empty() {
                return true;
            }
        }
        return false;
    }

    // Checks if file is within the sandboxed folder
    fn is_valid_file(&self, path: &PathBuf) -> bool {
        if path.exists() && path.is_file() {
            // JANK WAY TO CHECK: Merely checks if the home folder name is within the new path. Can obviously be bypassed if there are other folders with the same name as the home folder.
            let simplified_path = path.canonicalize().unwrap();
            println!("Checking valid file: {:?}",&simplified_path);
            let home_folder_name = &self.home_directory.iter().last().unwrap();
            let path_from_current_directory: PathBuf = simplified_path
                .clone()
                .iter()
                .skip_while(|s| **s != **home_folder_name)
                .collect();
            println!("Got: {:?}", &path_from_current_directory.as_os_str());
            if !path_from_current_directory.as_os_str().is_empty() {
                return true;
            }
        }
        return false;
    }
}

// experimental method for sending an error message after shutting down
// TODO: maybe make this a shutdown message?
impl ::std::ops::Drop for ConnectionHandler {
    // TODO: make this send it to all clients
    fn drop(&mut self) {
        let error_message: MessageSender = self.error_message("The server has been dropped, and you are now disconnected.".to_string());
        let final_msg_result = error_message.send_message(&self.tcpstream, &self.fsrw_mutex);
        if let Err(e) = final_msg_result {
            println!("{}", e);
        }
        println!("Connection shutdown");
        self.tcpstream.shutdown(std::net::Shutdown::Both);
    }
}
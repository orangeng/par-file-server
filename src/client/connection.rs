use crate::client::utilities::*;
use regex::Regex;
use std::io::{Error, Read};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

use crate::client::errors::*;
use crate::message::MessageKind;
use crate::client::message::receiver::MessageReceiver;
use crate::client::message::sender::MessageSender;

// 30 seconds connection try
const CONNECTION_TIMEOUT: Duration = Duration::new(30, 0);
const PORT_SWITCHING_TRIES: i32 = 50;
pub struct Connection {
    pub stream: Option<TcpStream>,
    pub addr: String,
    pub cwd: String,
}

impl Connection {
    // This function returns a Result. The Err(String) contains the string that will be
    // printed on the user interface
    pub fn process_command(&mut self, tokens: &Vec<&str>) -> Result<(), ClientError> {
        // Check if command is valid
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
            _ => return Err(ClientError::InvalidCommand),
        };

        // "connect" command
        if let Command::Connect = command_type {
            self.connect(&tokens)?;
            return Ok(());
        }

        // "help" command
        if let Command::Help = command_type {
            self.help();
            return Ok(());
        }
        // "status" command
        else if let Command::Status = command_type {
            self.status();
            return Ok(());
        }

        /******************** COMMANDS FROM HERE ONWARDS REQUIRE A WORKING TCPSTREAM ********************/

        // Check if stream is established
        if self.stream.is_none() {
            return Err(ClientError::ConnectionError);
        }

        match command_type {
            Command::Cd => self.cd(&tokens)?,
            Command::Ls => self.ls(&tokens)?,
            Command::Down => self.down(&tokens)?,
            Command::Up => self.up(&tokens)?,
            Command::Mkdir => self.mkdir(&tokens)?,
            _ => return Err(ClientError::InvalidCommand),
        }

        return Ok(());
    }

    fn connect(&mut self, tokens: &Vec<&str>) -> Result<(), ClientError> {
        // Some return strings
        let help: String =
            "Help:\n\tconnect [socket-addr]\n\t[socket-addr]: 'ip-addr:port' e.g. 127.0.0.1:12800"
                .to_string();

        // Insufficient / wrong no of arguments
        if tokens.len() != 2 {
            return Err(ClientError::WrongArgumentNum(help));
        }

        // Validate socket address
        let addr: &str = tokens[1];
        let socket_addr_re = Regex::new(r"^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d{1,5}$").unwrap();
        let valid: bool = socket_addr_re.is_match(addr);
        if !valid {
            return Err(ClientError::InvalidAddress(help));
        }

        let stream_result: Result<TcpStream, Error> =
            TcpStream::connect_timeout(&addr.parse().unwrap(), CONNECTION_TIMEOUT);
        if stream_result.is_err() {
            return Err(ClientError::ConnectionError);
        }

        // If connection opened successfully
        let mut stream: TcpStream = stream_result.unwrap();

        // Receives port number for when server demands a connection on a new port
        let mut buf: [u8; 4] = [0; 4];
        let port_read_result = stream.read(&mut buf);
        if port_read_result.is_err() {
            return Err(ClientError::ConnectionError);
        }
        let addr_split: Vec<&str> = addr.split(":").collect();
        let ip_addr = addr_split[0];
        let new_port: i32 = i32::from_le_bytes(buf);
        let new_addr: &str = &(ip_addr.to_string() + ":" + new_port.to_string().as_str());
        println!("New address to connect to: {}", new_addr);

        // Connect to the new port and retry if it doesn't work.
        for i in 0..PORT_SWITCHING_TRIES {
            let stream_result: Result<TcpStream, Error> =
                TcpStream::connect_timeout(&new_addr.parse().unwrap(), CONNECTION_TIMEOUT);
            if stream_result.is_err() {
                if i == PORT_SWITCHING_TRIES - 1 {
                    return Err(ClientError::ConnectionError);
                }
            } else {
                let stream = stream_result.unwrap();
                // Receives welcome message from server
                let confirmation_message: MessageReceiver = match MessageReceiver::new(&stream) {
                    Ok(server_message) => server_message,
                    Err(e) => {
                        return Err(ClientError::IOError(e.to_string()));
                    }
                };
                // Read in home directory on server and store it
                match confirmation_message.command {
                    MessageKind::Success => {
                        self.cwd = confirmation_message.arguments;
                        self.stream = Some(stream);
                        self.addr = new_addr.to_string();
                        println!("Welcome to parfs!");
                        return Ok(());
                    }
                    _ => {
                        return Err(ClientError::MessageError);
                    }
                };
            }
        }

        return Err(ClientError::ConnectionError);
    }

    fn cd(&mut self, tokens: &Vec<&str>) -> Result<(), ClientError> {
        let help: String = "Help:\n\tcd [file path]".to_string();

        // currently only supports non-spaced file paths
        // TODO: support quotation file paths
        if tokens.len() != 2 {
            return Err(ClientError::WrongArgumentNum(help));
        }
        let tcp_stream: &TcpStream = match &self.stream {
            Some(tcp) => &tcp,
            None => {
                return Err(ClientError::ConnectionError);
            }
        };

        // Sends cd request
        let message_sender: MessageSender =
            MessageSender::new(MessageKind::Cd, tokens[1].to_string(), None);
        let ms_result: Result<(), Error> = message_sender.send_message(tcp_stream);
        match ms_result {
            Err(e) => {
                return Err(ClientError::IOError(e.to_string()));
            }
            _ => {}
        }

        // Read in request output from server
        let confirmation_message: MessageReceiver = match MessageReceiver::new(&tcp_stream) {
            Ok(server_message) => server_message,
            Err(e) => {
                return Err(ClientError::IOError(e.to_string()));
            }
        };

        match confirmation_message.command {
            MessageKind::Success => {
                self.cwd = confirmation_message.arguments;
                return Ok(());
            }
            MessageKind::Error => {
                println!("{}", &confirmation_message.arguments);
                return Ok(());
            }
            _ => Err(ClientError::MessageError),
        }
    }

    fn ls(&self, tokens: &Vec<&str>) -> Result<(), ClientError> {
        // Some return strings
        let help: String = "Help:\n\tls".to_string();

        // Insufficient / wrong no of arguments
        if tokens.len() != 1 {
            return Err(ClientError::WrongArgumentNum(help));
        }

        // Borrow the TcpStream
        let tcp_stream: &TcpStream = match &self.stream {
            Some(tcp) => &tcp,
            None => {
                return Err(ClientError::ConnectionError);
            }
        };

        // Sends ls request
        let message_sender: MessageSender =
            MessageSender::new(MessageKind::Ls, "".to_string(), None);
        let ms_result: Result<(), Error> = message_sender.send_message(&tcp_stream);
        match ms_result {
            Err(e) => {
                return Err(ClientError::IOError(e.to_string()));
            }
            _ => {}
        }

        // Read in request output from server
        let confirmation_message: MessageReceiver = match MessageReceiver::new(&tcp_stream) {
            Ok(server_message) => server_message,
            Err(e) => {
                return Err(ClientError::IOError(e.to_string()));
            }
        };

        match confirmation_message.command {
            MessageKind::Success => {
                println!("{}", confirmation_message.arguments);
                return Ok(());
            }
            MessageKind::Error => {
                println!("{}", &confirmation_message.arguments);
                return Ok(());
            }
            _ => Err(ClientError::MessageError),
        }
    }

    fn mkdir(&self, tokens: &Vec<&str>) -> Result<(), ClientError> {
        // Some return strings
        let help: String = "Help:\n\tmkdir".to_string();

        // Insufficient / wrong no of arguments
        if tokens.len() != 2 {
            return Err(ClientError::WrongArgumentNum(help));
        }

        // Borrow the TcpStream
        let tcp_stream: &TcpStream = match &self.stream {
            Some(tcp) => &tcp,
            None => {
                return Err(ClientError::ConnectionError);
            }
        };

        // Sends mkdir request
        let message_sender: MessageSender =
            MessageSender::new(MessageKind::Mkdir, tokens[1].to_string(), None);
        let ms_result: Result<(), Error> = message_sender.send_message(&tcp_stream);
        match ms_result {
            Err(e) => {
                return Err(ClientError::IOError(e.to_string()));
            }
            _ => {}
        }

        // Read in request output from server
        let confirmation_message: MessageReceiver = match MessageReceiver::new(&tcp_stream) {
            Ok(server_message) => server_message,
            Err(e) => {
                return Err(ClientError::IOError(e.to_string()));
            }
        };

        match confirmation_message.command {
            MessageKind::Success => {
                return Ok(());
            }
            MessageKind::Error => {
                println!("{}", &confirmation_message.arguments);
                return Ok(());
            }
            _ => Err(ClientError::MessageError),
        }
    }

    fn down(&self, tokens: &Vec<&str>) -> Result<(), ClientError> {
        let help: String = "Help:
    \tdown [server-file] [local-dest]
    \t[server-file]: 'quicksort.pdf'
    \t[local-dest]: '/home/user/parfs-receive/'"
            .to_string();

        // currently only supports non-spaced file paths
        // TODO: support quotation file paths
        if tokens.len() != 3 {
            return Err(ClientError::WrongArgumentNum(help));
        }
        let tcp_stream: &TcpStream = match &self.stream {
            Some(tcp) => &tcp,
            None => {
                return Err(ClientError::ConnectionError);
            }
        };
        //Check that download location is valid
        let mut download_location = PathBuf::from(tokens[2]);
        if download_location.is_dir() {
            download_location = download_location.join(tokens[1]);
        }
        if (!download_location.exists() && !download_location.parent().unwrap().is_dir()) || download_location.is_dir(){
            return Err(ClientError::DestinationError(tokens[2].to_string()));
        }

        // Sends down request
        let message_sender: MessageSender =
            MessageSender::new(MessageKind::Down, tokens[1].to_string(), None);
        let ms_result: Result<(), Error> = message_sender.send_message(&tcp_stream);
        match ms_result {
            Err(e) => {
                return Err(ClientError::IOError(e.to_string()));
            }
            _ => {}
        }

        // Receives incoming payload
        let payload_message: MessageReceiver = match MessageReceiver::new(tcp_stream) {
            Ok(server_message) => server_message,
            Err(e) => {
                return Err(ClientError::IOError(e.to_string()));
            }
        };

        // Double check to see message is of MessageKind::File
        match payload_message.command {
            MessageKind::Error => {
                return Err(ClientError::DownloadError(payload_message.arguments))
            }
            MessageKind::File => (),
            _ => return Err(ClientError::MessageError),
        };

        // Start writing to local destination
        match payload_message.write_to(tcp_stream, PathBuf::from(download_location)) {
            Err(e) => return Err(ClientError::WriteError(e.to_string())),
            Ok(()) => Ok(()),
        }
    }

    fn up(&self, tokens: &Vec<&str>) -> Result<(), ClientError> {
        let help: String = "Help:
    \tup [local-file] [server-file]
    \t[local-file]: 'quicksort.pdf'
    \t[server-file]: 'quicksort.pdf'"
            .to_string();

        let tcp_stream: &TcpStream = match &self.stream {
            Some(tcp) => &tcp,
            None => {
                return Err(ClientError::ConnectionError);
            }
        };
        if tokens.len() != 3 {
            return Err(ClientError::WrongArgumentNum(help));
        }

        let mut file_path: PathBuf = PathBuf::from(tokens[1]);
        if !file_path.is_file() {
            return Err(ClientError::FileError(
                file_path.to_str().unwrap().to_string(),
            ));
        }
        // Sends down request
        let message_sender: MessageSender =
            MessageSender::new(MessageKind::Up, tokens[2].to_string(), None);
        let ms_result: Result<(), Error> = message_sender.send_message(&tcp_stream);
        match ms_result {
            Err(e) => {
                return Err(ClientError::IOError(e.to_string()));
            }
            _ => {}
        }
        // Receives incoming server message
        let server_message: MessageReceiver = match MessageReceiver::new(tcp_stream) {
            Ok(server_message) => server_message,
            Err(e) => {
                return Err(ClientError::IOError(e.to_string()));
            }
        };
        // Double check to see message is of MessageKind::Success
        match server_message.command {
            MessageKind::Error => return Err(ClientError::UploadError(server_message.arguments)),
            MessageKind::Success => (),
            _ => return Err(ClientError::MessageError),
        };

        //  Sending the file
        let file_message = MessageSender::new(MessageKind::File, "".to_string(), Some(file_path));
        match file_message.send_message(&tcp_stream) {
            Ok(_) => {}
            Err(e) => return Err(ClientError::IOError(e.to_string())),
        }

        // Check confirmation message
        let confirmation_message: MessageReceiver = match MessageReceiver::new(&tcp_stream) {
            Ok(server_message) => server_message,
            Err(e) => {
                return Err(ClientError::IOError(e.to_string()));
            }
        };

        match confirmation_message.command {
            MessageKind::Success => {
                return Ok(());
            }
            MessageKind::Error => {
                println!("{}", &confirmation_message.arguments);
                return Ok(());
            }
            _ => Err(ClientError::MessageError),
        }
    }

    fn help(&self) {
        for command in Command::iterator() {
            let command_str: String = command.get_str();
            println!(
                "{:?}{} {}",
                command_str,
                " ".repeat(20 - command_str.len()),
                command.get_desc()
            );
        }
    }

    fn status(&self) {
        if self.stream.is_none() {
            println!("{}", ClientError::ConnectionError.to_string());
            return;
        }
        println!("Connected to server at {}.", self.addr);
        println!("Current working directory is '{}'", self.cwd);
    }
}

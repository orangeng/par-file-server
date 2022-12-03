use std::{
    io,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    net::TcpStream,
    path::PathBuf,
};

use crate::message::*;

pub struct ConnectionHandler {
    tcpstream: TcpStream,
    home_directory: PathBuf,
}

impl ConnectionHandler {
    pub fn new(stream: TcpStream, home_directory: PathBuf) -> io::Result<Self> {
        let handler = Self {
            tcpstream: stream,
            home_directory,
        };
        let welcome_message =
            MessageSender::new(MessageKind::Success, "Welcome to parfs".to_string(), None);
        handler.send_message(welcome_message)?;
        return Ok(handler);
    }

    pub fn handle_connection(mut self) {
        loop {
            let tcp_reader = BufReader::new(&self.tcpstream);
            let mut client_request = match MessageReceiver::new(tcp_reader) {
                Ok(message) => message,
                Err(e) => panic!("summathing went wrong ere: {}", e),
            };
            self.process_message(client_request);
        }
    }

    fn process_message(&self, mut client_request: MessageReceiver) {
        match &client_request.command {
            MessageKind::Mkdir => {

            },
            MessageKind::Cd => {

            },
            MessageKind::Ls => {

            },
            MessageKind::Up => {

            },
            MessageKind::Down => {

            },
            _ => {},
        }
    }

    fn send_message(&self, message: MessageSender) -> io::Result<()> {
        let (headers, payload) = message.generate_message()?;
        let mut writer = BufWriter::new(&self.tcpstream);
        writer.write_all(&headers)?;
        match payload {
            Some(mut file) => {
                let mut length = 1;
                while length > 0 {
                    let buffer = file.fill_buf()?;
                    length = buffer.len();
                    file.consume(length);
                }
            }
            None => {}
        }
        return Ok(());
    }

    fn cd()
}

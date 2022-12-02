use std::{io, net::TcpStream, io::{BufReader, BufWriter, BufRead, Read, Write}};

use crate::message::*;

pub struct ConnectionHandler {
    tcpstream: TcpStream,
}

impl ConnectionHandler {
    pub fn new(stream: TcpStream) -> io::Result<Self>{
        let handler = Self{tcpstream: stream};
        let welcome_message = MessageSender::new(MessageKind::Success,
                                                                "Welcome to parfs".to_string(),
                                                                None);
        handler.send_message(welcome_message)?;
        return Ok(handler);
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
                    writer.write(buffer)?;
                    length = buffer.len();
                    file.consume(length);
                }
            },
            None => {}
        }
        return Ok(())
    }
}


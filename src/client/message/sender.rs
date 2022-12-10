use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::str::from_utf8;

use crate::client::utilities::print_progress;
use crate::message::{MessageKind, BUFFER_SIZE, HEADER_SIZE};

#[derive(Debug)]
pub struct MessageSender {
    pub command: MessageKind,
    pub arguments: String,
    pub file_path: Option<PathBuf>,
    pub payload_size: Option<u64>,
    // pub writer: BufWriter<&'a TcpStream>
}

// Note the use of big endian
impl MessageSender {
    // generator
    pub fn new(command: MessageKind, arguments: String, file_path: Option<PathBuf>) -> Self {
        Self {
            command,
            arguments,
            file_path,
            payload_size: None,
        }
    }

    // Blocking function!!!
    pub fn send_message(mut self, mut writer: &TcpStream) -> io::Result<()> {
        // println!("Sent message called once");
        // Generate message and send headers
        let (headers, payload) = self.generate_message()?;
        writer.write_all(&headers)?;

        // Send payload if any
        match payload {
            Some(mut file) => {
                let mut length = 1;
                let mut total_bytes_written: u64 = 0;
                let payload_size: u64 = self.payload_size.unwrap();
                while length > 0 {
                    print_progress(total_bytes_written, payload_size);
                    let buffer = file.fill_buf()?;
                    // println!("File to be sent: {:?}",buffer);
                    length = buffer.len();
                    total_bytes_written += length as u64;
                    writer.write_all(&buffer)?;
                    file.consume(length);
                }
                print!("\n");
            }
            None => {}
        }
        writer.flush()?;
        return Ok(());
    }

    // idk how to chain the vector and bufreader into a single iterator bro
    fn generate_message(&mut self) -> io::Result<(Vec<u8>, Option<BufReader<File>>)> {
        let mut payload_length: u64 = 0;
        let mut reader = None;
        match &self.file_path {
            Some(file_path) => {
                let file = File::open(file_path)?;
                payload_length = file.metadata()?.len();
                self.payload_size = Some(payload_length);
                reader = Some(BufReader::with_capacity(BUFFER_SIZE, file));
            }
            None => {}
        }
        let argument_bytes = self.arguments.as_bytes();
        let argument_length: u32 = argument_bytes.len().try_into().unwrap();

        let size: u64 = HEADER_SIZE as u64 + argument_length as u64 + payload_length;
        let mut headers: Vec<u8> = vec![];

        headers.extend(size.to_be_bytes());
        headers.push(self.command.clone() as u8);
        headers.extend(argument_length.to_be_bytes());
        headers.extend_from_slice(argument_bytes);
        Ok((headers, reader))
    }
}

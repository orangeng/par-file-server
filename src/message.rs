use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::str::from_utf8;

pub const HEADER_SIZE: usize = 13;
pub const BUFFER_SIZE: usize = 1048576;

// Refactor this rubbish with proper error handling, use custom types instead of io
// https://www.sheshbabu.com/posts/rust-error-handling/
#[derive(Debug, Clone, PartialEq)]
pub enum MessageKind {
    Connect = 001,
    Login = 002,
    Success = 003,
    Error = 004,
    Mkdir = 010,
    Cd = 020,
    Ls = 030,
    Up = 100,
    Down = 200,
    File = 255,
}

// what a hacky terrible thing
impl MessageKind {
    pub fn from_u8(value: u8) -> MessageKind {
        match value {
            001 => MessageKind::Connect,
            002 => MessageKind::Login,
            003 => MessageKind::Success,
            004 => MessageKind::Error,
            010 => MessageKind::Mkdir,
            020 => MessageKind::Cd,
            030 => MessageKind::Ls,
            100 => MessageKind::Up,
            200 => MessageKind::Down,
            255 => MessageKind::File,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

#[derive(Debug)]
pub struct MessageSender {
    pub command: MessageKind,
    pub arguments: String,
    pub file_path: Option<PathBuf>,
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
        }
    }

    // Blocking function!!!
    pub fn send_message(self, mut writer: &TcpStream) -> io::Result<()> {
        // println!("Sent message called once");
        // Generate message and send headers
        let (headers, payload) = self.generate_message()?;
        writer.write_all(&headers)?;

        // Send payload if any
        match payload {
            Some(mut file) => {
                let mut length = 1;
                while length > 0 {
                    let buffer = file.fill_buf()?;
                    // println!("File to be sent: {:?}",buffer);
                    length = buffer.len();
                    writer.write_all(&buffer)?;
                    file.consume(length);
                }
            }
            None => {}
        }
        writer.flush()?;
        return Ok(());
    }

    // idk how to chain the vector and bufreader into a single iterator bro
    fn generate_message(&self) -> io::Result<(Vec<u8>, Option<BufReader<File>>)> {
        let mut payload_length: u64 = 0;
        let mut reader = None;
        match &self.file_path {
            Some(file_path) => {
                let file = File::open(file_path)?;
                payload_length = file.metadata()?.len();
                reader = Some(BufReader::with_capacity(BUFFER_SIZE,file));
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

#[derive(Debug)]
pub struct MessageReceiver {
    pub command: MessageKind,
    pub arguments: String,
    pub payload_size: u64,
}

impl MessageReceiver {
    // blocks until it receives message headers and forms itself
    pub fn new(mut tcpstream: &TcpStream) -> io::Result<Self> {
        // Read in 10 byte header
        let mut headers: [u8; HEADER_SIZE] = [0; HEADER_SIZE];
        tcpstream.read_exact(&mut headers)?;

        // Split header into the 3 components
        let mut payload_size = u64::from_be_bytes(headers[0..8].try_into().unwrap());
        let command: MessageKind = MessageKind::from_u8(headers[8]);
        let argument_size = u32::from_be_bytes(headers[9..HEADER_SIZE].try_into().unwrap());

        // Read in arguments
        let mut argument_bytes: Vec<u8> = vec![0u8; argument_size as usize];
        tcpstream.read_exact(&mut argument_bytes)?;
        let argument_string = from_utf8(&argument_bytes).unwrap().to_string();
        payload_size -= HEADER_SIZE as u64 + argument_size as u64;

        // println!("message size: {}", payload_size);
        // println!("command: {}", headers[8]);
        // println!("arguments size: {}", argument_size);
        // println!("arguments: {}", argument_string);
        // println!("payload size: {}", payload_size);

        // Construct self
        let message_receiver: MessageReceiver = Self {
            command: command,
            arguments: argument_string,
            payload_size: payload_size,
        };

        Ok(message_receiver)
    }

    // Writes to a file_path
    pub fn write_to(
        self,
        tcpstream: &TcpStream,
        file_path: PathBuf,
    ) -> io::Result<()> {
        let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);
        let mut byte_count: u64 = 0;
        let mut reader = BufReader::with_capacity(BUFFER_SIZE, tcpstream );
        let capacity = reader.capacity() as u64;
        while byte_count < self.payload_size {
            // println!("Waiting for payload..");
            if self.payload_size - byte_count < capacity {
                let remaining_bytes = self.payload_size - byte_count;
                let mut buffer = vec![0u8; (remaining_bytes).try_into().unwrap()];
                reader.read_exact(&mut buffer)?;
                // println!("Received bytes: {:?}",buffer);
                // println!("About to write");
                writer.write(&buffer)?;
                writer.flush()?;
                // println!("Done writing");
                return Ok(());
            } else {
                let buffer = reader.fill_buf()?;
                writer.write(buffer)?;
                let length = buffer.len();
                reader.consume(length);
                byte_count += length as u64;
                // print!("{}\t", byte_count);
            };
        }
        writer.flush()?;
        return Ok(());
    }

    /* pub fn get_reader(self) -> BufReader<&'a TcpStream> {
        return self.payload;
    } */
}

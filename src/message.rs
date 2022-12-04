use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, BufRead};
use std::str::from_utf8;

// Refactor this rubbish with proper error handling, use custom types instead of io 
// https://www.sheshbabu.com/posts/rust-error-handling/
#[derive(Debug, Clone) ]
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
}

// what a hacky terrible thing
impl MessageKind {
    pub fn from_u8(value: u8) -> MessageKind {
        match value {
            001 => MessageKind::Connect ,
            002 => MessageKind::Login , 
            003 => MessageKind::Success,
            004 => MessageKind::Error,
            010 => MessageKind::Mkdir ,
            020 => MessageKind::Cd ,
            030 => MessageKind::Ls ,
            100 => MessageKind::Up,
            200 => MessageKind::Down,
            _ => panic!("Unknown value: {}", value),
        }
    }
}

#[derive(Debug)]
pub struct MessageSender <'a>{
    pub command:MessageKind,
    pub command_string: String,
    pub file_path: Option<PathBuf>,
    pub writer: BufWriter<&'a TcpStream>
}

impl <'a> MessageSender <'a>{

    // generator
    pub fn new(command: MessageKind, command_string: String, file_path: Option<PathBuf>, tcpstream: &'a TcpStream) -> Self {
        Self {command, command_string, file_path, writer: BufWriter::new(&tcpstream)}
    } 

    // blocking function!!!
    pub fn send_message(mut self) -> io::Result<()>{
        let (headers, payload) = self.generate_message()?;
        self.writer.write_all(&headers)?;
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
        self.writer.flush();
        return Ok(());
    }
    
    // idk how to chain the vector and bufreader into a single iterator bro
    fn generate_message(&self) -> io::Result<(Vec<u8>, Option<BufReader<File>> )> {
        let mut payload_length: u64 = 0;
        let mut reader = None;
        match &self.file_path {
            Some(file_path) => {
                let file = File::open(file_path)?;
                payload_length = file.metadata()?.len();
                reader = Some(BufReader::new(file));
            }
            None => {
            }
        }
        let command_string = self.command_string.as_bytes();
        let string_length: u64 = command_string.len().try_into().unwrap();
        if string_length > 255 {
            return Err(io::Error::new(io::ErrorKind::Other, "Command string too long"));
        }

        let size: u64 = 10 + string_length + payload_length; 
        let mut headers: Vec<u8> = vec![];
        headers.extend(size.to_be_bytes());
        headers.push(self.command.clone() as u8);
        headers.push(string_length as u8);
        headers.extend_from_slice(command_string);
        Ok((headers, reader))
    }

}

#[derive (Debug)]
pub struct MessageReceiver <'a>{
    pub command: MessageKind,
    pub command_string: String,
    pub payload: BufReader<&'a TcpStream>,
    pub payload_size: u64,
}

impl <'a> MessageReceiver <'a>{
    
    // blocks until it receives message headers and forms itself
    pub fn new(mut tcpstream: BufReader<&'a TcpStream>)-> io::Result<Self>{
        let mut headers: [u8; 10] = [0; 10];
        tcpstream.read_exact(&mut headers)?;
        let mut payload_size = u64::from_be_bytes(headers[0..8].try_into().unwrap());    
        let command: MessageKind = MessageKind::from_u8(headers[8]);
        let string_size: u8 = headers[9];
        let mut command_string: Vec<u8> = vec![0u8,string_size];
        tcpstream.read_exact(&mut command_string)?;
        let command_string = from_utf8(&command_string).unwrap().to_string(); 
        payload_size -= 10 + string_size as u64;
        Ok(Self {command, command_string, payload:tcpstream, payload_size})
    }

    // writes to a file_path
    pub fn write_to(mut self, file_path: PathBuf)-> io::Result<()>{
        let file = File::create(file_path).unwrap();
        let mut writer = BufWriter::new(file); 
        let mut byte_count: u64 = 0;
        while byte_count != self.payload_size {
            let buffer = self.payload.fill_buf()?;
            writer.write(buffer)?;
            let length = buffer.len();
            self.payload.consume(length);
            byte_count += length as u64;
        }
        writer.flush()?;
        return Ok(())
    }

    pub fn get_reader(self) -> BufReader<&'a TcpStream> {
        return self.payload;
    }
}



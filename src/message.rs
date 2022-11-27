use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, BufRead};
use std::str::from_utf8;

// Refactor this rubbish with proper error handling, use custom types instead of io 
// https://www.sheshbabu.com/posts/rust-error-handling/
#[derive(Debug) ]
pub enum MessageKind {
    Connect = 001,
    Login = 002,
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
pub struct MessageSender{
    pub command:MessageKind,
    pub command_string: String,
    pub file_path: Option<PathBuf>,
}

impl MessageSender{

    pub fn new(command: MessageKind, command_string: String, file_path: Option<PathBuf>) -> Self {
        Self {command, command_string, file_path}
    } 
    // idk how to chain the vector and bufreader into a single iterator bro
    pub fn generate_message(self) -> io::Result<(Vec<u8>, Option<BufReader<File>> )> {
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
        headers.push(self.command as u8);
        headers.push(string_length as u8);
        headers.extend_from_slice(command_string);
        Ok((headers, reader))
    }

}

#[derive (Debug)]
pub struct MessageReciever {
    pub command: MessageKind,
    pub command_string: String,
    pub payload: BufReader<TcpStream>,
    pub payload_size: u64,
}

impl MessageReciever {
    pub fn new(mut payload: BufReader<TcpStream>)-> io::Result<Self>{
        let mut headers = [0u8, 10];
        payload.read_exact(&mut headers)?;
        let mut payload_size = u64::from_be_bytes(headers[0..8].try_into().unwrap());    
        let command: MessageKind = MessageKind::from_u8(headers[9]);
        let string_size: u8 = headers[10];
        let mut command_string: Vec<u8> = vec![0u8,string_size];
        payload.read_exact(&mut command_string)?;
        let command_string = from_utf8(&command_string).unwrap().to_string(); 
        payload_size -= 10 + string_size as u64;
        Ok(Self {command, command_string, payload, payload_size})
    }

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

    pub fn get_reader(self) -> BufReader<TcpStream> {
        return self.payload;
    }
}



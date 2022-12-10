
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::num;
use std::path::PathBuf;
use std::str::from_utf8;

use crate::client::utilities::print_progress;

use crate::message::{MessageKind, BUFFER_SIZE, HEADER_SIZE};


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
            print_progress(byte_count,self.payload_size);
            if self.payload_size - byte_count < capacity {
                let remaining_bytes = self.payload_size - byte_count;
                let mut buffer = vec![0u8; (remaining_bytes).try_into().unwrap()];
                reader.read_exact(&mut buffer)?;
                // println!("Received bytes: {:?}",buffer);
                // println!("About to write");
                writer.write(&buffer)?;
                writer.flush()?;
                print_progress(self.payload_size,self.payload_size);
                print!("\n");
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
        print_progress(self.payload_size,self.payload_size);
        print!("\n");
        writer.flush()?;
        return Ok(());
    }

    /* pub fn get_reader(self) -> BufReader<&'a TcpStream> {
        return self.payload;
    } */
}


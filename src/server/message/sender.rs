use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::str::from_utf8;
use std::sync::{Arc, RwLockReadGuard};

use crate::message::{MessageKind, BUFFER_SIZE, HEADER_SIZE};
use crate::server::fsrw_mutex::*;

// DO NOT RELY ON MESSAGE SENDER TO VALIDATE FILEPATHS. ALL FILEPATHS ARE ASSUMED TO BE VALID.

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
    pub fn send_message(self, mut writer: &TcpStream, fsrw_mutex: &FsrwMutex) -> io::Result<()> {
        // println!("Sent message called once");
        // Generate message and send headers
        let headers = self.generate_headers()?;
        writer.write_all(&headers)?;

        // Send payload if any
        match &self.file_path {
            Some(file_path) => {
                // Lock file_dict
                let file_dict = match fsrw_mutex.file_dict.lock() {
                    Ok(guard) => guard,
                    // Need to handle this properly
                    Err(poisoned) => {
                        panic!("file_dict poisoned: {}", poisoned)
                    }
                };
                let file_lock = acquire_file_rwlock(file_dict, file_path.to_path_buf());
                // file_dict is automatically unlocked when acquire_file_rwlock consumes it

                // Lock rwlock as a reader
                let file = match file_lock.read() {
                    Ok(guard) => guard,
                    // Need to handle this properly
                    Err(poisoned) => {
                        panic!("file_dict poisoned: {}", poisoned)
                    }
                };

                // Send here
                let send_result = critical_region_send(file, writer);
                // critical_region_send drops the rwlock to the file but we also need to release the atomic reference counter file_lock regardless of write result
                drop(file_lock);

                // Update file_dict that file rwlock was unlocked
                // Lock file dict
                let file_dict = match fsrw_mutex.file_dict.lock() {
                    Ok(guard) => guard,
                    // Need to handle this properly
                    Err(poisoned) => {
                        panic!("file_dict poisoned: {}", poisoned)
                    }
                };
                release_file_rwlock(file_dict, file_path.to_path_buf());

                return send_result;
            }
            None => {}
        }
        writer.flush()?;
        return Ok(());
    }

    fn generate_headers(&self) -> io::Result<Vec<u8>> {
        let mut payload_length: u64 = 0;
        match &self.file_path {
            Some(file_path) => {
                let file = File::open(file_path)?;
                payload_length = file.metadata()?.len();
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
        Ok(headers)
    }
}

fn critical_region_send(file: RwLockReadGuard<File>, mut writer: &TcpStream) -> io::Result<()> {
    let mut file_reader = BufReader::with_capacity(BUFFER_SIZE, &*file);

    // Send file
    let mut length = 1;
    while length > 0 {
        let buffer = file_reader.fill_buf()?;
        // println!("File to be sent: {:?}",buffer);
        length = buffer.len();
        writer.write_all(&buffer)?;
        file_reader.consume(length);
    }
    return Ok(());
}

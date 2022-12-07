use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, ErrorKind};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::str::from_utf8;
use std::sync::RwLockWriteGuard;

use crate::message::{MessageKind, BUFFER_SIZE, HEADER_SIZE};
use crate::server::fsrw_mutex::*;

#[derive(Debug)]
pub struct MessageReceiver {
    pub command: MessageKind,
    pub arguments: String,
    pub payload_size: u64,
}

// DO NOT RELY ON MESSAGE RECEIVER TO VALIDATE FILEPATHS. ALL FILEPATHS ARE ASSUMED TO BE VALID.
// Note the use of big endian
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

    // Writes the message payload to a file_path. Assumes that file_path is valid!
    pub fn write_to(
        self,
        tcpstream: &TcpStream,
        file_path: PathBuf,
        fsrw_mutex: &FsrwMutex,
    ) -> io::Result<()> {
        // Acquire write access to the file
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

        // Lock rwlock as a writer
        let mut writer = match file_lock.write() {
            Ok(guard) => guard,
            // Need to handle this properly
            Err(poisoned) => {
                panic!("file_dict poisoned: {}", poisoned)
            }
        };

        // Write here
        let write_result = critical_region_write(self.payload_size, writer, &tcpstream);

        // Critical_region_write drops the rwlock to the file but we also need to release the atomic reference counter file_lock regardless of write result
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

        return write_result;
    }
}

// This code holds the critical region (where rwlock<File> is held) for write so failing here can be handled by the caller safely
fn critical_region_write(
    payload_size: u64,
    mut writer: RwLockWriteGuard<File>,
    tcpstream: &TcpStream,
) -> io::Result<()> {
    let mut byte_count: u64 = 0;
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, tcpstream);
    let capacity = reader.capacity() as u64;
    while byte_count < payload_size {
        // println!("Waiting for payload..");
        if payload_size - byte_count < capacity {
            let remaining_bytes = payload_size - byte_count;
            let mut buffer = vec![0u8; (remaining_bytes).try_into().unwrap()];
            reader.read_exact(&mut buffer)?;
            // println!("Received bytes: {:?}",buffer);
            // println!("About to write");
            writer.write(&buffer)?;
            byte_count += remaining_bytes;
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
    // println!("Done writing");

    return Ok(());
}

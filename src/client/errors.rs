use std::fmt;

#[derive (Debug)]
pub enum ClientError {
    InvalidCommand,
    InvalidAddress(String),
    ConnectionError,
    WrongArgumentNum(String),
    IOError(String),
    MessageError,
    DownloadError(String),
    WriteError(String),
    UploadError(String),
    FileError(String),
    DestinationError(String),
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            Self::ConnectionError => f.write_str("Error: Connection has not been successfully established."),
            Self::InvalidCommand => f.write_str("Error: Command was invalid. Type 'help' for a list of commands."),
            Self::InvalidAddress(help) => f.write_str(&format!("Error: Socket address is invalid. \n {}", help)),
            Self::WrongArgumentNum(help) => f.write_str(&format!("Error: Wrong number of arguments passed. \n {}", help)),
            Self::IOError(error) => f.write_str(&format!("Error: There was an error processing the command. Please try again! \n {}", error)),
            Self::MessageError => f.write_str("Error: No valid message was receieved from server."),
            Self::DownloadError(error) => f.write_str(&format!("Error: {}", error)),
            Self::WriteError(error) => f.write_str(&format!("Error: There was an issue the file to the local machine. \n {}", error)),
            Self::DestinationError(error) => f.write_str(&format!("Invalid path: {}", error)),
            Self::UploadError(error) => f.write_str(&format!("Error: {}", error)),
            Self::FileError(file) => f.write_str(&format!("Error: Cannot access {}: no such file", file))
        }
    }
}

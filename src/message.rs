
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
            _ => panic!("Unable to parse messagekind - Unknown value: {}", value),
        }
    }
}

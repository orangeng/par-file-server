use regex::Regex;
use std::env;
use std::{io::{self, Write, Error, Read}, process::exit, net::TcpStream};
use crate::message;

pub struct Connection{
  pub stream: TcpStream,
  pub addr: String
}

impl Connection {
    pub fn process_command(&mut self, tokens: &Vec<&str>) {
    }
}
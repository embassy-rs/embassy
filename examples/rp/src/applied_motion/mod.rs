/// Module to interact with Applied Motion Step/Servo motors.
///
/// Reference: https://appliedmotion.s3.amazonaws.com/Host-Command-Reference_920-0002W_0.pdf
use heapless::String;
use serde::{Deserialize, Serialize};

pub mod command;

/// Default baud rate for serial communication with Applied Motion devices.
pub const DEFAULT_BAUD_RATE: usize = 38_400; // Example, adjust as needed
const RX_BUFFER_SIZE: usize = 64;

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Message {
    pub address: Option<u8>, // Address is optional, only needed for RS-485.
    pub command: command::Command,
    pub param1: Option<String<16>>, // Parameter A
    pub param2: Option<String<16>>, // Parameter B
}

impl Message {
    pub fn new(
        address: Option<u8>,
        command: command::Command,
        param1: Option<String<16>>,
        param2: Option<String<16>>,
    ) -> Self {
        Self {
            address,
            command,
            param1,
            param2,
        }
    }
}

pub trait ToCommandStr {
    fn construct(&self) -> String<64>;
}

impl ToCommandStr for Message {
    fn construct(&self) -> String<64> {
        let mut buffer = String::<64>::new();
        if let Some(addr) = self.address {
            write!(buffer, "{}", addr).unwrap(); // Address for RS-485
        }
        write!(buffer, "{}", self.command.construct()).unwrap();

        if let Some(param1) = &self.param1 {
            write!(buffer, "{}", param1).unwrap();
        }
        if let Some(param2) = &self.param2 {
            write!(buffer, "{}", param2).unwrap();
        }

        write!(buffer, "\r").unwrap(); // End with carriage return
        buffer
    }
}

/// Enumerates potential errors that can occur while processing Applied Motion commands and responses.
#[derive(Debug, PartialEq)]
pub enum Error {
    ParsingError,
    WriteError,
    UnknownUart,
    InvalidIndex(u8),
    NoResponse,
    Nack(String<16>), // Store the error code or message received with a NACK
}

/// Represents an Applied Motion Device.
#[derive(Debug)]
pub struct AppliedMotion {
    pub address: Option<u8>,
    pub rx_buffer: String<RX_BUFFER_SIZE>,
    pub pending: Option<command::Command>,
    pub active: bool,
    pub delay_ms: u64,
}

impl AppliedMotion {
    pub fn new(address: Option<u8>) -> Self {
        Self {
            address,
            rx_buffer: String::<RX_BUFFER_SIZE>::new(),
            pending: None,
            active: false,
            delay_ms: 10,
        }
    }

    pub fn read_into(&mut self) -> &mut [u8] {
        unsafe { self.rx_buffer.as_mut_vec().as_mut() }
    }

    pub fn parse(&mut self) -> Result<Option<Response>, Error> {
        if self.rx_buffer.ends_with('\r') {
            let response = self.rx_buffer.trim();

            // Attempt to parse the response in the format "YXX=A"
            let mut parts = response.split('=');
            let command_part = parts.next();
            let data_part = parts.next();

            match (command_part, data_part) {
                (Some(cmd_part), Some(data)) => {
                    // Check if the command part starts with an address (digit)
                    let (address, command) = if let Some(first_char) = cmd_part.chars().next() {
                        if first_char.is_digit(10) {
                            // Extract the address if it's a digit (RS-485 case)
                            let addr = first_char.to_digit(10).unwrap() as u8;
                            (Some(addr), &cmd_part[1..])
                        } else {
                            // No address, use the full string as the command
                            (None, cmd_part)
                        }
                    } else {
                        (None, "")
                    };

                    // Ensure command has the right format (two uppercase letters)
                    if command.len() == 2 && command.chars().all(|c| c.is_ascii_uppercase()) {
                        let mut cmd_buffer = String::<4>::new();
                        cmd_buffer.push_str(command).unwrap();

                        let mut data_buffer = String::<16>::new();
                        data_buffer.push_str(data).unwrap();

                        let parsed_response = Response {
                            address,
                            command: cmd_buffer,
                            data: data_buffer,
                        };

                        self.rx_buffer.clear();
                        Ok(Some(parsed_response))
                    } else {
                        Err(Error::ParsingError)
                    }
                }
                _ => Err(Error::ParsingError),
            }
        } else {
            Ok(None) // Not enough data yet
        }
    }

    pub fn push_data_and_parse<C>(&mut self, chars: C) -> Result<Option<String<16>>, Error>
    where
        C: core::iter::IntoIterator<Item = char>,
    {
        for c in chars {
            match self.rx_buffer.push(c) {
                Ok(_) => {
                    if c == '\r' {
                        return self.parse();
                    }
                }
                Err(_) => {
                    self.rx_buffer.clear();
                }
            }
        }
        Ok(None)
    }
}

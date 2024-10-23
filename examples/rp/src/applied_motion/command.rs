use heapless::String;
use serde::{Deserialize, Serialize};

pub(crate) const CR: char = '\r';

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Command {
    AccelRate(f32),  // Sets or requests the acceleration rate in rev/sec².
    MaxAccel(f32),   // Sets or requests the maximum acceleration/deceleration in rev/sec².
}

impl Command {
    pub fn construct(&self) -> String<64> {
        let mut buffer = String::<64>::new();
        match self {
            Command::AccelRate(rate) => {
                write!(buffer, "AC{}", rate).unwrap();
            }
            Command::MaxAccel(max_rate) => {
                write!(buffer, "AM{}", max_rate).unwrap();
            }
        }
        write!(buffer, "{}", CR).unwrap(); // Append carriage return at the end.
        buffer
    }
}
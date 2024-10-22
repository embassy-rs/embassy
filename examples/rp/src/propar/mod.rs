//! Bronkhorst Mass Flow Meter FLOW-BUS Driver
//!
//! This driver enables interfacing with Bronkhorst Coriolis
//! mass flow meters over a serial interface, such as RS232
//! or RS485.
//! [FLOW-BUS](https://www.bronkhorst.com/en-us/downloads-en/manuals-and-quick-installation-guides/manual-qig-interfaces-software/)
//!

use heapless::Vec;

pub mod packet;

pub const START_BYTEACTER: char = ':';
pub const CARRIAGE_RETURN: char = '\r';
pub const LINE_FEED: char = '\n';

pub const UNIVERSAL_NODE_ADDRESS: u8 = 128;
pub const DEFAULT_NODE_ADDRESS: u8 = 3;

pub const BRONK_CHAINED_MASK: u8 = 0x80;
pub const BRONK_TYPE_MASK: u8 = 0x60;
pub const BRONK_PARAM_NUMBER_MASK: u8 = 0x1F;

pub const PAYLOAD_DATA_SIZE: usize = 8;
pub type PayloadData = Vec<Option<packet::PacketData>, PAYLOAD_DATA_SIZE>;

#[derive(Debug)]
pub struct Payload {
    pub packet: packet::Packet,
    pub data: PayloadData,
}

impl Payload {
    pub fn new() -> Self {
        Self {
            packet: packet::Packet::default(),
            data: Vec::from_slice(&[None]).unwrap(),
        }
    }

    fn len(&self) -> usize {
        self.data.iter().filter(|p| p.is_some()).count()
    }

    pub fn from_buffer(buffer: &mut [u8]) -> Result<Self, ()> {
        let mut payload = Self::new();

        let mut offset: usize = 0;

        // send the [0] byte to the trash
        match buffer[offset] as char {
            START_BYTEACTER => offset += 1,
            _ => return Err(()),
        };

        // send the [1] byte to the packet_length
        payload.packet.len = buffer[offset];
        offset += 1;

        // send the [2] byte to the node entry
        payload.packet.node = buffer[offset];
        offset += 1;

        // send the [3] byte to the command entry
        payload.packet.command = buffer[offset].into();
        offset += 1;

        // handle status response to a write w/status request
        if payload.packet.command == packet::CommandType::Status {
            // send the [4] byte to the status
            let s = packet::Status::from(buffer[offset]);
            payload.packet.status = Some(s);
            offset += 1;
        }

        // handle write response to a read request
        if payload.packet.command == packet::CommandType::WriteWithoutStatusAnswer
            || payload.packet.command == packet::CommandType::WriteWithSourceAddress
        {
            let mut chained = packet::Chained::Not;
            let mut index: usize = 0;

            while offset < (payload.packet.len + 1) as usize {
                let mut packet_data = packet::PacketData::default();

                // handle the process byte
                #[allow(unused_assignments)]
                if chained == packet::Chained::Not {
                    packet_data.process_data.chained = (buffer[offset] & BRONK_CHAINED_MASK).into();
                    packet_data.process_data.process_number = buffer[offset] & !BRONK_CHAINED_MASK;
                    chained = packet_data.process_data.chained;
                    offset += 1;
                }

                // handle the parameter byte
                packet_data.parameter_data.chained = (buffer[offset] & BRONK_CHAINED_MASK).into();
                packet_data.parameter_data.parameter_type =
                    (buffer[offset] & BRONK_TYPE_MASK).into();
                packet_data.parameter_data.parameter_number =
                    buffer[offset] & BRONK_PARAM_NUMBER_MASK;
                chained = packet_data.parameter_data.chained;

                offset += 1;

                let (start, end) = match &packet_data.parameter_data.parameter_type {
                    packet::ParamType::StringP => {
                        // first byte is string length
                        packet_data.value[0] = buffer[offset];
                        if packet_data.value[0] == 0 {
                            packet_data.value[0] = packet::PACKET_DATA_VALUE_LEN - 1;
                        }
                        offset += 1;
                        (
                            1_usize,
                            packet_data.value[0].clamp(1, packet::PACKET_DATA_VALUE_LEN - 1)
                                as usize,
                        )
                    }
                    p => (0_usize, p.to_length()),
                };

                for i in start..end {
                    match buffer.get(offset) {
                        Some(c) => {
                            packet_data.value[i] = *c;
                            if packet_data.value[i] == 0x00
                                && packet_data.parameter_data.parameter_type
                                    == packet::ParamType::StringP
                            {
                                offset += 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                    offset += 1;
                }

                payload.data[index] = Some(packet_data);
                index += 1;
            }
        }

        Ok(payload)
    }

    pub fn write_buffer(&mut self, buffer: &mut [u8]) -> Result<usize, ()> {
        let mut offset: usize = 0;

        // set the [0] byte in the tx_buffer to the START_BYTEACTER
        buffer[offset] = START_BYTEACTER as u8;
        offset += 1;

        // set the [1] byte of the packet to 0 as a placeholder for the message length
        buffer[offset] = 0;
        offset += 1;

        // set the [2] byte of the packet to the Node address
        buffer[offset] = self.packet.node;
        offset += 1;

        // set the [3] byte of the packet to the command type
        buffer[offset] = self.packet.command.into();
        offset += 1;

        // append the parameters
        for p in self.data.iter() {
            match p {
                Some(p) => {
                    // set the [4 + p * 2] byte of the packet to the command type
                    buffer[offset] = p.process_data.chained as u8 | p.process_data.process_number;
                    offset += 1;

                    buffer[offset] = p.parameter_data.chained as u8
                        | p.parameter_data.parameter_number
                        | p.parameter_data.parameter_type as u8;
                    offset += 1;

                    if p.len > 0 {
                        let len = match &p.parameter_data.parameter_type {
                            packet::ParamType::StringP => p.value[0] as usize,
                            p => p.to_length(),
                        };
                        buffer[offset..offset + len].copy_from_slice(&p.value[0..len]);
                        offset += len;
                    }

                    if self.packet.command == packet::CommandType::RequestParameter
                        && self.len() == 1
                    {
                        buffer[offset] = packet::Chained::Not as u8 | p.process_data.process_number;
                        offset += 1;

                        buffer[offset] = p.parameter_data.chained as u8
                            | p.parameter_data.parameter_number
                            | p.parameter_data.parameter_type as u8;
                        offset += 1;
                    }
                }
                _ => {}
            }
        }

        // now set [1] byte of the packet to to the message length
        buffer[1] = (offset - 2) as u8;

        buffer[offset] = CARRIAGE_RETURN as u8;
        offset += 1;

        buffer[offset] = LINE_FEED as u8;
        offset += 1;

        Ok(offset)
    }
}

pub fn request_measure(node: u8) -> Payload {
    let mut payload = Payload::new();

    payload.packet.node = node;
    payload.packet.command = packet::CommandType::RequestParameter;

    let mut packet_data = packet::PacketData::default();

    packet_data.process_data.chained = packet::Chained::Not;
    packet_data.process_data.process_number = packet::BRONK_MEASURE_P.process_number;
    packet_data.parameter_data.chained = packet::Chained::Not;
    packet_data.parameter_data.parameter_type = packet::BRONK_MEASURE_P.parameter_type;
    packet_data.parameter_data.parameter_number = packet::BRONK_MEASURE_P.fbnr;

    payload.data[0] = Some(packet_data);

    payload
}

pub fn write_setpoint(node: u8, setpoint: u16) -> Payload {
    let mut payload = Payload::new();
    payload.packet.node = node;
    payload.packet.command = packet::CommandType::WriteWithStatusAnswer;

    let mut packet_data = packet::PacketData::default();
    packet_data.process_data.chained = packet::Chained::Not;
    packet_data.process_data.process_number = packet::BRONK_SETPOINT_P.process_number;
    packet_data.parameter_data.chained = packet::Chained::Not;
    packet_data.parameter_data.parameter_type = packet::BRONK_SETPOINT_P.parameter_type;
    packet_data.parameter_data.parameter_number = packet::BRONK_SETPOINT_P.fbnr;

    let data = setpoint.to_be_bytes();
    packet_data.value[0..data.len()].copy_from_slice(&data[0..data.len()]);
    packet_data.len = data.len() as u8;

    payload.data[0] = Some(packet_data);

    payload
}

#[cfg(test)]
mod test {
    use super::{
        packet, request_measure, write_setpoint, Payload, CARRIAGE_RETURN, LINE_FEED,
        START_BYTEACTER,
    };

    const START_U8: u8 = START_BYTEACTER as u8;
    const CR_U8: u8 = CARRIAGE_RETURN as u8;
    const LF_U8: u8 = LINE_FEED as u8;

    #[allow(unused)]
    fn print_packet_values(p: &Payload) {
        for d in &p.data {
            match d {
                Some(d) => {
                    println!("{:?}", d.to_parsed_value());
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_request_measure() {
        let mut p = request_measure(1);
        let mut buffer: [u8; 256] = [0; 256];
        let len = p.write_buffer(&mut buffer).unwrap();
        assert_eq!(&[58, 6, 1, 4, 1, 32, 1, 32, 13, 10], &buffer[0..len]);
    }

    #[test]
    fn test_write_setpoint() {
        let mut p = write_setpoint(1, 600);
        let mut buffer: [u8; 256] = [0; 256];
        let len = p.write_buffer(&mut buffer).unwrap();
        assert_eq!(&[58, 6, 1, 1, 1, 33, 2, 88, 13, 10], &buffer[0..len]);
    }

    #[test]
    fn test_read_measure() {
        let mut buffer = [
            START_U8, 0x06, 0x80, 0x02, 0x01, 0x21, 0x7D, 0x00, CR_U8, LF_U8,
        ];
        let p = Payload::from_buffer(&mut buffer).unwrap();
        assert_eq!(
            p.data.get(0).unwrap().unwrap().to_parsed_value(),
            packet::ParamTypeValue::InetegerP(32000)
        );
    }

    #[test]
    fn test_read_fsetpoint() {
        let mut buffer = [
            START_U8, 0x08, 0x80, 0x02, 0x21, 0x41, 0x45, 0x3B, 0x80, 0x00, CR_U8, LF_U8,
        ];
        let p = Payload::from_buffer(&mut buffer).unwrap();
        assert_eq!(
            p.data.get(0).unwrap().unwrap().to_parsed_value(),
            packet::ParamTypeValue::FloatOrLongP(1161527296)
        );
    }

    #[test]
    fn test_read_fluidname() {
        let mut buffer = [
            START_U8, 0x0F, 0x80, 0x02, 0x01, 0x71, 0x0A, 0x41, 0x69, 0x52, 0x20, 0x20, 0x20, 0x20,
            0x20, 0x20, 0x20, CR_U8, LF_U8,
        ];
        let p = Payload::from_buffer(&mut buffer).unwrap();
        assert_eq!(
            p.data.get(0).unwrap().unwrap().to_parsed_value(),
            packet::ParamTypeValue::StringP([
                'A', 'i', 'R', ' ', ' ', ' ', ' ', ' ', ' ', '\0', '\0', '\0', '\0', '\0', '\0',
                '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0'
            ])
        );
    }

    #[test]
    fn test_read_bhtmodel_number() {
        let mut buffer = [
            START_U8, 0x1A, 0x03, 0x02, 0x71, 0x62, 0x00, 0x46, 0x2D, 0x32, 0x30, 0x31, 0x43, 0x56,
            0x2D, 0x35, 0x4B, 0x30, 0x2D, 0x41, 0x41, 0x44, 0x2D, 0x33, 0x33, 0x2D, 0x56, 0x00,
            CR_U8, LF_U8,
        ];
        let p = Payload::from_buffer(&mut buffer).unwrap();
        assert_eq!(
            p.data.get(0).unwrap().unwrap().to_parsed_value(),
            packet::ParamTypeValue::StringP([
                'F', '-', '2', '0', '1', 'C', 'V', '-', '5', 'K', '0', '-', 'A', 'A', 'D', '-',
                '3', '3', '-', 'V', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0', '\0'
            ])
        );
    }
}

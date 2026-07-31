pub const HEADER_LEN: usize = 5;
pub const CMD_IN: u8 = b'I';
pub const CMD_OUT: u8 = b'O';

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    In(u32),
    OutStarted(u32),
    OutComplete(u32),
    Unknown(u8),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Feed {
    pub consumed: usize,
    pub event: Option<Event>,
}

pub struct Parser {
    header: [u8; HEADER_LEN],
    header_len: usize,
    out_remaining: u32,
    out_total: u32,
}

impl Parser {
    pub const fn new() -> Self {
        Self {
            header: [0; HEADER_LEN],
            header_len: 0,
            out_remaining: 0,
            out_total: 0,
        }
    }

    pub fn feed(&mut self, input: &[u8]) -> Feed {
        if self.out_remaining != 0 {
            let consumed = input.len().min(self.out_remaining as usize);
            self.out_remaining -= consumed as u32;
            let event = if self.out_remaining == 0 {
                let total = self.out_total;
                self.out_total = 0;
                Some(Event::OutComplete(total))
            } else {
                None
            };
            return Feed { consumed, event };
        }

        let consumed = input.len().min(HEADER_LEN - self.header_len);
        let end = self.header_len + consumed;
        self.header[self.header_len..end].copy_from_slice(&input[..consumed]);
        self.header_len = end;

        if self.header_len != HEADER_LEN {
            return Feed { consumed, event: None };
        }

        let command = self.header[0];
        let count = u32::from_le_bytes([self.header[1], self.header[2], self.header[3], self.header[4]]);
        self.header_len = 0;

        let event = match command {
            CMD_IN => Some(Event::In(count)),
            CMD_OUT if count == 0 => Some(Event::OutComplete(0)),
            CMD_OUT => {
                self.out_remaining = count;
                self.out_total = count;
                Some(Event::OutStarted(count))
            }
            _ => Some(Event::Unknown(command)),
        };

        Feed { consumed, event }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(command: u8, count: u32) -> [u8; HEADER_LEN] {
        let count = count.to_le_bytes();
        [command, count[0], count[1], count[2], count[3]]
    }

    #[test]
    fn header_accepts_every_split_boundary() {
        let input = frame(CMD_IN, 257);

        for split_at in 1..HEADER_LEN {
            let mut parser = Parser::new();
            assert_eq!(
                parser.feed(&input[..split_at]),
                Feed {
                    consumed: split_at,
                    event: None,
                }
            );
            assert_eq!(
                parser.feed(&input[split_at..]),
                Feed {
                    consumed: HEADER_LEN - split_at,
                    event: Some(Event::In(257)),
                }
            );
        }
    }

    #[test]
    fn header_accepts_five_single_byte_slices() {
        let input = frame(CMD_IN, 17);
        let mut parser = Parser::new();

        for byte in &input[..HEADER_LEN - 1] {
            assert_eq!(
                parser.feed(core::slice::from_ref(byte)),
                Feed {
                    consumed: 1,
                    event: None,
                }
            );
        }
        assert_eq!(
            parser.feed(&input[HEADER_LEN - 1..]),
            Feed {
                consumed: 1,
                event: Some(Event::In(17)),
            }
        );
    }

    #[test]
    fn header_and_out_payload_share_one_slice() {
        let input = [CMD_OUT, 3, 0, 0, 0, 0, 1, 2];
        let mut parser = Parser::new();

        let started = parser.feed(&input);
        assert_eq!(
            started,
            Feed {
                consumed: HEADER_LEN,
                event: Some(Event::OutStarted(3)),
            }
        );
        assert_eq!(
            parser.feed(&input[started.consumed..]),
            Feed {
                consumed: 3,
                event: Some(Event::OutComplete(3)),
            }
        );
    }

    #[test]
    fn payload_completion_leaves_next_header_in_slice() {
        let mut parser = Parser::new();
        let out = frame(CMD_OUT, 2);
        assert_eq!(parser.feed(&out).event, Some(Event::OutStarted(2)));

        let input = [9, 10, CMD_IN, 17, 0, 0, 0];
        let complete = parser.feed(&input);
        assert_eq!(
            complete,
            Feed {
                consumed: 2,
                event: Some(Event::OutComplete(2)),
            }
        );
        assert_eq!(
            parser.feed(&input[complete.consumed..]),
            Feed {
                consumed: HEADER_LEN,
                event: Some(Event::In(17)),
            }
        );
    }

    #[test]
    fn zero_length_out_completes_with_its_header() {
        let mut parser = Parser::new();

        assert_eq!(
            parser.feed(&frame(CMD_OUT, 0)),
            Feed {
                consumed: HEADER_LEN,
                event: Some(Event::OutComplete(0)),
            }
        );
    }

    #[test]
    fn unknown_command_consumes_frame_and_recovers() {
        let input = [b'X', 0, 0, 0, 0, CMD_IN, 17, 0, 0, 0];
        let mut parser = Parser::new();

        let unknown = parser.feed(&input);
        assert_eq!(
            unknown,
            Feed {
                consumed: HEADER_LEN,
                event: Some(Event::Unknown(b'X')),
            }
        );
        assert_eq!(
            parser.feed(&input[unknown.consumed..]),
            Feed {
                consumed: HEADER_LEN,
                event: Some(Event::In(17)),
            }
        );
    }

    #[test]
    fn reset_discards_a_fragmented_header() {
        let mut parser = Parser::new();
        assert_eq!(parser.feed(&[CMD_OUT, 9]).event, None);

        parser = Parser::new();

        assert_eq!(parser.feed(&frame(CMD_IN, 17)).event, Some(Event::In(17)));
    }

    #[test]
    fn maximum_in_count_does_not_wrap() {
        let mut parser = Parser::new();

        assert_eq!(parser.feed(&frame(CMD_IN, u32::MAX)).event, Some(Event::In(u32::MAX)));
    }

    #[test]
    fn maximum_out_count_does_not_wrap() {
        let mut parser = Parser::new();

        assert_eq!(
            parser.feed(&frame(CMD_OUT, u32::MAX)).event,
            Some(Event::OutStarted(u32::MAX))
        );
        assert_eq!(
            parser.feed(&[0; 7]),
            Feed {
                consumed: 7,
                event: None,
            }
        );
    }
}

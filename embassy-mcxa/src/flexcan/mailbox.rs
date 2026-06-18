//! This module models the FlexCAN mailbox for Classic CAN (not FD). 
//! 
//! TX/outgoing messages are handled in the FlexCAN message buffer, which uses the memory area 80h - 27Fh (see page 1545 of the datasheet). This memory area is 512 bytes in total.
//! Each message consists of the CS Register (4 bytes), the Id Register (4 bytes), and the 8-byte message payload. So, each message buffer is 16 bytes in total.
//! This means that the message buffer can hold a total of 512 / 16 = 32 messages.
//! 
//! RX/incoming messages are handeled by the chip's Enhanced RX FIFO (see page 1556 of the datasheet).
//! This FIFO can store 12 messages, which are filled automatically by the hardware as they come in.
//! Messages can be dequeued from this FIFO by reading the 2000h - 2048h memory area as a message buffer, and then setting the erfda flag (to tell the hardware that the memory area is ready to be filled with the next message from the FIFO).

use nxp_pac::can as pac;

/// The "raw" data structure of a FlexCAN message described in the datasheet.
/// For Classic CAN, this is the CS Register (4 bytes), the Id Register (4 bytes), and the 8-byte message payload.
/// This structure is used for both TX and RX messages. As mentioned, TX messages live in the FlexCAN message buffer, while RX messages live in the Enhanced RX FIFO.
struct Message {
    pub cs: pac::Cs,
    pub id: pac::Id,
    pub payload: [u8; 8],
}

/// "Manager" for the message buffer memory area (80h - 27Fh), which this HAL uses for dispatching TX messages.
mod buffer {
    use super::Message;
    use super::pac;

    /// Writes a `Message` into one of the 32 message buffers.
    /// * `can` - The CAN peripheral to write in (e.g., CAN0, CAN1).
    /// * `message` - The Message to write.
    /// * `n` - The message buffer element to write (0 through 31).
    fn write(can: pac::Can, message: &Message, n: usize) {
        // Write in the payload
        let [b0, b1, b2, b3, b4, b5, b6, b7] = message.payload;
        let word0 = u32::from_be_bytes([b0, b1, b2, b3]);
        let word1 = u32::from_be_bytes([b4, b5, b6, b7]);
        can.word0(n).write(|w| { *w = word0 });
        can.word1(n).write(|w| { *w = word1 });

        can.id(n).write(|w| { w.0 = message.id.0 });
        can.cs(n).write(|w| { w.0 = message.cs.0 }); // Need to write in CS last because this is when we update CODE (which could trigger a TX dispatch)
    }

    /// Reads ibe if the 32 message buffers into a `Message`.
    /// * `can` - The CAN peripheral to read from (e.g., CAN0, CAN1).
    /// * `n` - The message buffer element to read (0 through 31).
    fn read(can: pac::Can, n: usize) -> Message {
        let cs = can.cs(n).read();
        let id = can.id(n).read();

        // Read out the payload
        let word0 = can.word0(n).read();
        let word1 = can.word1(n).read();
        let [b0, b1, b2, b3] = word0.to_be_bytes();
        let [b4, b5, b6, b7] = word1.to_be_bytes();
        let payload = [b0, b1, b2, b3, b4, b5, b6, b7];

        Message { cs, id, payload }
    }
}

mod tx {
    use super::Message;

    /// Possible errors from mailbox::tx
    enum TxError {
        /// When trying to read the `CODE` field of a TX message, no known `TxMessageCode` variant matched.
        UnknownCodeReading,
    }

    /// Represents the possible values of the `CODE` field inside a TX message.
    /// See pages 1546 - 1548 of the datasheet.
    enum TxMessageCode {
        /// TX: INACTIVE - Message buffer is not active.
        TxInactive, // Corresponds to CODE = 0b1000

        /// TX: ABORT - Message buffer is aborted.
        TxAbort,    // Corresponds to CODE = 0b1001

        /// TX: DATA - Message buffer is a TX data frame (MB RTR must be 0).
        TxData,     // Corresponds to (CODE == 0b1100) && (RTR bit == 0)

        /// TX: REMOTE - Message buffer is a Transmit Remote Request frame (MB RTR must be 1).
        TxRemote,   // Corresponds to (CODE == 0b1100) && (RTR bit == 1)

        /// TX: TANSWER - Message buffer is a Transmit Response frame from an incoming Remote Request frame.
        TxTanswer,  // Corresponds to CODE == 0b1110
    }

    // These codes couldn't be defined directly in the enum since TxData and TxRemote
    // technically have the same code (they're differentiated via the RTR bit).
    const TX_INACTIVE_CODE: u8 = 0b1000;
    const TX_ABORT_CODE: u8 = 0b1001;
    const TX_DATA_CODE: u8 = 0b1100;
    const TX_REMOTE_CODE: u8 = 0b1100;
    const TX_TANSWER_CODE: u8 = 0b1110;

    struct TxMessage{message: Message}
    impl TxMessage {
        /// Gets the current reading of this message's `CODE` field.
        fn code(&self) -> Result<TxMessageCode, TxError> {
            let code: u8 = self.message.cs.code();
            let rtr: bool = self.message.cs.rtr();
            match (code, rtr) {
                (TX_INACTIVE_CODE, _) =>  Ok(TxMessageCode::TxInactive),
                (TX_ABORT_CODE, _) =>     Ok(TxMessageCode::TxAbort),
                (TX_DATA_CODE, false) =>  Ok(TxMessageCode::TxData),
                (TX_REMOTE_CODE, true) => Ok(TxMessageCode::TxRemote),
                (TX_TANSWER_CODE, _) =>   Ok(TxMessageCode::TxTanswer),
                _ => Err(TxError::UnknownCodeReading)
            }
        }

        /// Sets this message's `CODE` field.
        /// Note: Passing in `TxData` or `TxRemote` may also impact the status of the `RTR` bit, since
        /// TxData requires RTR = 0 and TxRemote requires RTR = 1.
        fn set_code(&mut self, code: TxMessageCode) {
            match code {
                TxMessageCode::TxInactive => self.message.cs.set_code(TX_INACTIVE_CODE),
                TxMessageCode::TxAbort =>    self.message.cs.set_code(TX_ABORT_CODE),
                TxMessageCode::TxData =>   { self.message.cs.set_code(TX_DATA_CODE); self.message.cs.set_rtr(false); }
                TxMessageCode::TxRemote => { self.message.cs.set_code(TX_REMOTE_CODE); self.message.cs.set_rtr(true); }
                TxMessageCode::TxTanswer =>  self.message.cs.set_code(TX_TANSWER_CODE),
            }
        }
    }
}

struct TxMessage{message: Message}
impl TxMessage {

}
use nxp_pac::can2xx as pac;
use embedded_can::Id;

/// Frame Create Errors
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FrameCreateError {
    /// Data in header does not match supplied.
    NotEnoughData,
    /// Invalid data length not 0-8 for Classic packet or valid for FD.
    InvalidDataLength,
    /// Invalid ID.
    InvalidCanId,
}

/// Possible settings inside the CODE field of `pac::Cs`.
/// See pages 1546 - 1548 of the datasheet.
enum CodeRegister {
    /// RX: INACTIVE - Message buffer is not active.
    RxInactive, // Corresponds to CODE == 0b0000

    /// RX: EMPTY - Message buffer is active and empty.
    RxEmpty,    // Corresponds to CODE == 0b0100

    /// RX: FULL - Message buffer is full.
    RxFull,     // Corresponds to CODE == 0b0010

    /// RX: OVERRUN - Message buffer is being overwritten into a full buffer.
    RxOverrun,  // Corresponds to CODE = 0b0110

    /// RX: RANSWER - A frame was configured to recognize a Remote Request frame and transmit a Response frame in return.
    RxRanswer,  // Corresponds to CODE = 0b1010

    /// RX: BUSY - FlexCAN is updating the contents of the message buffer. The CPU must not access the message buffer.
    RxBusy,     // Corresponds to CODE[0] == 1

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
// u_Note: split this into two enums tomorrow.
// ^^ also,  should probably explicitly set the CODE register in ::new_classic(). Maybe have a parameter for whether it's a TX or RX frame? That way any frame that exists is always explicitly either a TX frame or an RX frame

// The first 64 bits of a CAN frame.
// See Table 267 on page 1545 of the MCXAxxx reference manual.
struct Header {
    cs: pac::Cs, // Bits 0 through 31 of the header.
    id: pac::Id, // Bits 32 through 63 of the header.
}

impl Header {
    /// Initializes a `Header` object set for classic CAN (not CAN FD).
    /// * `length` - Length of the frame payload in bytes. A classic CAN frame cannot be more than 8 bytes.
    /// * `extended_id` - Whether or not the frame has a extended ID.
    /// * `rtr` - Remote Transition Request setting. `false` configures this as a normal frame that sends out data, while `true` configures this as a remote frame (to request data from another node).
    const fn new_classic(length: u8, id: Id, rtr: bool) -> Result<Header, FrameCreateError> {
        let mut header = Header {cs: pac::Cs(0), id: pac::Id(0)};

        header.cs.set_edl(false); // EDL = 0 for classic frames
        header.cs.set_rtr(rtr);
        // BRS bit is irrelavent for classic frames
        // ESI bit is irrelavent for classic frames

        // Set DLC Bit (legnth of message in bytes)
        if length > 8 { return Err(FrameCreateError::InvalidDataLength) } // Standard CAN frames cannot be more than 8 bytes.
        header.cs.set_dlc(length);

        // Handle extended vs standard ID
        match id {
            Id::Standard(id) => {
                header.cs.set_ide(false); // standard id: IDE bit = 0 
            }
            Id::Extended(id) => {
                header.cs.set_ide(true); // extended id: IDE bit = 1
                header.cs.set_srr(true); // SRR bit must be 1 for extended frames

            }
        }






        Ok(header)
    }
}

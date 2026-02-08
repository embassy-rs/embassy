//! Implementations for CAN frame accessors
//! Meets the embedded_can Frame trait, and supports
//! conversion to and from hardware representation.
use embedded_can::{ExtendedId, Frame, Id, StandardId};

use crate::can::msgram::{MAX_DATA_LEN, MsgHeader, RxBufferElement, TxBufferElement, TxHeader};

pub struct MCanFrame {
    id: Id,
    dlc: usize,
    is_remote: bool,
    data: [u8; MAX_DATA_LEN], // TODO: CAN-FD will require larger data. This also affects peripheral setup and msgram configuration.
}

#[cfg(feature = "defmt")]
impl defmt::Format for MCanFrame {
    fn format(&self, fmt: defmt::Formatter<'_>) {
        match self.id() {
            embedded_can::Id::Standard(id) => {
                defmt::write!(
                    fmt,
                    "CAN frame: Standard ID={:x} len={}, data: {=[u8]:x}",
                    id.as_raw(),
                    self.dlc,
                    &self.data[0..self.dlc]
                )
            }
            embedded_can::Id::Extended(id) => {
                defmt::write!(
                    fmt,
                    "CAN frame: Extended ID={:x} len={}, data: {=[u8]:x}",
                    id.as_raw(),
                    self.dlc,
                    &self.data[0..self.dlc]
                )
            }
        }
    }
}

impl Frame for MCanFrame {
    fn new(id: impl Into<embedded_can::Id>, data: &[u8]) -> Option<Self> {
        if data.len() > MAX_DATA_LEN {
            return None;
        }

        let mut frame = MCanFrame {
            id: id.into(),
            dlc: data.len(),
            is_remote: false,
            data: [0u8; MAX_DATA_LEN],
        };

        frame.data[..data.len()].clone_from_slice(data);

        Some(frame)
    }
    fn new_remote(id: impl Into<embedded_can::Id>, dlc: usize) -> Option<Self> {
        if dlc > MAX_DATA_LEN {
            return None;
        }

        Some(MCanFrame {
            id: id.into(),
            dlc,
            is_remote: true,
            data: [0u8; MAX_DATA_LEN],
        })
    }
    fn is_extended(&self) -> bool {
        matches!(self.id(), Id::Extended(_))
    }

    fn id(&self) -> embedded_can::Id {
        self.id
    }

    fn dlc(&self) -> usize {
        self.dlc
    }

    fn data(&self) -> &[u8] {
        &self.data[..self.dlc]
    }

    fn is_remote_frame(&self) -> bool {
        self.is_remote
    }
}

impl From<RxBufferElement> for MCanFrame {
    fn from(value: RxBufferElement) -> Self {
        let id = if value.hdr.xtd() {
            // safety - we only read 29 bits of ID, there's no way this can be out of range.
            Id::Extended(unsafe { ExtendedId::new_unchecked(value.hdr.id()) })
        } else {
            let id_shifted = (value.hdr.id() >> 18) as u16;
            // Safety - we only read 29 bits of ID, and we just shifted away 18 of them,
            // leaving only 11 possible non-zero bits.
            Id::Standard(unsafe { StandardId::new_unchecked(id_shifted) })
        };

        // Clamp DLC to valid range in case peripheral returns invalid value
        let dlc = core::cmp::min(value.rxhdr.dlc() as usize, 8);

        MCanFrame {
            id,
            dlc,
            is_remote: value.hdr.rtr(),
            data: value.data,
        }
    }
}

impl MCanFrame {
    pub fn set_id(&mut self, id: impl Into<embedded_can::Id>) {
        self.id = id.into();
    }

    /// Convert this MCanFrame into a TXBufferElement ready for transmission.
    /// If msgid is None, then the EFC field will not be set and no confirmation will
    /// be sent to the event FIFO.
    pub(in crate::can) fn to_tx_buffer(&self, marker: Option<u8>) -> TxBufferElement {
        let mut glblheader = MsgHeader(0);

        glblheader.set_rtr(self.is_remote);
        match self.id {
            Id::Extended(extid) => {
                glblheader.set_xtd(true);
                glblheader.set_id(extid.as_raw());
            }
            Id::Standard(stdid) => {
                glblheader.set_xtd(false);
                glblheader.set_id((stdid.as_raw() as u32) << 18);
            }
        }

        let mut txhdr = TxHeader(0);
        txhdr.set_dlc(self.dlc as u8); // cast safety - checked on all ingest to ensure it's <= 8.

        if let Some(mm) = marker {
            txhdr.set_mm(mm);
            txhdr.set_efc(true);
        }

        TxBufferElement {
            hdr: glblheader,
            txhdr,
            data: self.data,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    // Confirm that construction and conversion into a TxBufferElement works as expected for standard frames.
    fn convert_for_transmission_standard() {
        let id = StandardId::new(0x7FF).unwrap();
        let data = [1, 2, 3, 4];
        let frame = MCanFrame::new(id, &data).expect("Frame creation failed");

        // Test with a marker (event FIFO enabled)
        let tx_element = frame.to_tx_buffer(Some(0xAA));

        // Standard IDs must be shifted 18 bits left in the M_CAN message RAM format
        assert_eq!(tx_element.hdr.id(), (0x7FF << 18));
        assert!(!tx_element.hdr.xtd());
        assert!(!tx_element.hdr.rtr());
        assert_eq!(tx_element.txhdr.dlc(), 4);
        assert_eq!(tx_element.txhdr.mm(), 0xAA);
        assert!(tx_element.txhdr.efc());
        assert_eq!(tx_element.data[..4], data);
    }

    #[test]
    // Confirm that construction and conversion into a TxBufferElement works as expected for extended frames.
    fn convert_for_transmission_extended() {
        let id = ExtendedId::new(0x1234567).unwrap();
        let data = [0xDE, 0xAD, 0xBE, 0xEF];
        let frame = MCanFrame::new(id, &data).expect("Frame creation failed");

        // Test without a marker (event FIFO disabled)
        let tx_element = frame.to_tx_buffer(None);

        // Extended IDs are stored as-is
        assert_eq!(tx_element.hdr.id(), 0x1234567);
        assert!(tx_element.hdr.xtd());
        assert_eq!(tx_element.txhdr.dlc(), 4);
        assert!(!tx_element.txhdr.efc());
        assert_eq!(tx_element.data[..4], data);
    }

    #[test]
    // Confirm that construction and conversion into a TxBufferElement works as expected for remote frames.
    fn convert_for_transmission_remote() {
        let id = StandardId::new(0x123).unwrap();

        let frame = MCanFrame::new_remote(id, 8).expect("Remote frame creation failed");

        let tx_element = frame.to_tx_buffer(None);

        assert!(tx_element.hdr.rtr());
        assert_eq!(tx_element.txhdr.dlc(), 8);
        assert_eq!(tx_element.hdr.id(), (0x123 << 18));
    }

    #[test]
    fn test_dlc_boundaries() {
        // Test zero-length data
        let id = Id::Standard(StandardId::new(0x123).unwrap());
        let frame_empty = MCanFrame::new(id, &[]).unwrap();
        assert_eq!(frame_empty.dlc(), 0);
        assert_eq!(frame_empty.to_tx_buffer(None).txhdr.dlc(), 0);

        // Test max-length data (8 bytes)
        let data = [0xFF; 8];
        let frame_full = MCanFrame::new(id, &data).unwrap();
        assert_eq!(frame_full.dlc(), 8);
        assert_eq!(frame_full.data()[..8], data);
    }

    #[test]
    fn test_invalid_data_length() {
        let id = Id::Standard(StandardId::new(0x123).unwrap());
        // Attempting to create a frame with 9 bytes should return None
        let too_much_data = [0u8; 9];
        let frame = MCanFrame::new(id, &too_much_data);
        assert!(frame.is_none());

        let frame = MCanFrame::new_remote(id, too_much_data.len());
        assert!(frame.is_none());
    }

    #[test]
    fn test_rx_to_mcan_frame_standard() {
        // Manually construct an RxBufferElement as the hardware would
        // For standard IDs, hardware puts them in bits [28:18]
        let mut hdr = MsgHeader(0);
        hdr.set_id(0x7FF << 18);
        hdr.set_xtd(false);
        hdr.set_rtr(false);

        let test_data = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];

        let rx_element = RxBufferElement {
            hdr,
            rxhdr: {
                let mut h = crate::can::msgram::RxHeader(0);
                h.set_dlc(4);
                h
            },
            data: test_data,
        };

        let frame = MCanFrame::from(rx_element);

        if let Id::Standard(sid) = frame.id() {
            assert_eq!(sid.as_raw(), 0x7FF);
        } else {
            panic!("Expected Standard ID");
        }
        assert_eq!(frame.dlc(), 4);

        assert_eq!(frame.data()[..4], test_data[..4]);
    }

    #[test]
    fn test_rx_to_mcan_frame_extended_remote() {
        // Manually construct an RxBufferElement as the hardware would for extended frame with rtr bit set.
        let mut hdr = MsgHeader(0);
        hdr.set_id(0x7FF);
        hdr.set_xtd(true);
        hdr.set_rtr(true);

        let rx_element = RxBufferElement {
            hdr,
            rxhdr: {
                let mut h = crate::can::msgram::RxHeader(0);
                h.set_dlc(5);
                h
            },
            data: [0x0u8; MAX_DATA_LEN],
        };

        let frame = MCanFrame::from(rx_element);

        if let Id::Extended(sid) = frame.id() {
            assert_eq!(sid.as_raw(), 0x7FF);
        } else {
            panic!("Expected Extended ID");
        }
        assert_eq!(frame.dlc(), 5);
    }
}

//! Module containing the tools needed to define and configure CAN frames for Classic CAN.
//!
//! The type(s) defined in this module are compatible with the `embedded-can` crate.

pub use crate::flexcan::id::{ExtendedId, Id, StandardId};

/// Represents the possible kinds of CAN frames.
#[derive(PartialEq)]
pub(in crate::flexcan) enum FrameKind {
    /// A "normal" CAN frame. Corresponds to RTR bit = 0.
    DataFrame,

    /// A Remote CAN frame. Corresponds to RTR bit = 1.
    RemoteFrame,
}

/// A CAN frame.
pub struct Frame {
    pub(in crate::flexcan) kind: FrameKind,
    pub(in crate::flexcan) id: Id,
    pub(in crate::flexcan) length: usize,
    pub(in crate::flexcan) data: [u8; 8],
}

impl Frame {
    /// Creates a new data frame.
    ///
    /// Returns `None` if `data` is longer than 8 bytes.
    pub fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
        if data.len() > 8 {
            return None;
        } // Make sure we're the right size.

        let mut buf = [0u8; 8];
        buf[..data.len()].copy_from_slice(data);

        Some(Frame {
            kind: FrameKind::DataFrame,
            id: id.into(),
            length: data.len(),
            data: buf,
        })
    }

    /// Creates a new remote frame.
    ///
    /// Returns `None` if `dlc` is greater than 8.
    pub fn new_remote(id: impl Into<Id>, dlc: usize) -> Option<Self> {
        if dlc > 8 {
            return None;
        } // Make sure we're the right size.

        Some(Frame {
            kind: FrameKind::RemoteFrame,
            id: id.into(),
            length: dlc,
            data: [0u8; 8],
        })
    }

    /// Returns `true` if this frame uses an extended (29-bit) ID.
    pub fn is_extended(&self) -> bool {
        matches!(self.id, Id::Extended(_))
    }

    /// Returns `true` if this is a remote frame (RTR = 1).
    pub fn is_remote_frame(&self) -> bool {
        self.kind == FrameKind::RemoteFrame
    }

    /// Returns this frame's identifier.
    pub fn id(&self) -> Id {
        self.id
    }

    /// Returns this frame's data length code (number of data bytes).
    pub fn dlc(&self) -> usize {
        self.length
    }

    /// Returns this frame's payload bytes (empty for remote frames).
    pub fn data(&self) -> &[u8] {
        match self.kind {
            FrameKind::RemoteFrame => &[],
            FrameKind::DataFrame => &self.data[..self.length],
        }
    }
}

impl embedded_can::Frame for Frame {
    fn new(id: impl Into<embedded_can::Id>, data: &[u8]) -> Option<Self> {
        Self::new(id.into(), data)
    }
    fn new_remote(id: impl Into<embedded_can::Id>, dlc: usize) -> Option<Self> {
        Self::new_remote(id.into(), dlc)
    }
    fn is_extended(&self) -> bool {
        Self::is_extended(self)
    }
    fn is_remote_frame(&self) -> bool {
        Self::is_remote_frame(self)
    }
    fn id(&self) -> embedded_can::Id {
        Self::id(self).into()
    }
    fn dlc(&self) -> usize {
        Self::dlc(self)
    }
    fn data(&self) -> &[u8] {
        Self::data(self)
    }
}

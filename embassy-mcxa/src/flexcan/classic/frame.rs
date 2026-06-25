pub use embedded_can::{Id, StandardId, ExtendedId};

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

impl embedded_can::Frame for Frame {
    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
        if data.len() > 8 { return None; } // Make sure we're the right size.

        let mut buf = [0u8; 8];
        buf[..data.len()].copy_from_slice(data);

        Some(Frame {
            kind: FrameKind::DataFrame,
            id: id.into(),
            length: data.len(),
            data: buf,
        })
    }

    fn new_remote(id: impl Into<Id>, dlc: usize) -> Option<Self> {
        if dlc > 8 { return None; } // Make sure we're the right size.

        Some(Frame {
            kind: FrameKind::RemoteFrame,
            id: id.into(),
            length: dlc,
            data: [0u8; 8],
        })
    }

    fn is_extended(&self) -> bool {
        matches!(self.id, Id::Extended(_))
    }

    fn is_remote_frame(&self) -> bool {
        return self.kind == FrameKind::RemoteFrame;
    }

    fn id(&self) -> Id {
        return self.id;
    }

    fn dlc(&self) -> usize {
        return self.length;
    }

    fn data(&self) -> &[u8] {
        match self.kind {
            FrameKind::RemoteFrame => return &[],
            FrameKind::DataFrame   => return &self.data[..self.length],
        }
    }

}
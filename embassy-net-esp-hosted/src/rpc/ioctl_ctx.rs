use micropb::{MessageDecode, MessageEncode, PbEncoder};

use crate::control::Error;
use crate::ioctl::Shared;

/// One encode → ioctl → decode round-trip through [`Shared`].
pub struct IoctlCtx<'a> {
    shared: &'a Shared,
}

impl IoctlCtx<'_> {
    pub fn new(shared: &Shared) -> IoctlCtx<'_> {
        IoctlCtx { shared }
    }

    pub async fn exchange(&mut self, msg: &mut (impl MessageDecode + MessageEncode)) -> Result<(), Error> {
        // Theoretical max overhead is 29 bytes. Biggest message is OTA write with 256 bytes.
        let mut buf = [0u8; 256 + 29];
        let buf_len = buf.len();

        let mut encoder = PbEncoder::new(&mut buf[..]);
        msg.encode(&mut encoder).map_err(|_| {
            warn!("failed to serialize control request");
            Error::Internal
        })?;
        let remaining = encoder.into_writer();
        let req_len = buf_len - remaining.len();

        struct CancelOnDrop<'a>(&'a Shared);

        impl CancelOnDrop<'_> {
            fn defuse(self) {
                core::mem::forget(self);
            }
        }

        impl Drop for CancelOnDrop<'_> {
            fn drop(&mut self) {
                self.0.ioctl_cancel();
            }
        }

        let ioctl = CancelOnDrop(self.shared);

        let resp_len = ioctl.0.ioctl(&mut buf, req_len).await;

        ioctl.defuse();

        msg.decode_from_bytes(&buf[..resp_len]).map_err(|_| {
            warn!("failed to deserialize control response");
            Error::Internal
        })?;

        Ok(())
    }
}

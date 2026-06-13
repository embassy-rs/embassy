use micropb::{MessageDecode, MessageEncode, PbEncoder};

use crate::MAX_IOCTL_SIZE;
use crate::control::Error;
use crate::ioctl::Shared;

/// One encode → ioctl → decode round-trip through [`Shared`].
pub struct IoctlCtx<'a> {
    pub(crate) shared: &'a Shared,
    ioctl_buffer: &'a mut [u8; MAX_IOCTL_SIZE],
}

impl<'a> IoctlCtx<'a> {
    pub fn new(shared: &'a Shared, ioctl_buffer: &'a mut [u8; MAX_IOCTL_SIZE]) -> IoctlCtx<'a> {
        IoctlCtx { shared, ioctl_buffer }
    }

    pub async fn exchange(&mut self, msg: &mut (impl MessageDecode + MessageEncode)) -> Result<(), Error> {
        let buf_len = self.ioctl_buffer.len();

        let mut encoder = PbEncoder::new(&mut self.ioctl_buffer[..]);
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

        let resp_len = ioctl.0.ioctl(self.ioctl_buffer, req_len).await;

        ioctl.defuse();

        msg.decode_from_bytes(&self.ioctl_buffer[..resp_len]).map_err(|_| {
            warn!("failed to deserialize control response");
            Error::Internal
        })?;

        Ok(())
    }
}

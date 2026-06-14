use micropb::{MessageDecode, MessageEncode, PbDecoder, PbEncoder};

use crate::MAX_IOCTL_SIZE;
use crate::control::Error;
use crate::ioctl::Shared;

#[derive(Default)]
pub(crate) enum IoctlMessage {
    #[default]
    None,

    #[cfg(feature = "esp-hosted-fg")]
    Fg(crate::proto::fg::CtrlMsg),

    #[cfg(feature = "esp-hosted-mcu")]
    Mcu(crate::proto::mcu::Rpc),
}

/// One encode → ioctl → decode round-trip through [`Shared`].
pub struct IoctlCtx<'a> {
    pub(crate) shared: &'a Shared,
    ioctl_buffer: &'a mut [u8; MAX_IOCTL_SIZE],
    msg_buffer: &'a mut IoctlMessage,
}

pub struct Ioctl<'a> {
    shared: &'a Shared,
    ioctl_buffer: &'a mut [u8; MAX_IOCTL_SIZE],
}

impl<'a> IoctlCtx<'a> {
    pub fn new(
        shared: &'a Shared,
        ioctl_buffer: &'a mut [u8; MAX_IOCTL_SIZE],
        msg_buffer: &'a mut IoctlMessage,
    ) -> IoctlCtx<'a> {
        IoctlCtx {
            shared,
            ioctl_buffer,
            msg_buffer,
        }
    }

    #[cfg(feature = "esp-hosted-fg")]
    pub(crate) fn fg(&mut self) -> (Ioctl<'_>, &mut crate::proto::fg::CtrlMsg) {
        *self.msg_buffer = IoctlMessage::Fg(Default::default());

        let ioctl = Ioctl {
            shared: self.shared,
            ioctl_buffer: self.ioctl_buffer,
        };

        if let IoctlMessage::Fg(msg) = self.msg_buffer {
            (ioctl, msg)
        } else {
            unreachable!()
        }
    }

    #[cfg(feature = "esp-hosted-mcu")]
    pub(crate) fn mcu(&mut self) -> (Ioctl<'_>, &mut crate::proto::mcu::Rpc) {
        *self.msg_buffer = IoctlMessage::Mcu(Default::default());

        let ioctl = Ioctl {
            shared: self.shared,
            ioctl_buffer: self.ioctl_buffer,
        };

        if let IoctlMessage::Mcu(msg) = self.msg_buffer {
            (ioctl, msg)
        } else {
            unreachable!()
        }
    }
}

impl Ioctl<'_> {
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

        let mut decoder = PbDecoder::new(&self.ioctl_buffer[..]);
        // Do not fail if we receive more data than what fits into the statically sized vecs.
        decoder.ignore_repeated_cap_err = true;
        msg.decode(&mut decoder, resp_len).map_err(|_| {
            warn!("failed to deserialize control response");
            Error::Internal
        })?;

        Ok(())
    }
}

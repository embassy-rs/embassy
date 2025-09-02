//! TODO

use crate::{AlignedBuffer, BlockingFirmwareUpdater, FirmwareUpdaterError};
use embassy_embedded_hal::Reset;
use embassy_usb::class::dfu::{consts::Error, dfu_mode::Handler};
use embedded_storage::nor_flash::{NorFlash, NorFlashErrorKind};

/// Internal state for USB DFU
pub struct Control<'d, DFU: NorFlash, STATE: NorFlash, RST: Reset, const BLOCK_SIZE: usize> {
    updater: BlockingFirmwareUpdater<'d, DFU, STATE>,
    reset: RST,

    offset: usize,
    buf: AlignedBuffer<BLOCK_SIZE>,

    #[cfg(feature = "_verify")]
    public_key: &'static [u8; 32],
}

impl<'d, DFU: NorFlash, STATE: NorFlash, RST: Reset, const BLOCK_SIZE: usize> Control<'d, DFU, STATE, RST, BLOCK_SIZE> {
    /// TODO
    pub fn new(
        updater: BlockingFirmwareUpdater<'d, DFU, STATE>,
        reset: RST,
        #[cfg(feature = "_verify")] public_key: &'static [u8; 32],
    ) -> Self {
        Self {
            updater,
            reset,

            offset: 0,
            buf: AlignedBuffer([0; BLOCK_SIZE]),

            #[cfg(feature = "_verify")]
            public_key,
        }
    }
}

impl<'d, DFU: NorFlash, STATE: NorFlash, RST: Reset, const BLOCK_SIZE: usize> Handler
    for Control<'d, DFU, STATE, RST, BLOCK_SIZE>
{
    fn start(&mut self) {
        self.offset = 0;
    }

    fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        if data.len() > BLOCK_SIZE {
            error!("USB data len exceeded block size");
            return Err(Error::Unknown);
        }

        debug!("Copying {} bytes to buffer", data.len());
        self.buf.as_mut()[..data.len()].copy_from_slice(data);

        self.updater
            .write_firmware(self.offset, self.buf.as_ref())
            .map_err(map_err)
    }

    fn finish(&mut self) -> Result<(), Error> {
        #[cfg(feature = "_verify")]
        let update_res: Result<(), FirmwareUpdaterError> = {
            const SIGNATURE_LEN: usize = 64;

            let mut signature = [0; SIGNATURE_LEN];
            let update_len = (self.offset - SIGNATURE_LEN) as u32;

            self.updater.read_dfu(update_len, &mut signature).and_then(|_| {
                self.updater
                    .verify_and_mark_updated(self.public_key, &signature, update_len)
            })
        };

        #[cfg(not(feature = "_verify"))]
        let update_res = self.updater.mark_updated();

        update_res.map_err(map_err)
    }

    fn switch_to_app(&mut self) {
        self.reset.sys_reset();
    }
}

fn map_err(e: FirmwareUpdaterError) -> Error {
    match e {
        FirmwareUpdaterError::Flash(e) => match e {
            NorFlashErrorKind::NotAligned => Error::Write,
            NorFlashErrorKind::OutOfBounds => Error::Address,
            _ => Error::Unknown,
        },
        FirmwareUpdaterError::Signature(_) => Error::Verify,
        FirmwareUpdaterError::BadState => Error::Unknown,
    }
}

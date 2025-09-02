//! TODO

use crate::BlockingFirmwareState;
use embassy_embedded_hal::Reset;
use embassy_usb::class::dfu::app_mode::Handler;
use embedded_storage::nor_flash::NorFlash;

/// Internal state fo
pub struct Control<'d, STATE: NorFlash, RST: Reset> {
    state: BlockingFirmwareState<'d, STATE>,
    reset: RST,
}

impl<'d, STATE: NorFlash, RST: Reset> Control<'d, STATE, RST> {
    /// TODO
    pub fn new(state: BlockingFirmwareState<'d, STATE>, reset: RST) -> Self {
        Self { state, reset }
    }
}

impl<'d, STATE: NorFlash, RST: Reset> Handler for Control<'d, STATE, RST> {
    fn switch_to_dfu(&mut self) {
        self.state.mark_dfu().expect("Failed to mark DFU mode in bootloader");
        self.reset.sys_reset()
    }
}

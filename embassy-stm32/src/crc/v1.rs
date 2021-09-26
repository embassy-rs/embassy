use crate::pac::{CRC as PAC_CRC, RCC};
use crate::peripherals::CRC;
use crate::rcc::sealed::RccPeripheral;

pub struct Crc {
    _peripheral: CRC
}

impl Crc{
    pub fn new(peripheral: CRC) -> Self{
        // enable CRC clock in RCC.
        CRC::enable();
        // Reset CRC to default values.
        CRC::reset();
        Self { _peripheral: peripheral}
    }

    pub fn reset() {
        unsafe { PAC_CRC.cr().modify(|w| w.set_reset(true)) };
    }
}
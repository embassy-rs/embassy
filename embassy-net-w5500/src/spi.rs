use embedded_hal_async::spi::{Operation, SpiDevice};

use crate::device::RegisterBlock;

pub type Address = (RegisterBlock, u16);

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SpiInterface<SPI>(pub SPI);

impl<SPI: SpiDevice> SpiInterface<SPI> {
    pub async fn read_frame(&mut self, address: Address, data: &mut [u8]) -> Result<(), SPI::Error> {
        let address_phase = address.1.to_be_bytes();
        let control_phase = [(address.0 as u8) << 3];
        let operations = &mut [
            Operation::Write(&address_phase),
            Operation::Write(&control_phase),
            Operation::TransferInPlace(data),
        ];
        self.0.transaction(operations).await
    }

    pub async fn write_frame(&mut self, address: Address, data: &[u8]) -> Result<(), SPI::Error> {
        let address_phase = address.1.to_be_bytes();
        let control_phase = [(address.0 as u8) << 3 | 0b0000_0100];
        let data_phase = data;
        let operations = &mut [
            Operation::Write(&address_phase[..]),
            Operation::Write(&control_phase),
            Operation::Write(&data_phase),
        ];
        self.0.transaction(operations).await
    }
}

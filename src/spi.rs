use crate::device::RegisterBlock;
use embedded_hal_async::spi::{Operation, SpiDevice};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SpiInterface<SPI>(pub SPI);

impl<SPI: SpiDevice> SpiInterface<SPI> {
    pub async fn read_frame(
        &mut self,
        block: RegisterBlock,
        address: u16,
        data: &mut [u8],
    ) -> Result<(), SPI::Error> {
        let address_phase = address.to_be_bytes();
        let control_phase = [(block as u8) << 3];
        let operations = &mut [
            Operation::Write(&address_phase),
            Operation::Write(&control_phase),
            Operation::TransferInPlace(data),
        ];
        self.0.transaction(operations).await
    }

    pub async fn write_frame(
        &mut self,
        block: RegisterBlock,
        address: u16,
        data: &[u8],
    ) -> Result<(), SPI::Error> {
        let address_phase = address.to_be_bytes();
        let control_phase = [(block as u8) << 3 | 0b0000_0100];
        let data_phase = data;
        let operations = &[&address_phase[..], &control_phase, &data_phase];
        self.0.write_transaction(operations).await
    }
}

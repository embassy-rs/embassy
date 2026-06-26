use embedded_hal_async::spi;

use crate::wiznet_spi_interface::{SpiType, WiznetSpiBus, WiznetSpiOperation};

impl<'a> From<WiznetSpiOperation<'a>> for spi::Operation<'a, u8> {
    fn from(value: WiznetSpiOperation<'a>) -> Self {
        match value {
            WiznetSpiOperation::Read(data) => spi::Operation::Read(data),
            WiznetSpiOperation::Write(data) | WiznetSpiOperation::WriteSingleLine(data) => spi::Operation::Write(data),
        }
    }
}

impl<SPI: spi::SpiDevice> WiznetSpiBus for SPI {
    const SPI_TYPE: SpiType = SpiType::Single;

    async fn transaction<'a, const N: usize>(
        &mut self,
        operations: [WiznetSpiOperation<'a>; N],
    ) -> Result<(), Self::Error> {
        let mut ops = operations.map(|op| op.into());
        self.transaction(&mut ops).await
    }
}

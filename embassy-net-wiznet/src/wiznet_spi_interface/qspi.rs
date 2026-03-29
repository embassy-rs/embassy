use embassy_embedded_hal::qspi;
use embedded_hal_async::spi::ErrorType;

use crate::wiznet_spi_interface::{SpiType, WiznetSpiBus, WiznetSpiOperation};

impl<'a> From<WiznetSpiOperation<'a>> for qspi::traits::Operation<'a, u8> {
    fn from(value: WiznetSpiOperation<'a>) -> Self {
        match value {
            WiznetSpiOperation::Read(data) => qspi::traits::Operation::Read(data),
            WiznetSpiOperation::Write(data)  => qspi::traits::Operation::Write(data),
            WiznetSpiOperation::WriteSingleLine(data) => qspi::traits::Operation::WriteSingleLine(data),
        }
    }
}

/// Wrapper to use QSPI with Wiznet device
pub struct WiznetQspiBus<T: qspi::traits::QspiDevice>(pub T);

impl<T: qspi::traits::QspiDevice> ErrorType for WiznetQspiBus<T> {
    type Error = T::Error;
}

impl<T: qspi::traits::QspiDevice> WiznetSpiBus for WiznetQspiBus<T> {
    const SPI_TYPE: SpiType = SpiType::Quad;

    async fn transaction<'a, const N: usize>(
        &mut self,
        operations: [WiznetSpiOperation<'a>; N],
    ) -> Result<(), T::Error> {
        let mut ops = operations.map(|op| op.into());
        self.0.transaction(&mut ops).await
    }
}

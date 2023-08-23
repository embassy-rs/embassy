use embedded_hal_1::digital::OutputPin;

use crate::{Runner, SpiBusCyw43};

pub struct BleConnector<'a, PWR, SPI> {
    runner: Runner<'a, PWR, SPI>,
}

impl<'a, PWR, SPI> BleConnector<'a, PWR, SPI>
where
    PWR: OutputPin,
    SPI: SpiBusCyw43,
{
    pub fn new(runner: Runner<'a, PWR, SPI>) -> BleConnector<'a, PWR, SPI> {
        return BleConnector { runner };
    }
}

#[derive(Debug)]
pub enum BleConnectorError {
    Unknown,
}

impl embedded_io_async::Error for BleConnectorError {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        embedded_io_async::ErrorKind::Other
    }
}

impl<'a, PWR, SPI> embedded_io_async::ErrorType for BleConnector<'a, PWR, SPI>
where
    PWR: OutputPin,
    SPI: SpiBusCyw43,
{
    type Error = BleConnectorError;
}

impl<'a, PWR, SPI> embedded_io_async::Read for BleConnector<'a, PWR, SPI>
where
    PWR: OutputPin,
    SPI: SpiBusCyw43,
{
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, BleConnectorError>
    where
        PWR: OutputPin,
        SPI: SpiBusCyw43,
    {
        // TODO: error handling?
        Ok(self.runner.hci_read(buf).await as usize)
    }
}

impl<'a, PWR, SPI> embedded_io_async::Write for BleConnector<'a, PWR, SPI>
where
    PWR: OutputPin,
    SPI: SpiBusCyw43,
{
    async fn write(&mut self, buf: &[u8]) -> Result<usize, BleConnectorError> {
        // TODO: error handling?
        self.runner.hci_write(buf).await;
        // TODO: do not assume entire buffer written?
        Ok(buf.len())
    }
}

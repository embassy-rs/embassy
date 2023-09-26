pub struct HciConnector {}

impl HciConnector {
    pub fn new() -> HciConnector {
        return HciConnector {};
    }
}

#[derive(Debug)]
pub enum HciConnectorError {
    Unknown,
}

impl embedded_io_async::Error for HciConnectorError {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        embedded_io_async::ErrorKind::Other
    }
}

impl embedded_io_async::ErrorType for HciConnector {
    type Error = HciConnectorError;
}

impl embedded_io_async::Read for HciConnector {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, HciConnectorError> {
        // TODO: how to get all the way to runner.hci_read()?
        Ok(0)
    }
}

impl embedded_io_async::Write for HciConnector {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, HciConnectorError> {
        // TODO: how to get all the way to runner.hci_write()?
        Ok(0)
    }
}

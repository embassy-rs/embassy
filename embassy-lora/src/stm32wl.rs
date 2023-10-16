use core::fmt::Debug;

use embassy_stm32::peripherals::SUBGHZSPI;
use embassy_stm32::{pac, spi as stm_spi, Peripheral};
use embedded_hal_async::spi::{self, Operation};

pub struct SubGhzSpiDevice<'d, Tx, Rx>
where
    Tx: stm_spi::TxDma<SUBGHZSPI>,
    Rx: stm_spi::RxDma<SUBGHZSPI>,
{
    bus: stm_spi::Spi<'d, SUBGHZSPI, Tx, Rx>,
}

impl<'d, Tx, Rx> SubGhzSpiDevice<'d, Tx, Rx>
where
    Tx: stm_spi::TxDma<SUBGHZSPI>,
    Rx: stm_spi::RxDma<SUBGHZSPI>,
{
    pub fn new(
        spi: impl Peripheral<P = SUBGHZSPI> + 'd,
        txdma: impl Peripheral<P = Tx> + 'd,
        rxdma: impl Peripheral<P = Rx> + 'd,
    ) -> Self {
        let bus = stm_spi::Spi::new_subghz(spi, txdma, rxdma);
        Self { bus }
    }
}

/// Error returned by SPI device implementations in this crate.
#[derive(Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum SpiDeviceError {
    /// An operation on the inner SPI bus failed.
    Spi(stm_spi::Error),
    /// DelayUs operations are not supported when the `time` Cargo feature is not enabled.
    DelayUsNotSupported,
}

impl spi::Error for SpiDeviceError {
    fn kind(&self) -> spi::ErrorKind {
        match self {
            Self::Spi(e) => e.kind(),
            Self::DelayUsNotSupported => spi::ErrorKind::Other,
        }
    }
}

impl<Tx, Rx> spi::ErrorType for SubGhzSpiDevice<'_, Tx, Rx>
where
    Tx: stm_spi::TxDma<SUBGHZSPI>,
    Rx: stm_spi::RxDma<SUBGHZSPI>,
{
    type Error = SpiDeviceError;
}

impl<Tx, Rx> spi::SpiDevice for SubGhzSpiDevice<'_, Tx, Rx>
where
    Tx: stm_spi::TxDma<SUBGHZSPI>,
    Rx: stm_spi::RxDma<SUBGHZSPI>,
{
    async fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        pac::PWR.subghzspicr().modify(|w| w.set_nss(false));

        let op_res: Result<(), Self::Error> = try {
            for op in operations {
                match op {
                    Operation::Read(buf) => self.bus.read(buf).await.map_err(SpiDeviceError::Spi)?,
                    Operation::Write(buf) => self.bus.write(buf).await.map_err(SpiDeviceError::Spi)?,
                    Operation::Transfer(read, write) => {
                        self.bus.transfer(read, write).await.map_err(SpiDeviceError::Spi)?
                    }
                    Operation::TransferInPlace(buf) => {
                        self.bus.transfer_in_place(buf).await.map_err(SpiDeviceError::Spi)?
                    }
                    #[cfg(not(feature = "time"))]
                    Operation::DelayUs(_) => Err(SpiDeviceError::DelayUsNotSupported)?,
                    #[cfg(feature = "time")]
                    Operation::DelayUs(us) => embassy_time::Timer::after_micros(*us as _).await,
                }
            }
        };

        pac::PWR.subghzspicr().modify(|w| w.set_nss(true));

        op_res
    }
}

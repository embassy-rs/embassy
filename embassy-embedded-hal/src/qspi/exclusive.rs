//! QSPI bus sharing mechanisms.

use super::traits::{ErrorType, Operation, QspiBus, QspiDevice};
use embedded_hal_1::digital::OutputPin;
use embedded_hal_async::delay::DelayNs;

use super::device_error::DeviceError;

/// Dummy [`DelayNs`](embedded_hal::delay::DelayNs) implementation that panics on use.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NoDelay;

/// [`QspiDevice`] implementation with exclusive access to the bus (not shared).
///
/// This is the most straightforward way of obtaining an [`QspiDevice`] from an [`QspiBus`],
/// ideal for when no sharing is required (only one SPI device is present on the bus).
pub struct ExclusiveDevice<BUS, CS, D> {
    bus: BUS,
    cs: CS,
    delay: D,
}

impl<BUS, CS, D> ExclusiveDevice<BUS, CS, D> {
    /// Create a new [`ExclusiveDevice`].
    ///
    /// This sets the `cs` pin high, and returns an error if that fails. It is recommended
    /// to set the pin high the moment it's configured as an output, to avoid glitches.
    #[inline]
    pub fn new(bus: BUS, mut cs: CS, delay: D) -> Result<Self, CS::Error>
    where
        CS: OutputPin,
    {
        cs.set_high()?;
        Ok(Self { bus, cs, delay })
    }

    /// Returns a reference to the underlying bus object.
    #[inline]
    pub fn bus(&self) -> &BUS {
        &self.bus
    }

    /// Returns a mutable reference to the underlying bus object.
    #[inline]
    pub fn bus_mut(&mut self) -> &mut BUS {
        &mut self.bus
    }
}

impl<BUS, CS> ExclusiveDevice<BUS, CS, NoDelay> {
    /// Create a new [`ExclusiveDevice`] without support for in-transaction delays.
    ///
    /// This sets the `cs` pin high, and returns an error if that fails. It is recommended
    /// to set the pin high the moment it's configured as an output, to avoid glitches.
    ///
    /// **Warning**: The returned instance *technically* doesn't comply with the `QspiDevice`
    /// contract, which mandates delay support. It is relatively rare for drivers to use
    /// in-transaction delays, so you might still want to use this method because it's more practical.
    ///
    /// Note that a future version of the driver might start using delays, causing your
    /// code to panic. This wouldn't be considered a breaking change from the driver side, because
    /// drivers are allowed to assume `QspiDevice` implementations comply with the contract.
    /// If you feel this risk outweighs the convenience of having `cargo` automatically upgrade
    /// the driver crate, you might want to pin the driver's version.
    ///
    /// # Panics
    ///
    /// The returned device will panic if you try to execute a transaction
    /// that contains any operations of type [`Operation::DelayNs`].
    #[inline]
    pub fn new_no_delay(bus: BUS, mut cs: CS) -> Result<Self, CS::Error>
    where
        CS: OutputPin,
    {
        cs.set_high()?;
        Ok(Self {
            bus,
            cs,
            delay: NoDelay,
        })
    }
}

impl<BUS, CS, D> ErrorType for ExclusiveDevice<BUS, CS, D>
where
    BUS: ErrorType,
    CS: OutputPin,
{
    type Error = DeviceError<BUS::Error, CS::Error>;
}

impl<Word: Copy + 'static, BUS, CS, D> QspiDevice<Word> for ExclusiveDevice<BUS, CS, D>
where
    BUS: QspiBus<Word>,
    CS: OutputPin,
    D: DelayNs,
{
    #[inline]
    async fn transaction(&mut self, operations: &mut [Operation<'_, Word>]) -> Result<(), Self::Error> {
        self.cs.set_low().map_err(DeviceError::Cs)?;

        let op_res = 'ops: {
            for op in operations {
                let res = match op {
                    Operation::Read(buf) => self.bus.read(buf).await,
                    Operation::Write(buf) => self.bus.write(buf).await,
                    Operation::WriteSingleLine(buf) => self.bus.write_single_line(buf).await,
                    Operation::DelayNs(ns) => match self.bus.flush().await {
                        Err(e) => Err(e),
                        Ok(()) => {
                            self.delay.delay_ns(*ns).await;
                            Ok(())
                        }
                    },
                };
                if let Err(e) = res {
                    break 'ops Err(e);
                }
            }
            Ok(())
        };

        // On failure, it's important to still flush and deassert CS.
        let flush_res = self.bus.flush().await;
        let cs_res = self.cs.set_high();

        op_res.map_err(DeviceError::Qspi)?;
        flush_res.map_err(DeviceError::Qspi)?;
        cs_res.map_err(DeviceError::Cs)?;

        Ok(())
    }
}

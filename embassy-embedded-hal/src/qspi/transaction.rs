//! Module for QSPI transaction
use super::traits::{ErrorType, Operation, QspiBus};
use embedded_hal_1::delay::DelayNs;
use embedded_hal_1::digital::OutputPin;

use super::device_error::DeviceError;

/// Common implementation to perform a transaction against the device.
#[inline]
pub fn transaction<Word, BUS, CS, D>(
    operations: &mut [Operation<Word>],
    bus: &mut BUS,
    delay: &mut D,
    cs: &mut CS,
) -> Result<(), DeviceError<BUS::Error, CS::Error>>
where
    BUS: QspiBus<Word> + ErrorType,
    CS: OutputPin,
    D: DelayNs,
    Word: Copy,
{
    cs.set_low().map_err(DeviceError::Cs)?;

    let op_res = operations.iter_mut().try_for_each(|op| match op {
        Operation::Read(buf) => bus.read(buf),
        Operation::Write(buf) => bus.write(buf),
        Operation::WriteSingleLine(buf) => bus.write_single_line(buf),
        Operation::DelayNs(ns) => {
            bus.flush()?;
            delay.delay_ns(*ns);
            Ok(())
        }
    });

    // On failure, it's important to still flush and deassert CS.
    let flush_res = bus.flush();
    let cs_res = cs.set_high();

    op_res.map_err(DeviceError::Qspi)?;
    flush_res.map_err(DeviceError::Qspi)?;
    cs_res.map_err(DeviceError::Cs)?;

    Ok(())
}

use embassy::channel::signal::Signal;
use embassy::util::{select, Either};

use crate::driver::Driver;
use crate::UsbDevice;

/// Am enabled usb device is a device that further receives external notifications
/// regarding whether it is enabled or not. A common example of where this is
/// required is when receiving notifications from the POWER peripheral that
/// USB has been connected to or removed. The device here wraps an existing
/// USB device, keeping it publically available so that device-oriented operations
/// may still be performed. A signal is also provided that enables/disables the
/// USB device, taking care of suspension and resumption. In the case of the POWER
/// peripheral, this signal can be used from within a POWER_CLOCK interrupt
/// handler. Alternatively, for softdevice usage where the POWER peripheral is not
/// available, similar USB power events can be leveraged.
pub struct EnabledUsbDevice<'d, D: Driver<'d>> {
    pub underlying: UsbDevice<'d, D>,
    enable_usb_signal: &'d Signal<bool>,
}

impl<'d, D: Driver<'d>> EnabledUsbDevice<'d, D> {
    /// Wrap an existing UsbDevice and take a signal that will be used
    /// to enable/disable it, perhaps from an external POWER_CLOCK
    /// interrupt, or the equivalent when dealing with softdevices.
    pub fn new(underlying: UsbDevice<'d, D>, enable_usb_signal: &'d Signal<bool>) -> Self {
        Self {
            underlying,
            enable_usb_signal,
        }
    }

    /// Runs the underlying `UsbDevice` taking care of reacting to USB becoming
    /// enabled/disabled.
    ///
    /// This future may leave the bus in an invalid state if it is dropped.
    /// After dropping the future, [`UsbDevice::disable()`] should be called
    /// before calling any other `UsbDevice` methods to fully reset the
    /// peripheral.
    pub async fn run(&mut self) -> ! {
        while !self.enable_usb_signal.wait().await {}
        loop {
            match select(
                self.underlying.run_until_suspend(),
                self.enable_usb_signal.wait(),
            )
            .await
            {
                Either::First(_) => {}
                Either::Second(enable) => {
                    if !enable {
                        self.underlying.disable().await;
                        while !self.enable_usb_signal.wait().await {}
                    }
                }
            }
            match select(self.underlying.wait_resume(), self.enable_usb_signal.wait()).await {
                Either::First(_) => {}
                Either::Second(enable) => {
                    if !enable {
                        self.underlying.disable().await;
                        while !self.enable_usb_signal.wait().await {}
                    }
                }
            }
        }
    }
}

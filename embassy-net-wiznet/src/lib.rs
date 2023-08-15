//! [`embassy-net`](https://crates.io/crates/embassy-net) driver for WIZnet ethernet chips.
#![no_std]
#![feature(async_fn_in_trait)]

pub mod chip;
mod device;

use embassy_futures::select::{select, Either};
use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::LinkState;
use embassy_time::{Duration, Timer};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;

use crate::chip::Chip;
use crate::device::WiznetDevice;

const MTU: usize = 1514;

/// Type alias for the embassy-net driver.
pub type Device<'d> = embassy_net_driver_channel::Device<'d, MTU>;

/// Internal state for the embassy-net integration.
pub struct State<const N_RX: usize, const N_TX: usize> {
    ch_state: ch::State<MTU, N_RX, N_TX>,
}

impl<const N_RX: usize, const N_TX: usize> State<N_RX, N_TX> {
    /// Create a new `State`.
    pub const fn new() -> Self {
        Self {
            ch_state: ch::State::new(),
        }
    }
}

/// Background runner for the driver.
///
/// You must call `.run()` in a background task for the driver to operate.
pub struct Runner<'d, C: Chip, SPI: SpiDevice, INT: Wait, RST: OutputPin> {
    mac: WiznetDevice<C, SPI>,
    ch: ch::Runner<'d, MTU>,
    int: INT,
    _reset: RST,
}

/// You must call this in a background task for the driver to operate.
impl<'d, C: Chip, SPI: SpiDevice, INT: Wait, RST: OutputPin> Runner<'d, C, SPI, INT, RST> {
    pub async fn run(mut self) -> ! {
        let (state_chan, mut rx_chan, mut tx_chan) = self.ch.split();
        loop {
            if self.mac.is_link_up().await {
                state_chan.set_link_state(LinkState::Up);
                loop {
                    match select(
                        async {
                            self.int.wait_for_low().await.ok();
                            rx_chan.rx_buf().await
                        },
                        tx_chan.tx_buf(),
                    )
                    .await
                    {
                        Either::First(p) => {
                            if let Ok(n) = self.mac.read_frame(p).await {
                                rx_chan.rx_done(n);
                            }
                        }
                        Either::Second(p) => {
                            self.mac.write_frame(p).await.ok();
                            tx_chan.tx_done();
                        }
                    }
                }
            } else {
                state_chan.set_link_state(LinkState::Down);
            }
        }
    }
}

/// Create a Wiznet ethernet chip driver for [`embassy-net`](https://crates.io/crates/embassy-net).
///
/// This returns two structs:
/// - a `Device` that you must pass to the `embassy-net` stack.
/// - a `Runner`. You must call `.run()` on it in a background task.
pub async fn new<'a, const N_RX: usize, const N_TX: usize, C: Chip, SPI: SpiDevice, INT: Wait, RST: OutputPin>(
    mac_addr: [u8; 6],
    state: &'a mut State<N_RX, N_TX>,
    spi_dev: SPI,
    int: INT,
    mut reset: RST,
) -> (Device<'a>, Runner<'a, C, SPI, INT, RST>) {
    // Reset the chip.
    reset.set_low().ok();
    // Ensure the reset is registered.
    Timer::after(Duration::from_millis(1)).await;
    reset.set_high().ok();

    // Wait for PLL lock. Some chips are slower than others.
    // Slowest is w5100s which is 100ms, so let's just wait that.
    Timer::after(Duration::from_millis(100)).await;

    let mac = WiznetDevice::new(spi_dev, mac_addr).await.unwrap();

    let (runner, device) = ch::new(&mut state.ch_state, ch::driver::HardwareAddress::Ethernet(mac_addr));
    (
        device,
        Runner {
            ch: runner,
            mac,
            int,
            _reset: reset,
        },
    )
}

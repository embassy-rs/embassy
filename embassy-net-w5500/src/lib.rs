#![no_std]
/// [`embassy-net`](crates.io/crates/embassy-net) driver for the WIZnet W5500 ethernet chip.
mod device;
mod socket;
mod spi;

use embassy_futures::select::{select, Either};
use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::LinkState;
use embassy_time::{Duration, Timer};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::spi::SpiDevice;

use crate::device::W5500;
const MTU: usize = 1514;

/// Type alias for the embassy-net driver for W5500
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

/// Background runner for the W5500.
///
/// You must call `.run()` in a background task for the W5500 to operate.
pub struct Runner<'d, SPI: SpiDevice, INT: Wait, RST: OutputPin> {
    mac: W5500<SPI>,
    ch: ch::Runner<'d, MTU>,
    int: INT,
    _reset: RST,
}

/// You must call this in a background task for the W5500 to operate.
impl<'d, SPI: SpiDevice, INT: Wait, RST: OutputPin> Runner<'d, SPI, INT, RST> {
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

/// Obtain a driver for using the W5500 with [`embassy-net`](crates.io/crates/embassy-net).
pub async fn new<'a, const N_RX: usize, const N_TX: usize, SPI: SpiDevice, INT: Wait, RST: OutputPin>(
    mac_addr: [u8; 6],
    state: &'a mut State<N_RX, N_TX>,
    spi_dev: SPI,
    int: INT,
    mut reset: RST,
) -> (Device<'a>, Runner<'a, SPI, INT, RST>) {
    // Reset the W5500.
    reset.set_low().ok();
    // Ensure the reset is registered.
    Timer::after(Duration::from_millis(1)).await;
    reset.set_high().ok();
    // Wait for the W5500 to achieve PLL lock.
    Timer::after(Duration::from_millis(2)).await;

    let mac = W5500::new(spi_dev, mac_addr).await.unwrap();

    let (runner, device) = ch::new(&mut state.ch_state, mac_addr);
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

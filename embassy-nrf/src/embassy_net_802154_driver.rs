//! embassy-net IEEE 802.15.4 driver

use embassy_futures::select::{Either3, select3};
use embassy_net_driver_channel::driver::LinkState;
use embassy_net_driver_channel::{self as ch};
use embassy_time::{Duration, Ticker};

use crate::radio::InterruptHandler;
use crate::radio::ieee802154::{Packet, Radio};
use crate::{self as nrf, interrupt};

/// MTU for the nrf radio.
pub const MTU: usize = Packet::CAPACITY as usize;

/// embassy-net device for the driver.
pub type Device<'d> = embassy_net_driver_channel::Device<'d, MTU>;

/// Internal state for the embassy-net driver.
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
pub struct Runner<'d> {
    radio: nrf::radio::ieee802154::Radio<'d>,
    ch: ch::Runner<'d, MTU>,
}

impl<'d> Runner<'d> {
    /// Drives the radio. Needs to run to use the driver.
    pub async fn run(mut self) -> ! {
        let (state_chan, mut rx_chan, mut tx_chan) = self.ch.split();
        let mut tick = Ticker::every(Duration::from_millis(500));
        let mut packet = Packet::new();
        state_chan.set_link_state(LinkState::Up);
        loop {
            match select3(
                async {
                    let rx_buf = rx_chan.rx_buf().await;
                    self.radio.receive(&mut packet).await.ok().map(|_| rx_buf)
                },
                tx_chan.tx_buf(),
                tick.next(),
            )
            .await
            {
                Either3::First(Some(rx_buf)) => {
                    let len = rx_buf.len().min(packet.len() as usize);
                    (&mut rx_buf[..len]).copy_from_slice(&*packet);
                    rx_chan.rx_done(len);
                }
                Either3::Second(tx_buf) => {
                    let len = tx_buf.len().min(Packet::CAPACITY as usize);
                    packet.copy_from_slice(&tx_buf[..len]);
                    self.radio.try_send(&mut packet).await.ok().unwrap();
                    tx_chan.tx_done();
                }
                _ => {}
            }
        }
    }
}

/// Make sure to use `HfclkSource::ExternalXtal` as the `hfclk_source`
/// to use the radio (nrf52840 product spec v1.11 5.4.1)
/// ```
/// # use embassy_nrf::config::*;
/// let mut config = Config::default();
/// config.hfclk_source = HfclkSource::ExternalXtal;
/// ```
pub async fn new<'a, const N_RX: usize, const N_TX: usize, T: nrf::radio::Instance, Irq>(
    mac_addr: [u8; 8],
    radio: nrf::Peri<'a, T>,
    irq: Irq,
    state: &'a mut State<N_RX, N_TX>,
) -> Result<(Device<'a>, Runner<'a>), ()>
where
    Irq: interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'a,
{
    let radio = Radio::new(radio, irq);

    let (runner, device) = ch::new(&mut state.ch_state, ch::driver::HardwareAddress::Ieee802154(mac_addr));

    Ok((device, Runner { ch: runner, radio }))
}

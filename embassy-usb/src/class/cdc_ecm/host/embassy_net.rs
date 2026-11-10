//! [`embassy-net`](https://crates.io/crates/embassy-net) driver for the CDC-ECM host class.

use embassy_futures::select::{Either, select};
use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::LinkState;
use embassy_usb_driver::host::UsbHostAllocator;

use super::{EcmHost, EcmHostReceiver, EcmHostSender};

/// Internal state for the embassy-net host integration.
pub struct State<const N_RX: usize, const N_TX: usize> {
    pub(crate) ch_state: ch::State<N_RX, N_TX>,
}

impl<const N_RX: usize, const N_TX: usize> State<N_RX, N_TX> {
    /// Create a new `State`.
    pub const fn new() -> Self {
        Self {
            ch_state: ch::State::new(),
        }
    }
}

impl<const N_RX: usize, const N_TX: usize> Default for State<N_RX, N_TX> {
    fn default() -> Self {
        Self::new()
    }
}

/// Background runner for the CDC-ECM host class.
///
/// You must call `.run()` in a background task for the class to operate.
pub struct Runner<'d, A: UsbHostAllocator<'d>> {
    pub(crate) tx_usb: EcmHostSender<'d, A>,
    pub(crate) rx_usb: EcmHostReceiver<'d, A>,
    pub(crate) ch: ch::Runner<'d>,
    pub(crate) mtu: usize,
}

impl<'d, A: UsbHostAllocator<'d>> Runner<'d, A> {
    /// Run the CDC-ECM host class.
    pub async fn run(mut self) -> ! {
        let (state_chan, mut rx_chan, mut tx_chan) = self.ch.split();
        state_chan.set_link_state(LinkState::Up);

        let rx_fut = async move {
            loop {
                let mut p = rx_chan.rx_buf().await;
                p.set_len(self.mtu.min(p.capacity()));
                match self.rx_usb.read_packet(&mut p).await {
                    Ok(n) => {
                        p.set_len(n);
                        rx_chan.rx(p).await;
                    }
                    Err(_) => {
                        state_chan.set_link_state(LinkState::Down);
                    }
                }
            }
        };

        let tx_fut = async move {
            loop {
                let p = tx_chan.tx().await;
                let _ = self.tx_usb.write_packet(&p).await;
            }
        };

        match select(rx_fut, tx_fut).await {
            Either::First(x) => x,
            Either::Second(x) => x,
        }
    }
}

/// Type alias for the embassy-net driver for CDC-ECM host.
pub type Device<'d> = embassy_net_driver_channel::Device<'d>;

impl<'d, A: UsbHostAllocator<'d>> EcmHost<'d, A> {
    /// Obtain an [`embassy-net`](https://crates.io/crates/embassy-net) network device driver.
    pub fn into_embassy_net_device<const N_RX: usize, const N_TX: usize>(
        self,
        state: &'d mut State<N_RX, N_TX>,
        ethernet_address: [u8; 6],
        mtu: usize,
    ) -> (Runner<'d, A>, Device<'d>) {
        let (tx_usb, rx_usb) = self.split();
        let (runner, device) = ch::new(
            &mut state.ch_state,
            ch::driver::HardwareAddress::Ethernet(ethernet_address),
            mtu,
        );

        (
            Runner {
                tx_usb,
                rx_usb,
                ch: runner,
                mtu,
            },
            device,
        )
    }
}

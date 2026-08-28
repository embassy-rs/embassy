//! [`embassy-net`](https://crates.io/crates/embassy-net) driver for the CDC-NCM class.

use embassy_futures::select::{Either, select};
use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::{LinkState, PacketBuf};
use embassy_usb_driver::Driver;

use super::{CdcNcmClass, Receiver, Sender};

/// Internal state for the embassy-net integration.
pub struct State<const N_RX: usize, const N_TX: usize> {
    ch_state: ch::State<N_RX, N_TX>,
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

/// Background runner for the CDC-NCM class.
///
/// You must call `.run()` in a background task for the class to operate.
pub struct Runner<'d, D: Driver<'d>> {
    tx_usb: Sender<'d, D>,
    rx_usb: Receiver<'d, D>,
    ch: ch::Runner<'d>,
    mtu: usize,
}

impl<'d, D: Driver<'d>> Runner<'d, D> {
    /// Run the CDC-NCM class.
    ///
    /// You must call this in a background task for the class to operate.
    pub async fn run(mut self) -> ! {
        let (state_chan, mut rx_chan, mut tx_chan) = self.ch.split();
        let rx_fut = async move {
            loop {
                trace!("WAITING for connection");
                state_chan.set_link_state(LinkState::Down);

                self.rx_usb.wait_connection().await.unwrap();

                trace!("Connected");
                state_chan.set_link_state(LinkState::Up);

                loop {
                    rx_chan.rx_ready().await;
                    let Some(mut p) = PacketBuf::try_new() else {
                        warn!("packet pool empty, can't receive");
                        // Back off, so we don't spin until the stack frees a buffer.
                        embassy_time::Timer::after_millis(1).await;
                        continue;
                    };
                    p.set_len(self.mtu);
                    match self.rx_usb.read_packet(&mut p).await {
                        Ok(n) => {
                            p.set_len(n);
                            rx_chan.rx(p).await;
                        }
                        Err(e) => {
                            warn!("error reading packet: {:?}", e);
                            break;
                        }
                    };
                }
            }
        };
        let tx_fut = async move {
            loop {
                let p = tx_chan.tx().await;
                if let Err(e) = self.tx_usb.write_packet(&p).await {
                    warn!("Failed to TX packet: {:?}", e);
                }
            }
        };
        match select(rx_fut, tx_fut).await {
            Either::First(x) => x,
            Either::Second(x) => x,
        }
    }
}

// would be cool to use a TAIT here, but it gives a "may not live long enough". rustc bug?
//pub type Device<'d> = impl embassy_net_driver_channel::driver::Driver + 'd;
/// Type alias for the embassy-net driver for CDC-NCM.
pub type Device<'d> = embassy_net_driver_channel::Device<'d>;

impl<'d, D: Driver<'d>> CdcNcmClass<'d, D> {
    /// Obtain a driver for using the CDC-NCM class with [`embassy-net`](https://crates.io/crates/embassy-net).
    pub fn into_embassy_net_device<const N_RX: usize, const N_TX: usize>(
        self,
        state: &'d mut State<N_RX, N_TX>,
        ethernet_address: [u8; 6],
        mtu: usize,
    ) -> (Runner<'d, D>, Device<'d>) {
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

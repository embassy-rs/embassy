#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

// must be first
mod fmt;

use core::convert::Infallible;
use core::mem::MaybeUninit;

use embassy_futures::select::{select, Either};
use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::LinkState;
use embedded_io_async::{BufRead, Write};
use ppproto::pppos::{BufferFullError, PPPoS, PPPoSAction};
pub use ppproto::{Config, Ipv4Status};

const MTU: usize = 1500;

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
pub struct Runner<'d> {
    ch: ch::Runner<'d, MTU>,
}

/// Error returned by [`Runner::run`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RunError<E> {
    /// Reading from the serial port failed.
    Read(E),
    /// Writing to the serial port failed.
    Write(E),
    /// Writing to the serial got EOF.
    Eof,
    /// PPP protocol was terminated by the peer
    Terminated,
}

impl<'d> Runner<'d> {
    /// You must call this in a background task for the driver to operate.
    ///
    /// If reading/writing to the underlying serial port fails, the link state
    /// is set to Down and the error is returned.
    ///
    /// It is allowed to cancel this function's future (i.e. drop it). This will terminate
    /// the PPP connection and set the link state to Down.
    ///
    /// After this function returns or is canceled, you can call it again to establish
    /// a new PPP connection.
    pub async fn run<RW: BufRead + Write>(
        &mut self,
        mut rw: RW,
        config: ppproto::Config<'_>,
        mut on_ipv4_up: impl FnMut(Ipv4Status),
    ) -> Result<Infallible, RunError<RW::Error>> {
        let mut ppp = PPPoS::new(config);
        ppp.open().unwrap();

        let (state_chan, mut rx_chan, mut tx_chan) = self.ch.borrow_split();
        state_chan.set_link_state(LinkState::Down);
        let _ondrop = OnDrop::new(|| state_chan.set_link_state(LinkState::Down));

        let mut rx_buf = [0; 2048];
        let mut tx_buf = [0; 2048];

        let mut needs_poll = true;
        let mut was_up = false;

        loop {
            let rx_fut = async {
                let buf = rx_chan.rx_buf().await;
                let rx_data = match needs_poll {
                    true => &[][..],
                    false => match rw.fill_buf().await {
                        Ok(rx_data) if rx_data.len() == 0 => return Err(RunError::Eof),
                        Ok(rx_data) => rx_data,
                        Err(e) => return Err(RunError::Read(e)),
                    },
                };
                Ok((buf, rx_data))
            };
            let tx_fut = tx_chan.tx_buf();
            match select(rx_fut, tx_fut).await {
                Either::First(r) => {
                    needs_poll = false;

                    let (buf, rx_data) = r?;
                    let n = ppp.consume(rx_data, &mut rx_buf);
                    rw.consume(n);

                    match ppp.poll(&mut tx_buf, &mut rx_buf) {
                        PPPoSAction::None => {}
                        PPPoSAction::Received(rg) => {
                            let pkt = &rx_buf[rg];
                            buf[..pkt.len()].copy_from_slice(pkt);
                            rx_chan.rx_done(pkt.len());
                        }
                        PPPoSAction::Transmit(n) => rw.write_all(&tx_buf[..n]).await.map_err(RunError::Write)?,
                    }

                    let status = ppp.status();
                    match status.phase {
                        ppproto::Phase::Dead => {
                            return Err(RunError::Terminated);
                        }
                        ppproto::Phase::Open => {
                            if !was_up {
                                on_ipv4_up(status.ipv4.unwrap());
                            }
                            was_up = true;
                            state_chan.set_link_state(LinkState::Up);
                        }
                        _ => {
                            was_up = false;
                            state_chan.set_link_state(LinkState::Down);
                        }
                    }
                }
                Either::Second(pkt) => {
                    match ppp.send(pkt, &mut tx_buf) {
                        Ok(n) => rw.write_all(&tx_buf[..n]).await.map_err(RunError::Write)?,
                        Err(BufferFullError) => unreachable!(),
                    }
                    tx_chan.tx_done();
                }
            }
        }
    }
}

/// Create a PPP embassy-net driver instance.
///
/// This returns two structs:
/// - a `Device` that you must pass to the `embassy-net` stack.
/// - a `Runner`. You must call `.run()` on it in a background task.
pub fn new<'a, const N_RX: usize, const N_TX: usize>(state: &'a mut State<N_RX, N_TX>) -> (Device<'a>, Runner<'a>) {
    let (runner, device) = ch::new(&mut state.ch_state, ch::driver::HardwareAddress::Ip);
    (device, Runner { ch: runner })
}

struct OnDrop<F: FnOnce()> {
    f: MaybeUninit<F>,
}

impl<F: FnOnce()> OnDrop<F> {
    fn new(f: F) -> Self {
        Self { f: MaybeUninit::new(f) }
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        unsafe { self.f.as_ptr().read()() }
    }
}

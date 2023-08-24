#![no_std]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

// must be first
mod fmt;

use core::convert::Infallible;
use core::mem::MaybeUninit;

use embassy_futures::select::{select3, Either3};
use embassy_net_driver_channel as ch;
use embassy_net_driver_channel::driver::LinkState;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embedded_io_async::{BufRead, Write, WriteAllError};
use ppproto::pppos::{BufferFullError, PPPoS, PPPoSAction};

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
pub struct Runner<'d, R: BufRead, W: Write> {
    ch: ch::Runner<'d, MTU>,
    r: R,
    w: W,
}

/// Error returned by [`Runner::run`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RunError<RE, WE> {
    /// Reading from the serial port failed.
    Read(RE),
    /// Writing to the serial port failed.
    Write(WE),
    /// Writing to the serial port wrote zero bytes, indicating it can't accept more data.
    WriteZero,
    /// Writing to the serial got EOF.
    Eof,
}

impl<'d, R: BufRead, W: Write> Runner<'d, R, W> {
    /// You must call this in a background task for the driver to operate.
    pub async fn run(mut self) -> Result<Infallible, RunError<R::Error, W::Error>> {
        let config = ppproto::Config {
            username: b"myuser",
            password: b"mypass",
        };
        let mut ppp = PPPoS::new(config);
        ppp.open().unwrap();

        let (state_chan, mut rx_chan, mut tx_chan) = self.ch.split();
        state_chan.set_link_state(LinkState::Down);
        let _ondrop = OnDrop::new(|| state_chan.set_link_state(LinkState::Down));

        let mut rx_buf = [0; 2048];
        let mut tx_buf = [0; 2048];

        let poll_signal: Signal<NoopRawMutex, ()> = Signal::new();
        poll_signal.signal(());

        loop {
            let mut poll = false;
            match select3(self.r.fill_buf(), tx_chan.tx_buf(), poll_signal.wait()).await {
                Either3::First(r) => {
                    let data = r.map_err(RunError::Read)?;
                    if data.is_empty() {
                        return Err(RunError::Eof);
                    }
                    let n = ppp.consume(data, &mut rx_buf);
                    self.r.consume(n);
                    poll = true;
                }
                Either3::Second(pkt) => {
                    match ppp.send(pkt, &mut tx_buf) {
                        Ok(n) => match self.w.write_all(&tx_buf[..n]).await {
                            Ok(()) => {}
                            Err(WriteAllError::WriteZero) => return Err(RunError::WriteZero),
                            Err(WriteAllError::Other(e)) => return Err(RunError::Write(e)),
                        },
                        Err(BufferFullError) => unreachable!(),
                    }
                    tx_chan.tx_done();
                }
                Either3::Third(_) => poll = true,
            }

            if poll {
                match ppp.poll(&mut tx_buf, &mut rx_buf) {
                    PPPoSAction::None => {}
                    PPPoSAction::Received(rg) => {
                        let pkt = &rx_buf[rg];
                        let buf = rx_chan.rx_buf().await; // TODO: fix possible deadlock
                        buf[..pkt.len()].copy_from_slice(pkt);
                        rx_chan.rx_done(pkt.len());

                        poll_signal.signal(());
                    }
                    PPPoSAction::Transmit(n) => {
                        match self.w.write_all(&tx_buf[..n]).await {
                            Ok(()) => {}
                            Err(WriteAllError::WriteZero) => return Err(RunError::WriteZero),
                            Err(WriteAllError::Other(e)) => return Err(RunError::Write(e)),
                        }
                        poll_signal.signal(());
                    }
                }

                match ppp.status().phase {
                    ppproto::Phase::Open => state_chan.set_link_state(LinkState::Up),
                    _ => state_chan.set_link_state(LinkState::Down),
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
pub fn new<'a, const N_RX: usize, const N_TX: usize, R: BufRead, W: Write>(
    state: &'a mut State<N_RX, N_TX>,
    r: R,
    w: W,
) -> (Device<'a>, Runner<'a, R, W>) {
    let (runner, device) = ch::new(&mut state.ch_state, ch::driver::HardwareAddress::Ip);
    (device, Runner { ch: runner, r, w })
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

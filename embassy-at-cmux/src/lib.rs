//! Implementation of 3GPP TS 27.010 based on
//! https://www.3gpp.org/ftp/tsg_t/tsg_t/tsgt_04/docs/pdfs/TP-99119.pdf

#![no_std]

mod fmt;
mod frame;

use core::cell::Cell;
use core::future::{poll_fn, Future};
use core::mem::MaybeUninit;
use core::task::Poll;

use embassy_futures::join::join;
use embassy_futures::select::{select, select_slice, Either};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::pipe::{Pipe, Reader, Writer};
use embassy_sync::signal::Signal;
use embassy_sync::waitqueue::AtomicWaker;
use embedded_io_async::{BufRead, Error, ErrorType, Read, Write};
use futures::FutureExt;
use heapless::Vec;

use crate::frame::{Frame, InformationType};

//   val   bit NAME   RX         TX
// 0x0001   0  FC     -          -
// 0x0002   1  RTC    107.DSR    108/2.DTR
// 0x0004   2  RTR    106.CTS    133.RFR / 105.RTS
// 0x0008   3  RFU1   -          -
// 0x0010   4  RFU2   -          -
// 0x0020   5  IC     125.RI     always 0
// 0x0040   6  DV     109.DCD    always 1
// 0x0080   7  B1     1 = signal break
// 0x0100   8  B2     reserved, always 0
// 0x0200   9  B3     reserved, always 0
// 0x0400  10  L1     |
// 0x0800  11  L2     | break length
// 0x1000  12  L3     | units of 200ms
// 0x2000  13  L4     |

struct Lines {
    rx: Cell<u16>,
    tx: Cell<u16>,
    hangup: Cell<bool>,
    hangup_mask: Cell<Option<(u16, u16)>>,
    hangup_waker: AtomicWaker,
}

impl Lines {
    const fn new() -> Self {
        Self {
            rx: Cell::new(0),
            tx: Cell::new(0),
            hangup: Cell::new(false),
            hangup_mask: Cell::new(None),
            hangup_waker: AtomicWaker::new(),
        }
    }

    fn check_hangup(&self) {
        if let Some((mask, val)) = self.hangup_mask.get() {
            if self.rx.get() & mask == val & mask {
                if !self.hangup.get() {
                    warn!("HANGUP detected!");
                    self.hangup_waker.wake();
                }
                self.hangup.set(true);
            }
        } else {
            self.hangup.set(false);
        }
    }
}

pub struct Mux<const N: usize, const BUF: usize> {
    tx: [Pipe<NoopRawMutex, BUF>; N],
    rx: [Pipe<NoopRawMutex, BUF>; N],
    lines: [Lines; N],
    line_status_updated: Signal<NoopRawMutex, ()>,
}

pub struct Channel<'a, const BUF: usize> {
    rx: Reader<'a, NoopRawMutex, BUF>,
    tx: Writer<'a, NoopRawMutex, BUF>,
    lines: &'a Lines,
    line_status_updated: &'a Signal<NoopRawMutex, ()>,
}

pub struct ChannelRx<'a, const BUF: usize> {
    rx: Reader<'a, NoopRawMutex, BUF>,
    lines: &'a Lines,
    line_status_updated: &'a Signal<NoopRawMutex, ()>,
}

pub struct ChannelTx<'a, const BUF: usize> {
    tx: Writer<'a, NoopRawMutex, BUF>,
    lines: &'a Lines,
    line_status_updated: &'a Signal<NoopRawMutex, ()>,
}

#[derive(Clone)]
pub struct ChannelLines<'a, const BUF: usize> {
    lines: &'a Lines,
    line_status_updated: &'a Signal<NoopRawMutex, ()>,
}

pub struct Runner<'a, const N: usize, const BUF: usize> {
    tx: [Reader<'a, NoopRawMutex, BUF>; N],
    rx: [Writer<'a, NoopRawMutex, BUF>; N],
    lines: &'a [Lines; N],
    line_status_updated: &'a Signal<NoopRawMutex, ()>,
}

impl<const N: usize, const BUF: usize> Mux<N, BUF> {
    const ONE_PIPE: Pipe<NoopRawMutex, BUF> = Pipe::new();

    pub const fn new() -> Self {
        const LINE: Lines = Lines::new();

        Self {
            rx: [Self::ONE_PIPE; N],
            tx: [Self::ONE_PIPE; N],
            lines: [LINE; N],
            line_status_updated: Signal::new(),
        }
    }

    pub fn start(&mut self) -> (Runner<'_, N, BUF>, [Channel<'_, BUF>; N]) {
        let mut chs = MaybeUninitArray::<_, N>::uninit();
        let mut runner_tx = MaybeUninitArray::<_, N>::uninit();
        let mut runner_rx = MaybeUninitArray::<_, N>::uninit();

        for (i, (tx, rx)) in self.tx.iter_mut().zip(self.rx.iter_mut()).enumerate() {
            let (rx_r, rx_w) = rx.split();
            let (tx_r, tx_w) = tx.split();
            let ch = Channel {
                rx: rx_r,
                tx: tx_w,
                lines: &self.lines[i],
                line_status_updated: &self.line_status_updated,
            };
            unsafe {
                chs.set(i, ch);
                runner_rx.set(i, rx_w);
                runner_tx.set(i, tx_r);
            }
        }
        let runner = Runner {
            rx: unsafe { runner_rx.assume_init() },
            tx: unsafe { runner_tx.assume_init() },
            lines: &self.lines,
            line_status_updated: &self.line_status_updated,
        };
        (runner, unsafe { chs.assume_init() })
    }
}

impl<'a, const N: usize, const BUF: usize> Runner<'a, N, BUF> {
    pub async fn run<R: BufRead, W: Write>(&mut self, mut port_r: R, mut port_w: W) -> ! {
        // Open channels
        for id in 0..(N as u8 + 1) {
            // Send open channel request
            debug!("open channel {}", id);
            let sabm = frame::Sabm { id };
            sabm.write(&mut port_w).await.unwrap();

            // Read response
            let mut header = frame::RxHeader::read(&mut port_r).await.unwrap();
            debug!("header {:?}", header);

            if let Err(e) = header.finalize().await {
                warn!("bad open channel resp: {:?}", e);
            }
        }

        // Set initial lines.
        for c in self.lines {
            c.tx.set(0x46)
        }
        self.line_status_updated.signal(());

        debug!("mux running");

        let tx_fut = async {
            let mut line_tx_sent = [0; N];
            loop {
                let mut futs: Vec<_, N> = Vec::new();
                for c in &mut self.tx {
                    let res = futs.push(c.fill_buf());
                    assert!(res.is_ok());
                }

                match select(select_slice(&mut futs), self.line_status_updated.wait()).await {
                    Either::First((buf, i)) => {
                        let frame = frame::Uih {
                            id: i as u8 + 1,
                            cr: frame::CR::Command,
                            information: &buf,
                        };
                        let len = frame.write(&mut port_w).await.unwrap();

                        drop(futs);

                        self.tx[i].consume(len);
                    }
                    Either::Second(()) => {
                        for i in 0..N {
                            let id = i as u8 + 1;
                            let lines = self.lines[i].tx.get();
                            if lines != line_tx_sent[i] {
                                line_tx_sent[i] = lines;

                                let b1 = (lines as u8 & 0x7F) << 1 | 1;
                                let b2 = ((lines >> 7) as u8 & 0x7F) << 1 | 1;

                                let frame = frame::Uih {
                                    id: 0,
                                    cr: frame::CR::Command,
                                    information: &[0xe3, 0x07, (id << 2) | 0x03, b1, b2],
                                };
                                frame.write(&mut port_w).await.unwrap();
                            }
                        }
                    }
                }
            }
        };

        let rx_fut = async {
            loop {
                let mut header = frame::RxHeader::read(&mut port_r).await.unwrap();

                if header.len > 0 {
                    if header.is_control() {
                        // control channel command
                        let mut buf = [0u8; 24];
                        let info = header.read_information(&mut buf).await.unwrap();

                        if info.is_command() {
                            let mut supported = true;

                            match info.typ() {
                                InformationType::MultiplexerCloseDown => {
                                    info!("The mobile station requested mux-mode termination");
                                }
                                InformationType::TestCommand => {
                                    debug!("Test command: {:?}", info.data);
                                }
                                InformationType::ModemStatusCommand if !info.data.is_empty() => {
                                    assert!(buf[2] & 3 == 3);
                                    let id = (buf[2] >> 2) as usize;

                                    let b1 = buf[3] >> 1;
                                    let b2 = buf[4] >> 1;
                                    let lines_rx = (b1 as u16) | (b2 as u16) << 7;
                                    let lines = &self.lines[id - 1];
                                    debug!("channel {:?} lines rx: {:02x} -> {:02x}", id, lines.rx.get(), lines_rx);
                                    lines.rx.set(lines_rx);
                                    lines.check_hangup();
                                }
                                InformationType::ModemStatusCommand if info.data.is_empty() => {
                                    error!("Modem status command, but no info");
                                }
                                n => {
                                    warn!("Unknown command {:?} from the control channel", n);

                                    // Send `InformationType::NonSupportedCommandResponse`

                                    supported = false;
                                }
                            }

                            if supported {
                                // acknowledge the command
                                frame::Uih {
                                    id: header.id(),
                                    cr: frame::CR::Response,
                                    information: &buf[..],
                                };
                            }
                        } else {
                            // received ack for a command
                            if info.typ() == InformationType::NonSupportedCommandResponse {
                                warn!("The mobile station didn't support the command sent");
                            } else {
                                debug!("Command acknowledged by the mobile station");
                            }
                        }
                    } else {
                        // data from logical channel
                        header.copy(&mut self.rx[header.id() as usize - 1]).await.unwrap();
                    }
                } else {
                    header.finalize().await.unwrap();
                }
            }
        };

        join(tx_fut, rx_fut).await;
        unreachable!()
    }
}

struct MaybeUninitArray<T, const N: usize>(MaybeUninit<[T; N]>);

impl<T, const N: usize> MaybeUninitArray<T, N> {
    fn uninit() -> Self {
        Self(MaybeUninit::uninit())
    }
    unsafe fn set(&mut self, i: usize, val: T) {
        (self.0.as_mut_ptr() as *mut T).add(i).write(val);
    }
    unsafe fn assume_init(self) -> [T; N] {
        self.0.assume_init()
    }
}

impl<'a, const BUF: usize> Channel<'a, BUF> {
    pub fn split(self) -> (ChannelRx<'a, BUF>, ChannelTx<'a, BUF>, ChannelLines<'a, BUF>) {
        (
            ChannelRx {
                rx: self.rx,
                lines: self.lines,
                line_status_updated: self.line_status_updated,
            },
            ChannelTx {
                tx: self.tx,
                lines: self.lines,
                line_status_updated: self.line_status_updated,
            },
            ChannelLines {
                lines: self.lines,
                line_status_updated: self.line_status_updated,
            },
        )
    }

    pub fn split_lines(&self) -> ChannelLines<'a, BUF> {
        ChannelLines {
            lines: self.lines,
            line_status_updated: self.line_status_updated,
        }
    }

    pub fn set_lines(&self, lines: u16) {
        self.lines.tx.set(lines);
        self.line_status_updated.signal(());
    }

    pub fn get_lines(&self) -> u16 {
        self.lines.rx.get()
    }

    pub fn set_hangup_detection(&self, mask: u16, val: u16) {
        self.lines.hangup_mask.set(Some((mask, val)));
        self.lines.check_hangup();
    }

    pub fn clear_hangup_detection(&self) {
        self.lines.hangup_mask.set(None);
        self.lines.check_hangup();
    }
}

impl<'a, const BUF: usize> ChannelRx<'a, BUF> {
    pub fn set_lines(&self, lines: u16) {
        self.lines.tx.set(lines);
        self.line_status_updated.signal(());
    }

    pub fn get_lines(&self) -> u16 {
        self.lines.rx.get()
    }
}

impl<'a, const BUF: usize> ChannelTx<'a, BUF> {
    pub fn set_lines(&self, lines: u16) {
        self.lines.tx.set(lines);
        self.line_status_updated.signal(());
    }

    pub fn get_lines(&self) -> u16 {
        self.lines.rx.get()
    }
}

impl<'a, const BUF: usize> ChannelLines<'a, BUF> {
    pub fn set_lines(&self, lines: u16) {
        self.lines.tx.set(lines);
        self.line_status_updated.signal(());
    }

    pub fn get_lines(&self) -> u16 {
        self.lines.rx.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChannelError {
    Hangup,
}

impl Error for ChannelError {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match self {
            Self::Hangup => embedded_io_async::ErrorKind::BrokenPipe,
        }
    }
}

impl<'a, const BUF: usize> ErrorType for Channel<'a, BUF> {
    type Error = ChannelError;
}

impl<'a, const BUF: usize> Read for Channel<'a, BUF> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        check_hangup(self.rx.read(buf), self.lines).await
    }
}

impl<'a, const BUF: usize> BufRead for Channel<'a, BUF> {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        check_hangup(self.rx.fill_buf(), self.lines).await
    }

    fn consume(&mut self, amt: usize) {
        self.rx.consume(amt)
    }
}

impl<'a, const BUF: usize> Write for Channel<'a, BUF> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        check_hangup(self.tx.write(buf), self.lines).await
    }
}

impl<'a, const BUF: usize> ErrorType for ChannelRx<'a, BUF> {
    type Error = ChannelError;
}

impl<'a, const BUF: usize> Read for ChannelRx<'a, BUF> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        check_hangup(self.rx.read(buf), self.lines).await
    }
}

impl<'a, const BUF: usize> BufRead for ChannelRx<'a, BUF> {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        check_hangup(self.rx.fill_buf(), self.lines).await
    }

    fn consume(&mut self, amt: usize) {
        self.rx.consume(amt)
    }
}

impl<'a, const BUF: usize> ErrorType for ChannelTx<'a, BUF> {
    type Error = ChannelError;
}

impl<'a, const BUF: usize> Write for ChannelTx<'a, BUF> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        check_hangup(self.tx.write(buf), self.lines).await
    }
}

fn wait_for_hangup(lines: &Lines) -> impl Future<Output = ()> + '_ {
    poll_fn(|cx| {
        if lines.hangup.get() {
            Poll::Ready(())
        } else {
            lines.hangup_waker.register(cx.waker());
            Poll::Pending
        }
    })
}

fn check_hangup<'a, F, R>(fut: F, lines: &'a Lines) -> impl Future<Output = Result<R, ChannelError>> + 'a
where
    F: Future<Output = R> + 'a,
{
    select(fut, wait_for_hangup(lines)).map(|e| match e {
        Either::First(r) => Ok(r),
        Either::Second(()) => Err(ChannelError::Hangup),
    })
}

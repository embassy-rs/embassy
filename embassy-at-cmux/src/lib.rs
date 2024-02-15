#![no_std]
#![feature(inline_const)]

mod fmt;

use core::cell::Cell;
use core::future::{poll_fn, Future};
use core::mem::MaybeUninit;
use core::task::Poll;

use embassy_futures::select::{select, select_slice, Either};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::pipe::{Pipe, Reader, Writer};
use embassy_sync::signal::Signal;
use embassy_sync::waitqueue::AtomicWaker;
use embedded_io_async::{BufRead, Error, ErrorType, Read, Write};
use futures::FutureExt;
use heapless::Vec;

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
    pub fn new() -> Self {

        Self {
            rx: [const { Pipe::new() }; N],
            tx: [const { Pipe::new() }; N],
            lines: [const {
                Lines {
                    rx: Cell::new(0),
                    tx: Cell::new(0),
                    hangup: Cell::new(false),
                    hangup_mask: Cell::new(None),
                    hangup_waker: AtomicWaker::new(),
                }
            }; N],
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
    pub async fn run<R: BufRead, W: Write>(mut self, mut port_r: R, mut port_w: W) {
        // Open channels
        for id in 0..(N + 1) {
            debug!("open channel {}", id);

            // Send open channel request
            let buf = &mut [0xf9, (id as u8) << 2 | 3, 0x3F, 0x01, 0, 0xf9];
            buf[4] = crc(&buf[1..4]);
            port_w.write_all(buf).await.unwrap();

            // Read response
            let mut resp = [0xf9; 4];
            while resp[0] == 0xf9 {
                read_exact(&mut port_r, &mut resp[..1]).await.unwrap();
            }
            read_exact(&mut port_r, &mut resp[1..]).await.unwrap();

            let mut expected = [(id as u8) << 2 | 3, 0x73, 0x01, 0x00];
            expected[3] = crc(&expected[..3]);

            if resp != expected {
                warn!("bad open channel resp. expected {:02x}, got {:02x}", expected, resp);
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
                        let id = i + 1;

                        // max 127 bytes, because that's what fits in a packet.
                        let len = buf.len().min(127);
                        let buf = &buf[..len];

                        write_packet(&mut port_w, id, buf).await.unwrap();

                        drop(futs);

                        self.tx[i].consume(len);
                    }
                    Either::Second(()) => {
                        for i in 0..N {
                            let id = i + 1;
                            let lines = self.lines[i].tx.get();
                            if lines != line_tx_sent[i] {
                                line_tx_sent[i] = lines;

                                let b1 = (lines as u8 & 0x7F) << 1 | 1;
                                let b2 = ((lines >> 7) as u8 & 0x7F) << 1 | 1;
                                let packet = [0xe3, 0x07, ((id as u8) << 2) | 0x03, b1, b2];
                                write_packet(&mut port_w, 0, &packet).await.unwrap();
                            }
                        }
                    }
                }
            }
        };

        let rx_fut = async {
            loop {
                let mut header = [0xf9; 3];
                while header[0] == 0xf9 {
                    read_exact(&mut port_r, &mut header[..1]).await.unwrap();
                }
                read_exact(&mut port_r, &mut header[1..]).await.unwrap();

                let id = (header[0] >> 2) as usize;
                let len = (header[2] >> 1) as usize;

                if id == 0 {
                    let mut buf = [0; 16];
                    read_exact(&mut port_r, &mut buf[..len]).await.unwrap();

                    // check length
                    assert!(buf[1] & 1 == 1);
                    let inner_len = (buf[1] >> 1) as usize;
                    assert!(len == 2 + inner_len);

                    match buf[0] {
                        // Modem Status Command
                        0xE3 => {
                            assert!(buf[2] & 3 == 3);
                            assert_eq!(inner_len, 3);
                            let id = (buf[2] >> 2) as usize;

                            let b1 = buf[3] >> 1;
                            let b2 = buf[4] >> 1;
                            let lines_rx = (b1 as u16) | (b2 as u16) << 7;
                            let lines = &self.lines[id - 1];
                            debug!("channel {:?} lines rx: {:02x} -> {:02x}", id, lines.rx.get(), lines_rx);
                            lines.rx.set(lines_rx);
                            lines.check_hangup();
                        }
                        // Modem Status Response
                        0xE1 => {}
                        n => warn!("Unknown control {:?}", n),
                    }
                } else {
                    copy(&mut port_r, &mut self.rx[id - 1], len).await.unwrap();
                }

                let mut trailer = [0; 1];
                read_exact(&mut port_r, &mut trailer).await.unwrap();

                if trailer[0] != crc(&header) {
                    warn!("bad crc");
                }
            }
        };

        select(tx_fut, rx_fut).await;
    }
}

async fn write_packet<W: Write>(w: &mut W, id: usize, buf: &[u8]) -> Result<(), W::Error> {
    let header = [0xf9, (id as u8) << 2 | 1, 0xEF, (buf.len() as u8) << 1 | 1];
    let trailer = [crc(&header[1..4]), 0xf9];
    w.write_all(&header).await?;
    w.write_all(&buf).await?;
    w.write_all(&trailer).await?;
    Ok(())
}

async fn read_exact<R: BufRead>(r: &mut R, mut data: &mut [u8]) -> Result<(), R::Error> {
    while !data.is_empty() {
        let buf = r.fill_buf().await?;
        if buf.is_empty() {
            panic!("EOF");
        }
        let n = buf.len().min(data.len());
        data[..n].copy_from_slice(&buf[..n]);
        data = &mut data[n..];
        r.consume(n);
    }
    Ok(())
}

async fn copy<R: BufRead, W: Write>(r: &mut R, w: &mut W, mut len: usize) -> Result<(), R::Error> {
    while len != 0 {
        let buf = r.fill_buf().await?;
        if buf.is_empty() {
            panic!("EOF");
        }
        let n = buf.len().min(len);
        let n = w.write(&buf[..n]).await.unwrap();
        if n == 0 {
            panic!("Write zero!");
        }
        r.consume(n);
        len -= n;
    }
    Ok(())
}

fn crc(buf: &[u8]) -> u8 {
    let mut s = 0xFF;

    for &b in buf {
        s ^= b;
        for _ in 0..8 {
            if (s & 0x01) != 0 {
                s = (s >> 1) ^ 0xe0;
            } else {
                s = s >> 1;
            }
        }
    }

    return 0xFF - s;
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

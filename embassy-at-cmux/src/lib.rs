//! Implementation of 3GPP TS 27.010 based on
//! https://www.3gpp.org/ftp/tsg_t/tsg_t/tsgt_04/docs/pdfs/TP-99119.pdf

#![cfg_attr(not(test), no_std)]

mod fmt;
mod frame;

use core::cell::Cell;
use core::future::{poll_fn, Future};
use core::mem::MaybeUninit;
use core::pin::pin;
use core::task::Poll;

use embassy_futures::select::{select, select3, select_slice, Either, Either3};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::pipe::{Pipe, Reader, Writer};
use embassy_sync::signal::Signal;
use embassy_sync::waitqueue::AtomicWaker;
use embassy_time::{Duration, Instant, Timer};
use embedded_io_async::{BufRead, ErrorType, Read, Write};
pub use frame::{Break, Control};
use frame::{Frame, Information, MultiplexerCloseDown};
use futures::FutureExt;
use heapless::Vec;

use crate::frame::{Error, FrameType, NonSupportedCommandResponse};

const MAX_PINGS: u8 = 3;

struct Lines {
    rx: Cell<(Control, Option<Break>)>,
    tx: Cell<(Control, Option<Break>)>,
    opened: Cell<bool>,
    hangup: Cell<bool>,
    hangup_mask: Cell<Option<(u16, u16)>>,
    hangup_waker: AtomicWaker,
}

impl Lines {
    const fn new() -> Self {
        Self {
            rx: Cell::new((Control::new(), None)),
            tx: Cell::new((Control::new(), None)),
            opened: Cell::new(false),
            hangup: Cell::new(false),
            hangup_mask: Cell::new(None),
            hangup_waker: AtomicWaker::new(),
        }
    }

    fn check_hangup(&self) {
        // if let Some((mask, val)) = self.hangup_mask.get() {
        if !self.rx.get().0.dv() {
            if !self.hangup.get() {
                warn!("HANGUP detected!");
                // self.hangup_waker.wake();
            }
            // self.hangup.set(true);
        }
        // } else {
        //     self.hangup.set(false);
        // }
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
    control_channel_opened: bool,
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
            control_channel_opened: false,
            lines: &self.lines,
            line_status_updated: &self.line_status_updated,
        };
        (runner, unsafe { chs.assume_init() })
    }
}

impl<'a, const N: usize, const BUF: usize> Runner<'a, N, BUF> {
    pub async fn run<R: BufRead, W: Write>(
        &mut self,
        mut port_r: R,
        mut port_w: W,
        max_frame_size: usize,
    ) -> Result<(), Error> {
        if !self.control_channel_opened {
            debug!("Opening control channel");

            // Send open channel request
            frame::Sabm { id: 0 }.write(&mut port_w).await?;
        }

        for channel_id in 0..N {
            if !self.lines[channel_id].opened.get() {
                debug!("Opening channel {}", channel_id);

                // Send open channel request
                frame::Sabm {
                    id: channel_id as u8 + 1,
                }
                .write(&mut port_w)
                .await?;
            }
        }

        let mut last_received = Instant::now();
        let mut ping_number = 1u8;

        loop {
            let mut futs: Vec<_, N> = Vec::new();
            for c in &mut self.tx {
                let res = futs.push(c.fill_buf());
                assert!(res.is_ok());
            }

            let ping_fut = Timer::at(last_received + Duration::from_secs(5 * ping_number as u64));

            match select3(
                select_slice(pin!(&mut futs)),
                frame::RxHeader::read(&mut port_r),
                ping_fut,
            )
            .await
            {
                Either3::First((buf, i)) => {
                    // let (control, _) = self.lines[i].tx.get();
                    // if control.fc() {
                    //     warn!("Channel {} TX flow controlled!", i + 1);
                    //     continue;
                    // }

                    if !self.lines[i].opened.get() {
                        continue;
                    }

                    let len = buf.len().min(max_frame_size);

                    let frame = frame::Uih {
                        id: i as u8 + 1,
                        information: Information::Data(&buf[..len]),
                    };

                    frame.write(&mut port_w).await?;

                    drop(futs);

                    self.tx[i].consume(len);
                }

                Either3::Second(Err(e)) => {
                    error!("Got error while searching for RX header: {:?}", e);
                    continue;
                }
                Either3::Second(Ok(mut header)) => {
                    trace!("{:?}", header);

                    last_received = Instant::now();

                    match header.frame_type {
                        FrameType::Ui | FrameType::Uih if header.is_control() => {
                            let info = header.read_information().await?;

                            if info.is_command() {
                                let mut supported = true;

                                match &info {
                                    Information::MultiplexerCloseDown(_cld) => {
                                        info!("The mobile station requested mux-mode termination");
                                        break;
                                    }
                                    Information::TestCommand => {
                                        debug!("Test command");
                                    }
                                    Information::ModemStatusCommand(msc) => {
                                        let lines = &self.lines[msc.dlci as usize - 1];
                                        let new_control = msc.control.with_ea(false);
                                        let new_brk = msc.brk.map(|b| b.with_ea(false));
                                        debug!(
                                            "channel {:?} lines rx: {:?} -> {:?}",
                                            msc.dlci,
                                            lines.rx.get(),
                                            (new_control, new_brk)
                                        );

                                        // Modem is telling us something about
                                        // channel `msc.dlci`.
                                        //
                                        // We need to ack this message by sending
                                        // `MSC` with the same payload, but
                                        // `CR::Response`.
                                        // lines.rx.set((new_control, new_brk));
                                        // self.line_status_updated.signal(());
                                        // lines.check_hangup();
                                    }
                                    n => {
                                        warn!("Unknown command {:?} for the control channel", n);

                                        // Send `InformationType::NonSupportedCommandResponse`
                                        frame::Uih {
                                            id: 0,
                                            information: Information::NonSupportedCommandResponse(
                                                NonSupportedCommandResponse {
                                                    cr: frame::CR::Response,
                                                    command_type: n.info_type(),
                                                },
                                            ),
                                        }
                                        .write(&mut port_w)
                                        .await?;

                                        supported = false;
                                    }
                                }

                                if supported {
                                    // acknowledge the command
                                    info.send_ack(&mut port_w).await?;
                                }
                            } else {
                                // received ack for a command
                                if let Information::NonSupportedCommandResponse(NonSupportedCommandResponse {
                                    command_type,
                                    ..
                                }) = info
                                {
                                    warn!(
                                        "The mobile station didn't support the command sent ({:?})",
                                        command_type
                                    );
                                } else {
                                    debug!("Command acknowledged by the mobile station");
                                }
                            }
                        }
                        FrameType::Ui | FrameType::Uih => {
                            // data from logical channel
                            let channel_id = header.id() as usize - 1;

                            // TODO: Set flow control bits on buffer full here?
                            // let lines = &self.lines[id];
                            // let (ctrl, brk) = lines.tx.get();
                            // lines.tx.set((ctrl.with_fc(true), brk));
                            // self.line_status_updated.signal(());

                            header.copy(&mut self.rx[channel_id]).await?;
                        }
                        FrameType::Sabm if header.is_control() => {
                            // channel open request
                            if self.control_channel_opened {
                                info!("Received SABM even though control channel was already open.");
                            } else {
                                info!("Control channel opened.");
                            }
                            self.control_channel_opened = true;
                            frame::Ua { id: 0 }.write(&mut port_w).await?;
                        }
                        FrameType::Sabm => {
                            let channel_id = header.id() as usize - 1;
                            if self.lines[channel_id].opened.get() {
                                info!("Received SABM even though channel {} was already open.", channel_id);
                            } else {
                                info!("Logical channel {} opened.", channel_id);
                            }
                            self.lines[channel_id].opened.set(true);
                            frame::Ua { id: header.id() }.write(&mut port_w).await?;
                        }
                        FrameType::Ua if header.is_control() => {
                            if self.control_channel_opened {
                                self.control_channel_opened = false;
                                info!("Control channel closed.");
                            } else {
                                self.control_channel_opened = true;
                                info!("Control channel opened.");

                                // send version Siemens version test
                                // frame::Uih {
                                //     id: 0,
                                //     information: Information::Data(b"\x23\x21\x04TEMUXVERSION2\0\0"),
                                // }
                                // .write(&mut port_w)
                                // .await?;
                            }
                        }
                        FrameType::Ua => {
                            let channel_id = header.id() as usize - 1;
                            if self.lines[channel_id].opened.get() {
                                info!("Logical channel {} closed.", channel_id);
                                self.lines[channel_id].opened.set(false);
                            } else {
                                info!("Logical channel {} opened.", channel_id);

                                self.lines[channel_id].opened.set(true);
                            }
                        }
                        FrameType::Dm if header.is_control() => {
                            info!("Couldn't open control channel. -> Terminating MUX");
                            break;
                        }
                        FrameType::Dm => {
                            info!("Logical channel {} couldn't be opened.", header.id() - 1);
                        }
                        FrameType::Disc if header.is_control() => {
                            if self.control_channel_opened {
                                info!("Control channel closed.");
                                self.control_channel_opened = false;
                                frame::Ua { id: 0 }.write(&mut port_w).await?;
                                break;
                            } else {
                                info!("Received DISC even though control channel was already closed.");
                                frame::Dm { id: 0 }.write(&mut port_w).await?;
                            }
                        }
                        FrameType::Disc => {
                            let channel_id = header.id() as usize - 1;
                            if self.lines[channel_id].opened.get() {
                                self.lines[channel_id].opened.set(false);
                                info!("Logical channel {} closed.", channel_id);
                                frame::Ua { id: header.id() }.write(&mut port_w).await?;
                            } else {
                                info!("Received DISC even though channel {} was already closed.", channel_id);
                                frame::Dm { id: header.id() }.write(&mut port_w).await?;
                            }
                        }
                    }

                    header.finalize().await?;
                }
                Either3::Third(_) if ping_number >= MAX_PINGS => {}
                Either3::Third(_) => {
                    // Nothing has been received for a while -> test the modem
                    debug!("Sending PING to the modem.");
                    // frame::Uih {
                    //     id: 0,
                    //     information: Information::Data(b"\x23\x09PING"),
                    // }
                    // .write(&mut port_w)
                    // .await?;
                    ping_number += 1;
                }
            }
        }

        for id in (0..N).rev() {
            let channel_id = id + 1;
            info!("Closing down the logical channel {}.", channel_id);
            if self.lines[channel_id].opened.get() {
                frame::Disc { id: channel_id as u8 }.write(&mut port_w).await?;
            }
        }

        if self.control_channel_opened {
            info!("Sending close down request to the multiplexer.");
            frame::Uih {
                id: 0,
                information: Information::MultiplexerCloseDown(MultiplexerCloseDown { cr: frame::CR::Command }),
            }
            .write(&mut port_w)
            .await?;
        }

        Ok(())
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

    pub fn set_lines(&self, control: Control, brk: Option<Break>) {
        self.lines.tx.set((control, brk));
        self.line_status_updated.signal(());
    }

    pub fn get_lines(&self) -> (Control, Option<Break>) {
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
    pub fn set_lines(&self, control: Control, brk: Option<Break>) {
        self.lines.tx.set((control, brk));
        self.line_status_updated.signal(());
    }

    pub fn get_lines(&self) -> (Control, Option<Break>) {
        self.lines.rx.get()
    }
}

impl<'a, const BUF: usize> ChannelTx<'a, BUF> {
    pub fn set_lines(&self, control: Control, brk: Option<Break>) {
        self.lines.tx.set((control, brk));
        self.line_status_updated.signal(());
    }

    pub fn get_lines(&self) -> (Control, Option<Break>) {
        self.lines.rx.get()
    }
}

impl<'a, const BUF: usize> ChannelLines<'a, BUF> {
    pub fn set_lines(&self, control: Control, brk: Option<Break>) {
        self.lines.tx.set((control, brk));
        self.line_status_updated.signal(());
    }

    pub fn get_lines(&self) -> (Control, Option<Break>) {
        self.lines.rx.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChannelError {
    Hangup,
}

impl embedded_io_async::Error for ChannelError {
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

//! Direct Memory Access (DMA)
use core::future::{poll_fn, Future};
use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};

use embassy_hal_common::{impl_peripheral, into_ref, Peripheral, PeripheralRef};
use embassy_sync::waitqueue::AtomicWaker;
use pac::dma::regs::CtrlTrig;
use pac::dma::vals::{DataSize, TreqSel};

use crate::interrupt::InterruptExt;
use crate::pac::dma::vals;
use crate::{interrupt, pac, peripherals};

#[cfg(feature = "rt")]
#[interrupt]
fn DMA_IRQ_0() {
    let ints0 = pac::DMA.ints0().read().ints0();
    for channel in 0..CHANNEL_COUNT {
        let ctrl_trig = pac::DMA.ch(channel).ctrl_trig().read();
        if ctrl_trig.ahb_error() {
            panic!("DMA: error on DMA_0 channel {}", channel);
        }

        if ints0 & (1 << channel) == (1 << channel) {
            CHANNEL_WAKERS[channel].wake();
        }
    }
    pac::DMA.ints0().write(|w| w.set_ints0(ints0));
}

pub(crate) unsafe fn init() {
    interrupt::DMA_IRQ_0.disable();
    interrupt::DMA_IRQ_0.set_priority(interrupt::Priority::P3);

    pac::DMA.inte0().write(|w| w.set_inte0(0xFFFF));

    interrupt::DMA_IRQ_0.enable();
}

pub unsafe fn read<'a, C: Channel, W: Word>(
    ch: impl Peripheral<P = C> + 'a,
    from: *const W,
    to: *mut [W],
    dreq: u8,
) -> Transfer<'a, C> {
    let (to_ptr, len) = crate::dma::slice_ptr_parts(to);
    copy_inner(
        ch,
        from as *const u32,
        to_ptr as *mut u32,
        len,
        W::size(),
        false,
        true,
        dreq,
    )
}

pub unsafe fn write<'a, C: Channel, W: Word>(
    ch: impl Peripheral<P = C> + 'a,
    from: *const [W],
    to: *mut W,
    dreq: u8,
) -> Transfer<'a, C> {
    let (from_ptr, len) = crate::dma::slice_ptr_parts(from);
    copy_inner(
        ch,
        from_ptr as *const u32,
        to as *mut u32,
        len,
        W::size(),
        true,
        false,
        dreq,
    )
}

static DUMMY: u32 = 0;

pub unsafe fn write_repeated<'a, C: Channel, W: Word>(
    ch: impl Peripheral<P = C> + 'a,
    to: *mut W,
    len: usize,
    dreq: u8,
) -> Transfer<'a, C> {
    copy_inner(
        ch,
        &DUMMY as *const u32,
        to as *mut u32,
        len,
        W::size(),
        false,
        false,
        dreq,
    )
}

pub unsafe fn copy<'a, C: Channel, W: Word>(
    ch: impl Peripheral<P = C> + 'a,
    from: &[W],
    to: &mut [W],
) -> Transfer<'a, C> {
    let (from_ptr, from_len) = crate::dma::slice_ptr_parts(from);
    let (to_ptr, to_len) = crate::dma::slice_ptr_parts_mut(to);
    assert_eq!(from_len, to_len);
    copy_inner(
        ch,
        from_ptr as *const u32,
        to_ptr as *mut u32,
        from_len,
        W::size(),
        true,
        true,
        vals::TreqSel::PERMANENT.0,
    )
}

fn copy_inner<'a, C: Channel>(
    ch: impl Peripheral<P = C> + 'a,
    from: *const u32,
    to: *mut u32,
    len: usize,
    data_size: DataSize,
    incr_read: bool,
    incr_write: bool,
    dreq: u8,
) -> Transfer<'a, C> {
    into_ref!(ch);

    let p = ch.regs();

    p.read_addr().write_value(from as u32);
    p.write_addr().write_value(to as u32);
    p.trans_count().write_value(len as u32);

    compiler_fence(Ordering::SeqCst);

    p.ctrl_trig().write(|w| {
        // TODO: Add all DREQ options to pac vals::TreqSel, and use
        // `set_treq:sel`
        w.0 = ((dreq as u32) & 0x3f) << 15usize;
        w.set_data_size(data_size);
        w.set_incr_read(incr_read);
        w.set_incr_write(incr_write);
        w.set_chain_to(ch.number());
        w.set_en(true);
    });

    compiler_fence(Ordering::SeqCst);
    Transfer::new(ch)
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Transfer<'a, C: Channel> {
    channel: PeripheralRef<'a, C>,
}

impl<'a, C: Channel> Transfer<'a, C> {
    pub(crate) fn new(channel: impl Peripheral<P = C> + 'a) -> Self {
        into_ref!(channel);

        Self { channel }
    }
}

impl<'a, C: Channel> Drop for Transfer<'a, C> {
    fn drop(&mut self) {
        let p = self.channel.regs();
        pac::DMA
            .chan_abort()
            .modify(|m| m.set_chan_abort(1 << self.channel.number()));
        while p.ctrl_trig().read().busy() {}
    }
}

impl<'a, C: Channel> Unpin for Transfer<'a, C> {}
impl<'a, C: Channel> Future for Transfer<'a, C> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // We need to register/re-register the waker for each poll because any
        // calls to wake will deregister the waker.
        CHANNEL_WAKERS[self.channel.number() as usize].register(cx.waker());

        if self.channel.regs().ctrl_trig().read().busy() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

pub enum Read<'a, W: Word> {
    Constant(&'a W),
    Increase(&'a [W]),
    // TODO ring also possible, but more complicated due to generic size and alignment requirements
}

impl<'a, W: Word> Read<'a, W> {
    fn is_increase(&self) -> bool {
        match *self {
            Self::Constant(_) => false,
            Self::Increase(_) => true,
        }
    }

    fn address(&self) -> u32 {
        match *self {
            Self::Constant(w) => (w as *const W) as u32,
            Self::Increase(w) => w.as_ptr() as u32,
        }
    }

    fn forward(&mut self, n: usize) -> () {
        if let Self::Increase(w) = *self {
            *self = Self::Increase(&w[n..]);
        }
    }
}

struct InnerChannels<'a, C1: Channel, C2: Channel> {
    data: PeripheralRef<'a, C1>,
    control: PeripheralRef<'a, C2>,
}

impl<'a, C1: Channel, C2: Channel> Drop for InnerChannels<'a, C1, C2> {
    fn drop(&mut self) {
        pac::DMA
            .chan_abort()
            .modify(|m| m.set_chan_abort((1 << self.data.number()) | (1 << self.control.number())));

        // wait until both channels are ready again, this should go quite fast so no async used here
        while self.data.regs().ctrl_trig().read().busy() || self.control.regs().ctrl_trig().read().busy() {}
    }
}

pub struct ContinuousTransfer<'a, 'b, 'c, 'r, W: Word, C1: Channel, C2: Channel> {
    channels: InnerChannels<'a, C1, C2>,
    #[allow(dead_code)] // reference is kept to signal that dma channels are writing to it
    buffer: &'b mut [W],
    control_input: &'c mut [[u32; 4]; 2],
    dreq: TreqSel,
    read: Read<'r, W>,
}

impl<'a, 'b, 'c, 'r, W: Word, C1: Channel, C2: Channel> ContinuousTransfer<'a, 'b, 'c, 'r, W, C1, C2> {
    pub fn start_new(
        ch1: PeripheralRef<'a, C1>,
        ch2: PeripheralRef<'a, C2>,
        control_input: &'c mut [[u32; 4]; 2],
        buffer: &'b mut [W],
        dreq: TreqSel,
        mut read: Read<'r, W>,
    ) -> ContinuousTransfer<'a, 'b, 'c, 'r, W, C1, C2> {
        let channels = InnerChannels {
            data: ch1,
            control: ch2,
        };

        // configure what control channel writes
        // using registers: READ_ADDR, WRITE_ADDR, TRANS_COUNT, CTRL_TRIG
        let mut w = CtrlTrig(0);
        w.set_treq_sel(dreq);
        w.set_data_size(W::size());
        w.set_incr_read(read.is_increase());
        w.set_incr_write(true);
        w.set_chain_to(channels.data.number()); // chain disabled by default
        w.set_en(true);
        w.set_irq_quiet(false);

        *control_input = [
            [read.address(), buffer.as_ptr() as u32, buffer.len() as u32, w.0], // first control write
            [0; 4],                                                             // Null trigger to stop
        ];

        // Configure data channel
        // will be set by control channel
        let pd = channels.data.regs();
        pd.read_addr().write_value(0);
        pd.write_addr().write_value(0);
        pd.trans_count().write_value(0);
        pd.al1_ctrl().write_value(0);

        // Configure control channel
        let pc = channels.control.regs();
        pc.write_addr().write_value(pd.read_addr().as_ptr() as u32);
        pc.read_addr().write_value(control_input.as_ptr() as u32);
        pc.trans_count().write_value(4); // each control input is 4 u32s long

        // trigger control channel
        compiler_fence(Ordering::SeqCst);
        pc.ctrl_trig().write(|w| {
            w.set_treq_sel(TreqSel::PERMANENT);
            w.set_data_size(rp_pac::dma::vals::DataSize::SIZE_WORD); // 4 byte
            w.set_incr_read(true); // step through control_input
            w.set_incr_write(true); // yes, but ring is required
            w.set_ring_sel(true); // wrap write addresses
            w.set_ring_size(4); // 1 << 4 = 16 = 4 * sizeof(u32) bytes
            w.set_chain_to(channels.control.number()); // disable chain, data channel is triggered by write
            w.set_irq_quiet(false);
            w.set_en(true);
        });
        compiler_fence(Ordering::SeqCst);

        // wait until control ran
        while pc.ctrl_trig().read().busy() {}

        // reset control
        control_input[0] = [0; 4];
        pc.read_addr().write_value(control_input.as_ptr() as u32);

        read.forward(buffer.len());

        ContinuousTransfer {
            channels,
            buffer,
            control_input,
            dreq,
            read,
        }
    }

    pub async fn next<'new_buf>(
        self,
        buffer: &'new_buf mut [W],
    ) -> (ContinuousTransfer<'a, 'new_buf, 'c, 'r, W, C1, C2>, bool) {
        let ContinuousTransfer {
            channels,
            buffer: _old, // is free now, and the compiler knows it
            control_input,
            dreq,
            mut read,
        } = self;

        let pc = channels.control.regs();
        let pd = channels.data.regs();

        let mut w = CtrlTrig(0);
        w.set_treq_sel(dreq);
        w.set_data_size(W::size());
        w.set_incr_read(read.is_increase());
        w.set_incr_write(true);
        w.set_chain_to(channels.data.number()); // chain disabled by default
        w.set_en(true);
        w.set_irq_quiet(false);

        // configure control
        control_input[0] = [read.address(), buffer.as_ptr() as u32, buffer.len() as u32, w.0];

        // enable chain, now we can't change control safely anymore
        compiler_fence(Ordering::SeqCst);
        pd.al1_ctrl().write_value({
            let mut ctrl = pd.ctrl_trig().read();
            ctrl.set_chain_to(channels.control.number());
            ctrl.0
        });

        if pc.read_addr().read() == control_input.as_ptr() as u32 && pd.ctrl_trig().read().busy() {
            poll_fn(|cx: &mut Context<'_>| {
                // the more efficient solution would be to use the interrupts,
                // but I was not able to get it working robustly
                cx.waker().wake_by_ref();
                if pc.read_addr().read() == control_input.as_ptr() as u32 + 16 {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;

            // reset control
            assert!(!pc.ctrl_trig().read().busy());
            control_input[0] = [0; 4];
            pc.read_addr().write_value(control_input.as_ptr() as u32);

            read.forward(buffer.len());

            (
                ContinuousTransfer {
                    channels,
                    buffer,
                    control_input,
                    dreq,
                    read,
                },
                true,
            )
        } else {
            if pc.read_addr().read() == control_input.as_ptr() as u32 {
                // trigger control to restart loop
                pc.ctrl_trig().write_value(pc.ctrl_trig().read());
                compiler_fence(Ordering::SeqCst);
            }
            // if control read already moved, data has already been activated

            // wait for control to complete
            while pc.ctrl_trig().read().busy() {}
            // reset control
            control_input[0] = [0; 4];
            pc.read_addr().write_value(control_input.as_ptr() as u32);

            read.forward(buffer.len());

            (
                ContinuousTransfer {
                    channels,
                    control_input,
                    buffer,
                    dreq,
                    read,
                },
                false,
            )
        }
    }

    pub async fn stop(self) {
        // when no longer enabling the chain, data simply stops
        poll_fn(|cx| {
            // using interrupts would be nicer
            cx.waker().wake_by_ref();
            if self.channels.data.regs().ctrl_trig().read().busy() {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        })
        .await;
    }

    pub fn abort(self) {} // drop channels
}

pub(crate) const CHANNEL_COUNT: usize = 12;
const NEW_AW: AtomicWaker = AtomicWaker::new();
static CHANNEL_WAKERS: [AtomicWaker; CHANNEL_COUNT] = [NEW_AW; CHANNEL_COUNT];

mod sealed {
    pub trait Channel {}

    pub trait Word {}
}

pub trait Channel: Peripheral<P = Self> + sealed::Channel + Into<AnyChannel> + Sized + 'static {
    fn number(&self) -> u8;

    fn regs(&self) -> pac::dma::Channel {
        pac::DMA.ch(self.number() as _)
    }

    fn degrade(self) -> AnyChannel {
        AnyChannel { number: self.number() }
    }
}

pub trait Word: sealed::Word {
    fn size() -> vals::DataSize;
}

impl sealed::Word for u8 {}
impl Word for u8 {
    fn size() -> vals::DataSize {
        vals::DataSize::SIZE_BYTE
    }
}

impl sealed::Word for u16 {}
impl Word for u16 {
    fn size() -> vals::DataSize {
        vals::DataSize::SIZE_HALFWORD
    }
}

impl sealed::Word for u32 {}
impl Word for u32 {
    fn size() -> vals::DataSize {
        vals::DataSize::SIZE_WORD
    }
}

pub struct AnyChannel {
    number: u8,
}

impl_peripheral!(AnyChannel);

impl sealed::Channel for AnyChannel {}
impl Channel for AnyChannel {
    fn number(&self) -> u8 {
        self.number
    }
}

macro_rules! channel {
    ($name:ident, $num:expr) => {
        impl sealed::Channel for peripherals::$name {}
        impl Channel for peripherals::$name {
            fn number(&self) -> u8 {
                $num
            }
        }

        impl From<peripherals::$name> for crate::dma::AnyChannel {
            fn from(val: peripherals::$name) -> Self {
                crate::dma::Channel::degrade(val)
            }
        }
    };
}

// TODO: replace transmutes with core::ptr::metadata once it's stable
#[allow(unused)]
pub(crate) fn slice_ptr_parts<T>(slice: *const [T]) -> (usize, usize) {
    unsafe { core::mem::transmute(slice) }
}

#[allow(unused)]
pub(crate) fn slice_ptr_parts_mut<T>(slice: *mut [T]) -> (usize, usize) {
    unsafe { core::mem::transmute(slice) }
}

channel!(DMA_CH0, 0);
channel!(DMA_CH1, 1);
channel!(DMA_CH2, 2);
channel!(DMA_CH3, 3);
channel!(DMA_CH4, 4);
channel!(DMA_CH5, 5);
channel!(DMA_CH6, 6);
channel!(DMA_CH7, 7);
channel!(DMA_CH8, 8);
channel!(DMA_CH9, 9);
channel!(DMA_CH10, 10);
channel!(DMA_CH11, 11);

// TODO as in rp2040 datasheet 2.5.3.2, dreq can only be used by one
// channel at a time to prevent errors. Should we enforce this?
#[allow(non_camel_case_types, dead_code)]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Dreq {
    PIO0_TX0 = 0x0,
    PIO0_TX1 = 0x1,
    PIO0_TX2 = 0x2,
    PIO0_TX3 = 0x3,
    PIO0_RX0 = 0x4,
    PIO0_RX1 = 0x5,
    PIO0_RX2 = 0x6,
    PIO0_RX3 = 0x7,
    PIO1_TX0 = 0x8,
    PIO1_TX1 = 0x9,
    PIO1_TX2 = 0xa,
    PIO1_TX3 = 0xb,
    PIO1_RX0 = 0xc,
    PIO1_RX1 = 0xd,
    PIO1_RX2 = 0xe,
    PIO1_RX3 = 0xf,
    SPI0_TX = 0x10,
    SPI0_RX = 0x11,
    SPI1_TX = 0x12,
    SPI1_RX = 0x13,
    UART0_TX = 0x14,
    UART0_RX = 0x15,
    UART1_TX = 0x16,
    UART1_RX = 0x17,
    PWM_WRAP0 = 0x18,
    PWM_WRAP1 = 0x19,
    PWM_WRAP2 = 0x1a,
    PWM_WRAP3 = 0x1b,
    PWM_WRAP4 = 0x1c,
    PWM_WRAP5 = 0x1d,
    PWM_WRAP6 = 0x1e,
    PWM_WRAP7 = 0x1f,
    I2C0_TX = 0x20,
    I2C0_RX = 0x21,
    I2C1_TX = 0x22,
    I2C1_RX = 0x23,
    ADC = 0x24,
    XIP_STREAM = 0x25,
    XIP_SSITX = 0x26,
    XIP_SSIRX = 0x27,
}

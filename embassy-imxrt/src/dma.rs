//! DMA driver.

use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::{Context, Poll};

use embassy_hal_internal::{Peri, PeripheralType, impl_peripheral};
use embassy_sync::waitqueue::AtomicWaker;
use pac::dma::vals::{Dstinc, Srcinc, Width};

use crate::clocks::enable_and_reset;
use crate::interrupt::InterruptExt;
use crate::peripherals::DMA0;
use crate::sealed::Sealed;
use crate::{BitIter, interrupt, pac, peripherals};

pub(crate) const MAX_CHUNK_SIZE: usize = 1024;

#[cfg(feature = "rt")]
#[interrupt]
fn DMA0() {
    let reg = crate::pac::DMA0;

    if reg.intstat().read().activeerrint() {
        let err = reg.errint0().read().0;

        for channel in BitIter(err) {
            error!("DMA error interrupt on channel {}!", channel);
            reg.errint0().write(|w| w.set_channel(channel as usize, true));
            CHANNEL_WAKERS[channel as usize].wake();
        }
    }

    if reg.intstat().read().activeint() {
        let ia = reg.inta0().read().0;

        for channel in BitIter(ia) {
            reg.inta0().write(|w| w.set_channel(channel as usize, true));
            CHANNEL_WAKERS[channel as usize].wake();
        }
    }
}

/// Initialize DMA controllers (DMA0 only, for now)
///
/// # Safety
///
/// Must be called exactly once during system initialization.
pub(crate) unsafe fn init() {
    let sysctl0 = crate::pac::SYSCTL0;
    let dmactl0 = crate::pac::DMA0;

    enable_and_reset::<DMA0>();

    interrupt::DMA0.disable();
    interrupt::DMA0.set_priority(interrupt::Priority::P3);

    dmactl0.ctrl().modify(|w| w.set_enable(true));

    // Set channel descriptor SRAM base address
    // Descriptor base must be 1K aligned
    let descriptor_base = core::ptr::addr_of!(DESCRIPTORS.descs) as u32;
    dmactl0.srambase().write(|w| w.set_offset(descriptor_base));

    // Ensure AHB priority it highest (M4 == DMAC0)
    sysctl0.ahbmatrixprior().modify(|w| w.set_m4(0));

    interrupt::DMA0.unpend();
    interrupt::DMA0.enable();
}

/// DMA read.
///
/// # Safety
///
/// Slice must point to a valid location reachable by DMA.
pub unsafe fn read<'a, C: Channel, W: Word>(ch: Peri<'a, C>, from: *const W, to: *mut [W]) -> Transfer<'a, C> {
    let count = (to.len().div_ceil(W::size() as usize) - 1) as isize;

    copy_inner(
        ch,
        from as *const u32,
        (to as *mut u32).byte_offset(count * W::size()),
        W::width(),
        count,
        false,
        true,
        true,
    )
}

/// DMA write.
///
/// # Safety
///
/// Slice must point to a valid location reachable by DMA.
pub unsafe fn write<'a, C: Channel, W: Word>(ch: Peri<'a, C>, from: *const [W], to: *mut W) -> Transfer<'a, C> {
    let count = (from.len().div_ceil(W::size() as usize) - 1) as isize;

    copy_inner(
        ch,
        (from as *const u32).byte_offset(count * W::size()),
        to as *mut u32,
        W::width(),
        count,
        true,
        false,
        true,
    )
}

/// DMA copy between slices.
///
/// # Safety
///
/// Slices must point to locations reachable by DMA.
pub unsafe fn copy<'a, C: Channel, W: Word>(ch: Peri<'a, C>, from: &[W], to: &mut [W]) -> Transfer<'a, C> {
    let from_len = from.len();
    let to_len = to.len();
    assert_eq!(from_len, to_len);

    let count = (from_len.div_ceil(W::size() as usize) - 1) as isize;

    copy_inner(
        ch,
        from.as_ptr().byte_offset(count * W::size()) as *const u32,
        to.as_mut_ptr().byte_offset(count * W::size()) as *mut u32,
        W::width(),
        count,
        true,
        true,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
fn copy_inner<'a, C: Channel>(
    ch: Peri<'a, C>,
    from: *const u32,
    to: *mut u32,
    width: Width,
    count: isize,
    incr_read: bool,
    incr_write: bool,
    periph: bool,
) -> Transfer<'a, C> {
    let p = ch.regs();

    unsafe {
        DESCRIPTORS.descs[ch.number() as usize].src = from as u32;
        DESCRIPTORS.descs[ch.number() as usize].dest = to as u32;
    }

    compiler_fence(Ordering::SeqCst);

    p.errint0().write(|w| w.set_channel(ch.number() as usize, true));
    p.inta0().write(|w| w.set_channel(ch.number() as usize, true));

    p.channel(ch.number() as usize).cfg().write(|w| {
        w.set_chpriority(0);
        w.set_periphreqen(periph);
        w.set_hwtrigen(false);
    });

    p.intenset0().write(|w| w.set_channel(ch.number() as usize, true));

    p.channel(ch.number() as usize).xfercfg().write(|w| {
        w.set_xfercount(count as u16);
        w.set_cfgvalid(true);
        w.set_clrtrig(true);
        w.set_reload(false);
        w.set_setinta(true);
        w.set_width(width);
        w.set_srcinc(match incr_read {
            false => Srcinc::NO_INCREMENT,
            true => Srcinc::WIDTH_X_1,
            // REVISIT: what about WidthX2 and WidthX4?
        });
        w.set_dstinc(match incr_write {
            false => Dstinc::NO_INCREMENT,
            true => Dstinc::WIDTH_X_1,
            // REVISIT: what about WidthX2 and WidthX4?
        });
    });

    p.enableset0().write(|w| w.set_channel(ch.number() as usize, true));

    p.channel(ch.number() as usize).xfercfg().modify(|w| w.set_swtrig(true));

    compiler_fence(Ordering::SeqCst);

    Transfer::new(ch)
}

/// DMA transfer driver.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Transfer<'a, C: Channel> {
    channel: Peri<'a, C>,
}

impl<'a, C: Channel> Transfer<'a, C> {
    pub(crate) fn new(channel: Peri<'a, C>) -> Self {
        Self { channel }
    }

    pub(crate) fn abort(&mut self) -> usize {
        let p = self.channel.regs();

        p.abort0()
            .write(|w| w.set_channel(self.channel.number() as usize, true));
        while p.busy0().read().channel(self.channel.number() as usize) {}

        p.enableclr0()
            .write(|w| w.set_channel(self.channel.number() as usize, true));

        let width: u8 = p
            .channel(self.channel.number() as usize)
            .xfercfg()
            .read()
            .width()
            .into();

        let count = p.channel(self.channel.number() as usize).xfercfg().read().xfercount() + 1;

        usize::from(count) * usize::from(width)
    }
}

impl<'a, C: Channel> Drop for Transfer<'a, C> {
    fn drop(&mut self) {
        self.abort();
    }
}

impl<'a, C: Channel> Unpin for Transfer<'a, C> {}
impl<'a, C: Channel> Future for Transfer<'a, C> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Re-register the waker on each call to poll() because any calls to
        // wake will deregister the waker.
        CHANNEL_WAKERS[self.channel.number() as usize].register(cx.waker());

        if !self
            .channel
            .regs()
            .active0()
            .read()
            .channel(self.channel.number() as usize)
        {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

/// DMA channel descriptor
#[derive(Copy, Clone)]
#[repr(C)]
struct Descriptor {
    reserved: u32,
    src: u32,
    dest: u32,
    link: u32,
}

impl Descriptor {
    const fn new() -> Self {
        Self {
            reserved: 0,
            src: 0,
            dest: 0,
            link: 0,
        }
    }
}

#[repr(align(1024))]
struct Descriptors {
    descs: [Descriptor; CHANNEL_COUNT],
}

impl Descriptors {
    const fn new() -> Self {
        Self {
            descs: [const { Descriptor::new() }; CHANNEL_COUNT],
        }
    }
}

static mut DESCRIPTORS: Descriptors = Descriptors::new();
static CHANNEL_WAKERS: [AtomicWaker; CHANNEL_COUNT] = [const { AtomicWaker::new() }; CHANNEL_COUNT];
pub(crate) const CHANNEL_COUNT: usize = 33;

/// DMA channel interface.
#[allow(private_bounds)]
pub trait Channel: PeripheralType + Sealed + Into<AnyChannel> + Sized + 'static {
    /// Channel number.
    fn number(&self) -> u8;

    /// Channel registry block.
    fn regs(&self) -> pac::dma::Dma {
        crate::pac::DMA0
    }
}

/// DMA word.
#[allow(private_bounds)]
pub trait Word: Sealed {
    /// Transfer width.
    fn width() -> Width;

    /// Size in bytes for the width.
    fn size() -> isize;
}

impl Sealed for u8 {}
impl Word for u8 {
    fn width() -> Width {
        Width::BIT_8
    }

    fn size() -> isize {
        1
    }
}

impl Sealed for u16 {}
impl Word for u16 {
    fn width() -> Width {
        Width::BIT_16
    }

    fn size() -> isize {
        2
    }
}

impl Sealed for u32 {}
impl Word for u32 {
    fn width() -> Width {
        Width::BIT_32
    }

    fn size() -> isize {
        4
    }
}

/// Type erased DMA channel.
pub struct AnyChannel {
    number: u8,
}

impl_peripheral!(AnyChannel);

impl Sealed for AnyChannel {}
impl Channel for AnyChannel {
    fn number(&self) -> u8 {
        self.number
    }
}

macro_rules! channel {
    ($name:ident, $num:expr) => {
        impl Sealed for peripherals::$name {}
        impl Channel for peripherals::$name {
            fn number(&self) -> u8 {
                $num
            }
        }

        impl From<peripherals::$name> for crate::dma::AnyChannel {
            fn from(val: peripherals::$name) -> Self {
                Self { number: val.number() }
            }
        }
    };
}

channel!(DMA0_CH0, 0);
channel!(DMA0_CH1, 1);
channel!(DMA0_CH2, 2);
channel!(DMA0_CH3, 3);
channel!(DMA0_CH4, 4);
channel!(DMA0_CH5, 5);
channel!(DMA0_CH6, 6);
channel!(DMA0_CH7, 7);
channel!(DMA0_CH8, 8);
channel!(DMA0_CH9, 9);
channel!(DMA0_CH10, 10);
channel!(DMA0_CH11, 11);
channel!(DMA0_CH12, 12);
channel!(DMA0_CH13, 13);
channel!(DMA0_CH14, 14);
channel!(DMA0_CH15, 15);
channel!(DMA0_CH16, 16);
channel!(DMA0_CH17, 17);
channel!(DMA0_CH18, 18);
channel!(DMA0_CH19, 19);
channel!(DMA0_CH20, 20);
channel!(DMA0_CH21, 21);
channel!(DMA0_CH22, 22);
channel!(DMA0_CH23, 23);
channel!(DMA0_CH24, 24);
channel!(DMA0_CH25, 25);
channel!(DMA0_CH26, 26);
channel!(DMA0_CH27, 27);
channel!(DMA0_CH28, 28);
channel!(DMA0_CH29, 29);
channel!(DMA0_CH30, 30);
channel!(DMA0_CH31, 31);
channel!(DMA0_CH32, 32);

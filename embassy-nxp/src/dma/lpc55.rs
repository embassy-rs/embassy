use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Context, Poll};

use critical_section::Mutex;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_hal_internal::{impl_peripheral, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::pac::{DMA0, SYSCON, *};
use crate::{peripherals, Peri};

#[interrupt]
fn DMA0() {
    let inta = DMA0.inta0().read().ia();
    for channel in 0..CHANNEL_COUNT {
        if (DMA0.errint0().read().err() & (1 << channel)) != 0 {
            panic!("DMA: error on DMA_0 channel {}", channel);
        }

        if (inta & (1 << channel)) != 0 {
            CHANNEL_WAKERS[channel].wake();
            DMA0.inta0().modify(|w| w.set_ia(1 << channel));
        }
    }
}

pub(crate) fn init() {
    assert_eq!(core::mem::size_of::<DmaDescriptor>(), 16, "Descriptor must be 16 bytes");
    assert_eq!(
        core::mem::align_of::<DmaDescriptor>(),
        16,
        "Descriptor must be 16-byte aligned"
    );
    assert_eq!(
        core::mem::align_of::<DmaDescriptorTable>(),
        512,
        "Table must be 512-byte aligned"
    );
    // Start clock for DMA
    SYSCON.ahbclkctrl0().modify(|w| w.set_dma0(true));
    // Reset DMA
    SYSCON
        .presetctrl0()
        .modify(|w| w.set_dma0_rst(syscon::vals::Dma0Rst::ASSERTED));
    SYSCON
        .presetctrl0()
        .modify(|w| w.set_dma0_rst(syscon::vals::Dma0Rst::RELEASED));

    // Address bits 31:9 of the beginning of the DMA descriptor table
    critical_section::with(|cs| {
        DMA0.srambase()
            .write(|w| w.set_offset((DMA_DESCRIPTORS.borrow(cs).as_ptr() as u32) >> 9));
    });
    // Enable DMA controller
    DMA0.ctrl().modify(|w| w.set_enable(true));

    unsafe {
        crate::pac::interrupt::DMA0.enable();
    }
    info!("DMA initialized");
}

/// DMA read.
///
/// SAFETY: Slice must point to a valid location reachable by DMA.
pub unsafe fn read<'a, C: Channel, W: Word>(ch: Peri<'a, C>, from: *const W, to: *mut [W]) -> Transfer<'a, C> {
    copy_inner(
        ch,
        from as *const u32,
        to as *mut W as *mut u32,
        to.len(),
        W::size(),
        false,
        true,
    )
}

/// DMA write.
///
/// SAFETY: Slice must point to a valid location reachable by DMA.
pub unsafe fn write<'a, C: Channel, W: Word>(ch: Peri<'a, C>, from: *const [W], to: *mut W) -> Transfer<'a, C> {
    copy_inner(
        ch,
        from as *const W as *const u32,
        to as *mut u32,
        from.len(),
        W::size(),
        true,
        false,
    )
}

/// DMA copy between slices.
///
/// SAFETY: Slices must point to locations reachable by DMA.
pub unsafe fn copy<'a, C: Channel, W: Word>(ch: Peri<'a, C>, from: &[W], to: &mut [W]) -> Transfer<'a, C> {
    let from_len = from.len();
    let to_len = to.len();
    assert_eq!(from_len, to_len);
    copy_inner(
        ch,
        from.as_ptr() as *const u32,
        to.as_mut_ptr() as *mut u32,
        from_len,
        W::size(),
        true,
        true,
    )
}

fn copy_inner<'a, C: Channel>(
    ch: Peri<'a, C>,
    from: *const u32,
    to: *mut u32,
    len: usize,
    data_size: crate::pac::dma::vals::Width,
    incr_src: bool,
    incr_dest: bool,
) -> Transfer<'a, C> {
    let p = ch.regs();

    // Buffer ending address = buffer starting address + (XFERCOUNT * the transfer increment)
    // XREFCOUNT = the number of transfers performed - 1.
    // The 1st transfer is included in the starting address.
    let source_end_addr = if incr_src {
        from as u32 + len as u32 - 1
    } else {
        from as u32
    };
    let dest_end_addr = if incr_dest {
        to as u32 + len as u32 - 1
    } else {
        to as u32
    };

    compiler_fence(Ordering::SeqCst);

    critical_section::with(|cs| {
        DMA_DESCRIPTORS.borrow(cs).borrow_mut().descriptors[ch.number() as usize] = DmaDescriptor {
            reserved: 0,
            source_end_addr,
            dest_end_addr,
            next_desc: 0, // Since only single transfers are made, there is no need for reload descriptor address.
        }
    });

    compiler_fence(Ordering::SeqCst);

    p.cfg().modify(|w| {
        // Peripheral DMA requests are enabled.
        // DMA requests that pace transfers can be interpreted then.
        w.set_periphreqen(true);
        // There is no need to have them on.
        // No complex transfers are performed for now.
        w.set_hwtrigen(false);
        w.set_chpriority(0);
    });

    p.xfercfg().modify(|w| {
        // This bit indicates whether the current channel descriptor is
        // valid and can potentially be acted upon,
        // if all other activation criteria are fulfilled.
        w.set_cfgvalid(true);
        // Indicates whether the channel’s control structure will be reloaded
        // when the current descriptor is exhausted.
        // Reloading allows ping-pong and linked transfers.
        w.set_reload(false);
        // There is no hardware distinction between interrupt A and B.
        // They can be used by software to assist with more complex descriptor usage.
        // By convention, interrupt A may be used when only one interrupt flag is needed.
        w.set_setinta(true);
        w.set_setintb(false);
        w.set_width(data_size);
        w.set_srcinc(if incr_src {
            dma::vals::Srcinc::WIDTH_X_1
        } else {
            dma::vals::Srcinc::NO_INCREMENT
        });
        w.set_dstinc(if incr_dest {
            dma::vals::Dstinc::WIDTH_X_1
        } else {
            dma::vals::Dstinc::NO_INCREMENT
        });
        // Total number of transfers to be performed, minus 1 encoded.
        w.set_xfercount((len as u16) - 1);
        // Before triggering the channel, it has to be enabled.
        w.set_swtrig(false);
    });

    compiler_fence(Ordering::SeqCst);
    DMA0.enableset0().write(|w| w.set_ena(1 << ch.number()));
    DMA0.intenset0().write(|w| w.set_inten(1 << ch.number()));

    compiler_fence(Ordering::SeqCst);
    // Start transfer.
    DMA0.settrig0().write(|w| w.set_trig(1 << ch.number()));
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
}

impl<'a, C: Channel> Drop for Transfer<'a, C> {
    fn drop(&mut self) {
        DMA0.enableclr0().write(|w| w.set_clr(1 << self.channel.number()));
        while (DMA0.busy0().read().bsy() & (1 << self.channel.number())) != 0 {}
        DMA0.abort0().write(|w| w.set_abortctrl(1 << self.channel.number()));
    }
}

impl<'a, C: Channel> Unpin for Transfer<'a, C> {}
impl<'a, C: Channel> Future for Transfer<'a, C> {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // We need to register/re-register the waker for each poll because any
        // calls to wake will deregister the waker.
        CHANNEL_WAKERS[self.channel.number() as usize].register(cx.waker());
        // Check if it is busy or not.
        if (DMA0.busy0().read().bsy() & (1 << self.channel.number())) != 0 {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

// Total number of channles including both DMA0 and DMA1.
// In spite of using only DMA0 channels, the descriptor table
// should be of this size.
pub(crate) const CHANNEL_COUNT: usize = 32;

static CHANNEL_WAKERS: [AtomicWaker; CHANNEL_COUNT] = [const { AtomicWaker::new() }; CHANNEL_COUNT];

// See section 22.5.2 (table 450)
// UM11126, Rev. 2.8
// The size of a descriptor must be aligned to a multiple of 16 bytes.
#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct DmaDescriptor {
    /// 0x0 Reserved.
    reserved: u32,
    /// 0x4 Source data end address.
    source_end_addr: u32,
    /// 0x8 Destination end address.
    dest_end_addr: u32,
    /// 0xC Link to next descriptor.
    next_desc: u32,
}

// See section 22.6.3
// UM11126, Rev. 2.8
// The table must begin on a 512 byte boundary.
#[repr(C, align(512))]
struct DmaDescriptorTable {
    descriptors: [DmaDescriptor; CHANNEL_COUNT],
}

// DMA descriptors are stored in on-chip SRAM.
static DMA_DESCRIPTORS: Mutex<RefCell<DmaDescriptorTable>> = Mutex::new(RefCell::new(DmaDescriptorTable {
    descriptors: [DmaDescriptor {
        reserved: 0,
        source_end_addr: 0,
        dest_end_addr: 0,
        next_desc: 0,
    }; CHANNEL_COUNT],
}));

trait SealedChannel {}
trait SealedWord {}

/// DMA channel interface.
#[allow(private_bounds)]
pub trait Channel: PeripheralType + SealedChannel + Into<AnyChannel> + Sized + 'static {
    /// Channel number.
    fn number(&self) -> u8;

    /// Channel registry block.
    fn regs(&self) -> crate::pac::dma::Channel {
        crate::pac::DMA0.channel(self.number() as _)
    }
}

/// DMA word.
#[allow(private_bounds)]
pub trait Word: SealedWord {
    /// Word size.
    fn size() -> crate::pac::dma::vals::Width;
}

impl SealedWord for u8 {}
impl Word for u8 {
    fn size() -> crate::pac::dma::vals::Width {
        crate::pac::dma::vals::Width::BIT_8
    }
}

impl SealedWord for u16 {}
impl Word for u16 {
    fn size() -> crate::pac::dma::vals::Width {
        crate::pac::dma::vals::Width::BIT_16
    }
}

impl SealedWord for u32 {}
impl Word for u32 {
    fn size() -> crate::pac::dma::vals::Width {
        crate::pac::dma::vals::Width::BIT_32
    }
}

/// Type erased DMA channel.
pub struct AnyChannel {
    number: u8,
}

impl_peripheral!(AnyChannel);

impl SealedChannel for AnyChannel {}
impl Channel for AnyChannel {
    fn number(&self) -> u8 {
        self.number
    }
}

macro_rules! channel {
    ($name:ident, $num:expr) => {
        impl SealedChannel for peripherals::$name {}
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
channel!(DMA_CH12, 12);
channel!(DMA_CH13, 13);
channel!(DMA_CH14, 14);
channel!(DMA_CH15, 15);
channel!(DMA_CH16, 16);
channel!(DMA_CH17, 17);
channel!(DMA_CH18, 18);
channel!(DMA_CH19, 19);
channel!(DMA_CH20, 20);
channel!(DMA_CH21, 21);
channel!(DMA_CH22, 22);

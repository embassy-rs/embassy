//! DMA

pub mod channel;
pub mod transfer;

use core::marker::PhantomData;
use core::ptr;

use embassy_hal_internal::impl_peripheral;
use embassy_hal_internal::interrupt::InterruptExt;
use embassy_sync::waitqueue::AtomicWaker;

use crate::clocks::enable_and_reset;
use crate::dma::channel::Channel;
use crate::peripherals::{self, DMA0};
use crate::{interrupt, Peri, PeripheralType};

// TODO:
//
//  - add support for DMA1
//  - support other transfer data widths (8-bit only)
//  - locking on common dma register configuration

const DMA_CHANNEL_COUNT: usize = 33;

/// DMA channel descriptor
#[derive(Copy, Clone, Debug)]
#[repr(C)]
struct ChannelDescriptor {
    reserved: u32,
    src_data_end_addr: u32,
    dst_data_end_addr: u32,
    nxt_desc_link_addr: u32,
}

/// DMA channel descriptor memory block (1KB aligned)
#[repr(align(1024))]
#[derive(Copy, Clone, Debug)]
struct DescriptorBlock {
    list: [ChannelDescriptor; DMA_CHANNEL_COUNT],
}

/// DMA channel descriptor list
static mut DESCRIPTORS: DescriptorBlock = DescriptorBlock {
    list: [ChannelDescriptor {
        reserved: 0,
        src_data_end_addr: 0,
        dst_data_end_addr: 0,
        nxt_desc_link_addr: 0,
    }; DMA_CHANNEL_COUNT],
};

/// DMA errors
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// Configuration requested is not supported
    UnsupportedConfiguration,
}

// One waker per channel
static DMA_WAKERS: [AtomicWaker; DMA_CHANNEL_COUNT] = [const { AtomicWaker::new() }; DMA_CHANNEL_COUNT];

#[cfg(feature = "rt")]
#[interrupt]
#[allow(non_snake_case)]
fn DMA0() {
    dma0_irq_handler(&DMA_WAKERS);
}

#[cfg(feature = "rt")]
fn dma0_irq_handler<const N: usize>(wakers: &[AtomicWaker; N]) {
    // SAFETY: unsafe needed to take pointer to Dma0 during interrupt handling
    let reg = unsafe { crate::pac::Dma0::steal() };

    // Is an error interrupt pending?
    if reg.intstat().read().activeerrint().bit() {
        let err = reg.errint0().read().bits();
        // Loop through interrupt bitfield, excluding trailing and leading zeros looking for interrupt source(s)
        for channel in err.trailing_zeros()..(32 - err.leading_zeros()) {
            if err & (1 << channel) != 0 {
                error!("DMA error interrupt on channel {}!", channel);
                // Clear the pending interrupt for this channel
                // SAFETY: unsafe due to .bits usage
                reg.errint0().write(|w| unsafe { w.err().bits(1 << channel) });
                wakers[channel as usize].wake();
            }
        }
    }

    // Is a transfer complete interrupt pending?
    if reg.intstat().read().activeint().bit() {
        let ia = reg.inta0().read().bits();
        // Loop through interrupt bitfield, excluding trailing and leading zeros looking for interrupt source(s)
        for channel in ia.trailing_zeros()..(32 - ia.leading_zeros()) {
            if ia & (1 << channel) != 0 {
                // Clear the pending interrupt for this channel
                // SAFETY: unsafe due to .bits usage
                reg.inta0().write(|w| unsafe { w.ia().bits(1 << channel) });
                wakers[channel as usize].wake();
            }
        }
    }
}

/// Initialize DMA controllers (DMA0 only, for now)
pub(crate) fn init() {
    // SAFETY: init should only be called once during HAL initialization
    let sysctl0 = unsafe { crate::pac::Sysctl0::steal() };
    let dmactl0 = unsafe { crate::pac::Dma0::steal() };

    enable_and_reset::<DMA0>();

    // Enable DMA controller
    dmactl0.ctrl().modify(|_, w| w.enable().set_bit());

    // Set channel descriptor SRAM base address
    // SAFETY: unsafe due to .bits usage and use of a mutable static (DESCRIPTORS.list)
    unsafe {
        // Descriptor base must be 1K aligned
        let descriptor_base = ptr::addr_of!(DESCRIPTORS.list) as u32;
        dmactl0.srambase().write(|w| w.bits(descriptor_base));
    }

    // Ensure AHB priority it highest (M4 == DMAC0)
    // SAFETY: unsafe due to .bits usage
    sysctl0.ahbmatrixprior().modify(|_, w| unsafe { w.m4().bits(0) });

    // Enable DMA interrupts on DMA0
    interrupt::DMA0.unpend();
    // SAFETY: enabling the dma0 controller interrupt is an unsafe call
    unsafe {
        interrupt::DMA0.enable();
    }
}

/// DMA device
pub struct Dma<'d> {
    _lifetime: PhantomData<&'d ()>,
}

struct DmaInfo {
    regs: crate::pac::Dma0,
    ch_num: usize,
}

impl<'d> Dma<'d> {
    /// Reserves a DMA channel for exclusive use
    pub fn reserve_channel<T: Instance>(_inner: Peri<'d, T>) -> Option<Channel<'d>> {
        if T::info().is_some() {
            Some(Channel {
                info: T::info().unwrap(),
                _lifetime: PhantomData,
            })
        } else {
            None
        }
    }
}

trait SealedInstance {
    fn info() -> Option<DmaInfo>;
}

/// DMA instance trait
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static + Send {
    /// Interrupt for this DMA instance
    type Interrupt: interrupt::typelevel::Interrupt;
}

macro_rules! dma_channel_instance {
    ($instance: ident, $controller: ident, $interrupt: ident, $number: expr) => {
        impl Instance for peripherals::$instance {
            type Interrupt = crate::interrupt::typelevel::$interrupt;
        }

        impl SealedInstance for peripherals::$instance {
            fn info() -> Option<DmaInfo> {
                Some(DmaInfo {
                    // SAFETY: safe from single executor
                    regs: unsafe { crate::pac::$controller::steal() },
                    ch_num: $number,
                })
            }
        }
    };
}

dma_channel_instance!(DMA0_CH0, Dma0, DMA0, 0);
dma_channel_instance!(DMA0_CH1, Dma0, DMA0, 1);
dma_channel_instance!(DMA0_CH2, Dma0, DMA0, 2);
dma_channel_instance!(DMA0_CH3, Dma0, DMA0, 3);
dma_channel_instance!(DMA0_CH4, Dma0, DMA0, 4);
dma_channel_instance!(DMA0_CH5, Dma0, DMA0, 5);
dma_channel_instance!(DMA0_CH6, Dma0, DMA0, 6);
dma_channel_instance!(DMA0_CH7, Dma0, DMA0, 7);
dma_channel_instance!(DMA0_CH8, Dma0, DMA0, 8);
dma_channel_instance!(DMA0_CH9, Dma0, DMA0, 9);
dma_channel_instance!(DMA0_CH10, Dma0, DMA0, 10);
dma_channel_instance!(DMA0_CH11, Dma0, DMA0, 11);
dma_channel_instance!(DMA0_CH12, Dma0, DMA0, 12);
dma_channel_instance!(DMA0_CH13, Dma0, DMA0, 13);
dma_channel_instance!(DMA0_CH14, Dma0, DMA0, 14);
dma_channel_instance!(DMA0_CH15, Dma0, DMA0, 15);
dma_channel_instance!(DMA0_CH16, Dma0, DMA0, 16);
dma_channel_instance!(DMA0_CH17, Dma0, DMA0, 17);
dma_channel_instance!(DMA0_CH18, Dma0, DMA0, 18);
dma_channel_instance!(DMA0_CH19, Dma0, DMA0, 19);
dma_channel_instance!(DMA0_CH20, Dma0, DMA0, 20);
dma_channel_instance!(DMA0_CH21, Dma0, DMA0, 21);
dma_channel_instance!(DMA0_CH22, Dma0, DMA0, 22);
dma_channel_instance!(DMA0_CH23, Dma0, DMA0, 23);
dma_channel_instance!(DMA0_CH24, Dma0, DMA0, 24);
dma_channel_instance!(DMA0_CH25, Dma0, DMA0, 25);
dma_channel_instance!(DMA0_CH26, Dma0, DMA0, 26);
dma_channel_instance!(DMA0_CH27, Dma0, DMA0, 27);
dma_channel_instance!(DMA0_CH28, Dma0, DMA0, 28);
dma_channel_instance!(DMA0_CH29, Dma0, DMA0, 29);
dma_channel_instance!(DMA0_CH30, Dma0, DMA0, 30);
dma_channel_instance!(DMA0_CH31, Dma0, DMA0, 31);
dma_channel_instance!(DMA0_CH32, Dma0, DMA0, 32);

/// IMPORTANT: DO NOT USE unless you are aware of the performance implications of not using DMA.
/// NoDma should only be used when a Flexcomm doesn't support DMA, such as Flexcomm 15.
///
/// For other transport layers, like UART, NoDma is not supported.
pub struct NoDma;
impl_peripheral!(NoDma);

impl Instance for NoDma {
    type Interrupt = crate::interrupt::typelevel::DMA0;
}

impl SealedInstance for NoDma {
    fn info() -> Option<DmaInfo> {
        None
    }
}

//! USB driver.

use core::{marker::PhantomData, slice};

use atomic_polyfill::{compiler_fence, Ordering};
use embassy_sync::waitqueue::AtomicWaker;
use embassy_usb_driver::Direction;
use crate::{interrupt, pac, peripherals};

trait SealedInstance {
    fn regs() -> crate::pac::usb::Usb;
    fn dpram() -> crate::pac::usb_dpram::UsbDpram;
    
    // FIXME(svd): Add EPX to svd
    fn dpram_epx_control() -> pac::common::Reg<pac::usb_dpram::regs::EpControl, pac::common::RW>;
}

/// USB peripheral instance.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + 'static {
    /// Interrupt for this peripheral.
    type Interrupt: interrupt::typelevel::Interrupt;
}

impl crate::usb::SealedInstance for peripherals::USB {
    fn regs() -> pac::usb::Usb {
        pac::USB
    }
    fn dpram() -> pac::usb_dpram::UsbDpram {
        pac::USB_DPRAM
    }

    fn dpram_epx_control() -> pac::common::Reg<pac::usb_dpram::regs::EpControl, pac::common::RW> {
        unsafe {
            pac::common::Reg::from_ptr((Self::dpram().as_ptr().byte_offset(0x100)) as _)
        }
    }
}

impl crate::usb::Instance for peripherals::USB {
    type Interrupt = crate::interrupt::typelevel::USBCTRL_IRQ;
}

const EP_COUNT: usize = 16;
const EP_MEMORY_SIZE: usize = 4096;
const EP_MEMORY: *mut u8 = pac::USB_DPRAM.as_ptr() as *mut u8;
const DPRAM_DATA_OFFSET: u16 = 0x180;

const NEW_AW: AtomicWaker = AtomicWaker::new();
static BUS_WAKER: AtomicWaker = NEW_AW;
static EP_IN_WAKERS: [AtomicWaker; EP_COUNT] = [NEW_AW; EP_COUNT];
static EP_OUT_WAKERS: [AtomicWaker; EP_COUNT] = [NEW_AW; EP_COUNT];

struct EndpointBuffer<T: Instance> {
    addr: u16,
    len: u16,
    _phantom: PhantomData<T>,
}

impl<T: Instance> EndpointBuffer<T> {
    const fn new(addr: u16, len: u16) -> Self {
        Self {
            addr,
            len,
            _phantom: PhantomData,
        }
    }

    fn read(&mut self, buf: &mut [u8]) {
        assert!(buf.len() <= self.len as usize);
        compiler_fence(Ordering::SeqCst);
        let mem = unsafe { slice::from_raw_parts(EP_MEMORY.add(self.addr as _), buf.len()) };
        buf.copy_from_slice(mem);
        compiler_fence(Ordering::SeqCst);
    }

    fn write(&mut self, buf: &[u8]) {
        assert!(buf.len() <= self.len as usize);
        compiler_fence(Ordering::SeqCst);
        let mem = unsafe { slice::from_raw_parts_mut(EP_MEMORY.add(self.addr as _), buf.len()) };
        mem.copy_from_slice(buf);
        compiler_fence(Ordering::SeqCst);
    }
}

trait Dir {
    fn dir() -> Direction;
}

/// Type for In direction.
pub enum In {}
impl Dir for In {
    fn dir() -> Direction {
        Direction::In
    }
}

/// Type for Out direction.
pub enum Out {}
impl Dir for Out {
    fn dir() -> Direction {
        Direction::Out
    }
}

pub mod host;
pub mod device;

// FIXME: Compat
pub use device::*;

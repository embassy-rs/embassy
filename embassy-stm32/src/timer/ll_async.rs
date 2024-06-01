//! Low level async timer driver.

use core::future::Future;
use core::marker::PhantomData;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::{Channel, GeneralInstance4Channel, TimerBits};

/// All timer interrupts
#[derive(Clone, Copy)]
pub enum InterruptFlag {
    /// Update
    Update = 0,
    /// Capture/compare 1
    CaptureCompare1 = 1,
    /// Capture/compare 1
    CaptureCompare2 = 2,
    /// Capture/compare 1
    CaptureCompare3 = 3,
    /// Capture/compare 1
    CaptureCompare4 = 4,
    /// COM event
    ComEvent = 5,
    /// Trigger
    Trigger = 6,
    /// Break
    Break = 7,
}

/// Timer future
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct InputCaptureFuture<T: GeneralInstance4Channel> {
    regs: crate::pac::timer::TimGp32,
    flag: InterruptFlag,
    phantom: PhantomData<T>,
}

impl<T: GeneralInstance4Channel> InputCaptureFuture<T> {
    /// Enable the interrupt source and returns a new instance of Future
    pub fn new(regs: *mut (), flag: InterruptFlag) -> Self {
        let this = Self {
            regs: unsafe { crate::pac::timer::TimGp32::from_ptr(regs) },
            flag,
            phantom: PhantomData,
        };

        // set interrupt enable
        this.regs.dier().modify(|w| w.0 |= 1u32 << flag as u32);

        this
    }
}

impl From<Channel> for InterruptFlag {
    fn from(value: Channel) -> Self {
        match value {
            Channel::Ch1 => InterruptFlag::CaptureCompare1,
            Channel::Ch2 => InterruptFlag::CaptureCompare2,
            Channel::Ch3 => InterruptFlag::CaptureCompare3,
            Channel::Ch4 => InterruptFlag::CaptureCompare4,
        }
    }
}

impl<T: GeneralInstance4Channel> Drop for InputCaptureFuture<T> {
    fn drop(&mut self) {
        critical_section::with(|_| {
            // clear interrupt enable
            self.regs.dier().modify(|w| w.0 &= !(1u32 << self.flag as u32));
        });
    }
}

impl<T: GeneralInstance4Channel> Future for InputCaptureFuture<T> {
    type Output = u32;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        T::state().wakers[self.flag as usize].register(cx.waker());

        // if interrupt enable is cleared, this means the interrupt handler executed, thus we can return the value
        let dier = self.regs.dier().read();
        if (dier.0 & (1u32 << self.flag as u32)) == 0 {
            let val = match self.flag {
                InterruptFlag::CaptureCompare1 => self.regs.ccr(Channel::Ch1.index()).read(),
                InterruptFlag::CaptureCompare2 => self.regs.ccr(Channel::Ch2.index()).read(),
                InterruptFlag::CaptureCompare3 => self.regs.ccr(Channel::Ch3.index()).read(),
                InterruptFlag::CaptureCompare4 => self.regs.ccr(Channel::Ch4.index()).read(),
                _ => self.regs.cnt().read(), // return the counter value
            };
            Poll::Ready(val)
        } else {
            Poll::Pending
        }
    }
}

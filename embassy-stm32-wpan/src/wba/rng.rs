use core::mem::MaybeUninit;

use embassy_stm32::rng::{Instance, Rng};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::zerocopy_channel::{self, Channel, Receiver, Sender};

pub struct RngState {
    buf: [[u8; 8]; 8],
    inner: MaybeUninit<RngStateInner<'static>>,
}

impl RngState {
    pub const fn new() -> Self {
        Self {
            buf: [[0u8; 8]; 8],
            inner: MaybeUninit::uninit(),
        }
    }
}

struct RngStateInner<'d> {
    ch: Channel<'d, CriticalSectionRawMutex, [u8; 8]>,
}

pub struct RngService<'d, T: Instance> {
    rng: Rng<'d, T>,
    sender: Sender<'d, CriticalSectionRawMutex, [u8; 8]>,
}

impl<'d, T: Instance> RngService<'d, T> {
    pub fn new(
        rng: Rng<'d, T>,
        state: &'static mut RngState,
    ) -> (Self, Receiver<'d, CriticalSectionRawMutex, [u8; 8]>) {
        // safety: this is a self-referential struct, however:
        // - it can't move while the `'d` borrow is active.
        // - when the borrow ends, the dangling references inside the MaybeUninit will never be used again.
        let state_uninit: *mut MaybeUninit<RngStateInner<'d>> =
            (&mut state.inner as *mut MaybeUninit<RngStateInner<'static>>).cast();
        let state = unsafe { &mut *state_uninit }.write(RngStateInner {
            ch: zerocopy_channel::Channel::new(&mut state.buf[..]),
        });

        let (sender, receiver) = state.ch.split();

        (Self { rng, sender }, receiver)
    }

    pub async fn run(&mut self) -> ! {
        loop {
            let buf = self.sender.send().await;
            self.rng.async_fill_bytes(&mut buf[..]).await.unwrap();
            self.sender.send_done();
        }
    }
}

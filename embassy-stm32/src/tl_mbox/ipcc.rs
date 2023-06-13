use core::future::poll_fn;
use core::task::Poll;

use atomic_polyfill::{compiler_fence, Ordering};
use embassy_cortex_m::interrupt::Interrupt;
use embassy_time::{Duration, Timer};

use self::sealed::Instance;
use super::mm::MemoryManager;
use crate::interrupt;
use crate::peripherals::IPCC;
use crate::rcc::sealed::RccPeripheral;

/// Interrupt handler.
pub struct ReceiveInterruptHandler {}

impl interrupt::Handler<interrupt::IPCC_C1_RX> for ReceiveInterruptHandler {
    unsafe fn on_interrupt() {
        let regs = IPCC::regs();

        let channels = [
            IpccChannel::Channel1,
            IpccChannel::Channel2,
            IpccChannel::Channel3,
            IpccChannel::Channel4,
            IpccChannel::Channel5,
            IpccChannel::Channel6,
        ];

        // Status register gives channel occupied status. For rx, use cpu1.
        let sr = unsafe { regs.cpu(1).sr().read() };
        regs.cpu(0).mr().modify(|w| {
            for channel in channels {
                if sr.chf(channel as usize) {
                    // info!("ipcc: rx occupied interrupt ch: {}", channel as usize + 1);

                    // If bit is set to 1 then interrupt is disabled; we want to disable the interrupt
                    w.set_chom(channel as usize, true);

                    // There shouldn't be a race because the channel is masked only if the interrupt has fired
                    IPCC::state().rx_waker_for(channel).wake();
                }
            }
        })
    }
}

pub struct TransmitInterruptHandler {}

impl interrupt::Handler<interrupt::IPCC_C1_TX> for TransmitInterruptHandler {
    unsafe fn on_interrupt() {
        let regs = IPCC::regs();

        let channels = [
            IpccChannel::Channel1,
            IpccChannel::Channel2,
            IpccChannel::Channel3,
            IpccChannel::Channel4,
            IpccChannel::Channel5,
            IpccChannel::Channel6,
        ];

        // Status register gives channel occupied status. For tx, use cpu0.
        let sr = unsafe { regs.cpu(0).sr().read() };
        regs.cpu(0).mr().modify(|w| {
            for channel in channels {
                if !sr.chf(channel as usize) {
                    // info!("ipcc: tx free interrupt ch: {}", channel as usize + 1);

                    // If bit is set to 1 then interrupt is disabled; we want to disable the interrupt
                    w.set_chfm(channel as usize, true);

                    // There shouldn't be a race because the channel is masked only if the interrupt has fired
                    IPCC::state().tx_waker_for(channel).wake();
                }
            }
        });

        compiler_fence(Ordering::SeqCst);

        // MemoryManager is special
        MemoryManager::run_queue();
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Default)]
pub struct Config {
    // TODO: add IPCC peripheral configuration, if any, here
    // reserved for future use
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum IpccChannel {
    Channel1 = 0,
    Channel2 = 1,
    Channel3 = 2,
    Channel4 = 3,
    Channel5 = 4,
    Channel6 = 5,
}

pub struct Ipcc;

impl Ipcc {
    pub fn enable(_config: Config) {
        IPCC::enable();
        IPCC::reset();
        IPCC::set_cpu2(true);

        unsafe { _configure_pwr() };

        let regs = IPCC::regs();

        unsafe {
            regs.cpu(0).cr().modify(|w| {
                w.set_rxoie(true);
                w.set_txfie(true);
            })
        }

        // enable interrupts
        crate::interrupt::IPCC_C1_RX::unpend();
        crate::interrupt::IPCC_C1_TX::unpend();

        unsafe { crate::interrupt::IPCC_C1_RX::enable() };
        unsafe { crate::interrupt::IPCC_C1_TX::enable() };
    }

    /// Send data to an IPCC channel. The closure is called to write the data when appropriate.
    pub async fn send(channel: IpccChannel, f: impl FnOnce()) {
        let regs = IPCC::regs();

        Self::flush(channel).await;
        compiler_fence(Ordering::SeqCst);

        f();

        compiler_fence(Ordering::SeqCst);

        info!("ipcc: ch {}: send data", channel as u8);
        unsafe { regs.cpu(0).scr().write(|w| w.set_chs(channel as usize, true)) }
    }

    /// Wait for the tx channel to become clear
    pub async fn flush(channel: IpccChannel) {
        let regs = IPCC::regs();

        // This is a race, but is nice for debugging
        if unsafe { regs.cpu(0).sr().read() }.chf(channel as usize) {
            info!("ipcc: ch {}:  wait for tx free", channel as u8);
        }

        poll_fn(|cx| {
            IPCC::state().tx_waker_for(channel).register(cx.waker());
            // If bit is set to 1 then interrupt is disabled; we want to enable the interrupt
            unsafe { regs.cpu(0).mr().modify(|w| w.set_chfm(channel as usize, false)) }

            compiler_fence(Ordering::SeqCst);

            if !unsafe { regs.cpu(0).sr().read() }.chf(channel as usize) {
                // If bit is set to 1 then interrupt is disabled; we want to disable the interrupt
                unsafe { regs.cpu(0).mr().modify(|w| w.set_chfm(channel as usize, true)) }

                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }

    /// Receive data from an IPCC channel. The closure is called to read the data when appropriate.
    pub async fn receive<R>(channel: IpccChannel, mut f: impl FnMut() -> Option<R>) -> R {
        let regs = IPCC::regs();

        loop {
            // This is a race, but is nice for debugging
            if !unsafe { regs.cpu(1).sr().read() }.chf(channel as usize) {
                info!("ipcc: ch {}:  wait for rx occupied", channel as u8);
            }

            poll_fn(|cx| {
                IPCC::state().rx_waker_for(channel).register(cx.waker());
                // If bit is set to 1 then interrupt is disabled; we want to enable the interrupt
                unsafe { regs.cpu(0).mr().modify(|w| w.set_chom(channel as usize, false)) }

                compiler_fence(Ordering::SeqCst);

                if unsafe { regs.cpu(1).sr().read() }.chf(channel as usize) {
                    // If bit is set to 1 then interrupt is disabled; we want to disable the interrupt
                    unsafe { regs.cpu(0).mr().modify(|w| w.set_chfm(channel as usize, true)) }

                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;

            info!("ipcc: ch {}:  read data", channel as u8);
            compiler_fence(Ordering::SeqCst);

            match f() {
                Some(ret) => return ret,
                None => {}
            }

            info!("ipcc: ch {}: clear rx", channel as u8);
            compiler_fence(Ordering::SeqCst);
            // If the channel is clear and the read function returns none, fetch more data
            unsafe { regs.cpu(0).scr().write(|w| w.set_chc(channel as usize, true)) }
        }
    }
}

impl sealed::Instance for crate::peripherals::IPCC {
    fn regs() -> crate::pac::ipcc::Ipcc {
        crate::pac::IPCC
    }

    fn set_cpu2(enabled: bool) {
        unsafe { crate::pac::PWR.cr4().modify(|w| w.set_c2boot(enabled)) }
    }

    fn state() -> &'static self::sealed::State {
        static STATE: self::sealed::State = self::sealed::State::new();
        &STATE
    }
}

pub(crate) mod sealed {
    use embassy_sync::waitqueue::AtomicWaker;

    use super::*;

    pub struct State {
        rx_wakers: [AtomicWaker; 6],
        tx_wakers: [AtomicWaker; 6],
    }

    impl State {
        pub const fn new() -> Self {
            const WAKER: AtomicWaker = AtomicWaker::new();

            Self {
                rx_wakers: [WAKER; 6],
                tx_wakers: [WAKER; 6],
            }
        }

        pub fn rx_waker_for(&self, channel: IpccChannel) -> &AtomicWaker {
            match channel {
                IpccChannel::Channel1 => &self.rx_wakers[0],
                IpccChannel::Channel2 => &self.rx_wakers[1],
                IpccChannel::Channel3 => &self.rx_wakers[2],
                IpccChannel::Channel4 => &self.rx_wakers[3],
                IpccChannel::Channel5 => &self.rx_wakers[4],
                IpccChannel::Channel6 => &self.rx_wakers[5],
            }
        }

        pub fn tx_waker_for(&self, channel: IpccChannel) -> &AtomicWaker {
            match channel {
                IpccChannel::Channel1 => &self.tx_wakers[0],
                IpccChannel::Channel2 => &self.tx_wakers[1],
                IpccChannel::Channel3 => &self.tx_wakers[2],
                IpccChannel::Channel4 => &self.tx_wakers[3],
                IpccChannel::Channel5 => &self.tx_wakers[4],
                IpccChannel::Channel6 => &self.tx_wakers[5],
            }
        }
    }

    pub trait Instance: crate::rcc::RccPeripheral {
        fn regs() -> crate::pac::ipcc::Ipcc;
        fn set_cpu2(enabled: bool);
        fn state() -> &'static State;
    }
}

unsafe fn _configure_pwr() {
    let rcc = crate::pac::RCC;

    // set RF wake-up clock = LSE
    rcc.csr().modify(|w| w.set_rfwkpsel(0b01));
}

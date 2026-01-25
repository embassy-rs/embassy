//! Inter-Process Communication Controller (IPCC)

use core::future::poll_fn;
use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};
use core::task::Poll;

use embassy_hal_internal::Peri;
use embassy_sync::waitqueue::AtomicWaker;

use crate::cpu::CoreId;
use crate::interrupt::typelevel::Interrupt;
use crate::peripherals::IPCC;
use crate::rcc::SealedRccPeripheral;
use crate::{interrupt, rcc};

/// Interrupt handler.
#[cfg(not(feature = "_core-cm0p"))]
pub struct ReceiveInterruptHandler {}

#[cfg(not(feature = "_core-cm0p"))]
impl interrupt::typelevel::Handler<interrupt::typelevel::IPCC_C1_RX> for ReceiveInterruptHandler {
    unsafe fn on_interrupt() {
        let regs = IPCC::regs();
        let core = CoreId::current();

        trace!("ipcc: rx interrupt");
        // Status register gives channel occupied status. For rx, use the other.
        let sr = regs.cpu(core.other().to_index()).sr().read();
        regs.cpu(core.to_index()).mr().modify(|w| {
            for index in 0..5 {
                if sr.chf(index as usize) {
                    // If bit is set to 1 then interrupt is disabled; we want to disable the interrupt
                    w.set_chom(index as usize, true);

                    // There shouldn't be a race because the channel is masked only if the interrupt has fired
                    IPCC::state().rx_waker_for(index).wake();
                }
            }
        })
    }
}

/// TX interrupt handler.
#[cfg(not(feature = "_core-cm0p"))]
pub struct TransmitInterruptHandler {}

#[cfg(not(feature = "_core-cm0p"))]
impl interrupt::typelevel::Handler<interrupt::typelevel::IPCC_C1_TX> for TransmitInterruptHandler {
    unsafe fn on_interrupt() {
        let regs = IPCC::regs();
        let core = CoreId::current();

        trace!("ipcc: rx interrupt");
        // Status register gives channel occupied status. For tx, use cpu0.
        let sr = regs.cpu(core.to_index()).sr().read();
        regs.cpu(core.to_index()).mr().modify(|w| {
            for index in 0..5 {
                if !sr.chf(index as usize) {
                    // If bit is set to 1 then interrupt is disabled; we want to disable the interrupt
                    w.set_chfm(index as usize, true);

                    // There shouldn't be a race because the channel is masked only if the interrupt has fired
                    IPCC::state().tx_waker_for(index).wake();
                }
            }
        });
    }
}

/// TX/RX interrupt handler
#[cfg(feature = "_core-cm0p")]
pub struct InterruptHandler {}

#[cfg(feature = "_core-cm0p")]
impl interrupt::typelevel::Handler<interrupt::typelevel::IPCC_C2_RX_C2_TX> for InterruptHandler {
    unsafe fn on_interrupt() {
        let regs = IPCC::regs();
        let core = CoreId::current();

        // Status register gives channel occupied status. For rx, use the other.
        let rx_sr = regs.cpu(core.other().to_index()).sr().read();
        // Status register gives channel occupied status. For tx, use cpu0.
        let tx_sr = regs.cpu(core.to_index()).sr().read();

        regs.cpu(core.to_index()).mr().modify(|w| {
            for index in 0..5 {
                if rx_sr.chf(index as usize) {
                    // If bit is set to 1 then interrupt is disabled; we want to disable the interrupt
                    w.set_chom(index as usize, true);

                    // There shouldn't be a race because the channel is masked only if the interrupt has fired
                    IPCC::state().rx_waker_for(index).wake();
                }
                if !tx_sr.chf(index as usize) {
                    // If bit is set to 1 then interrupt is disabled; we want to disable the interrupt
                    w.set_chfm(index as usize, true);

                    // There shouldn't be a race because the channel is masked only if the interrupt has fired
                    IPCC::state().tx_waker_for(index).wake();
                }
            }
        });
    }
}

/// IPCC config.
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    /// Boot CPU2 is true (default is true)
    #[cfg(not(feature = "_core-cm0p"))]
    pub c2_boot: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            #[cfg(not(feature = "_core-cm0p"))]
            c2_boot: true,
        }
    }
}

/// IPCC TX Channel
pub struct IpccTxChannel<'a> {
    index: u8,
    _lifetime: PhantomData<&'a mut usize>,
}

impl<'a> IpccTxChannel<'a> {
    pub(crate) const fn new(index: u8) -> Self {
        core::assert!(index < 6);

        Self {
            index: index,
            _lifetime: PhantomData,
        }
    }

    /// Send data to an IPCC channel. The closure is called to write the data when appropriate.
    pub async fn send(&mut self, f: impl FnOnce()) {
        let regs = IPCC::regs();
        let core = CoreId::current();

        self.flush().await;

        f();

        compiler_fence(Ordering::SeqCst);

        trace!("ipcc: ch {}: send data", self.index as u8);
        regs.cpu(core.to_index())
            .scr()
            .write(|w| w.set_chs(self.index as usize, true));
    }

    /// Wait for the tx channel to become clear
    pub async fn flush(&mut self) {
        let _scoped_block_stop = IPCC::RCC_INFO.block_stop();
        let regs = IPCC::regs();
        let core = CoreId::current();

        // This is a race, but is nice for debugging
        if regs.cpu(core.to_index()).sr().read().chf(self.index as usize) {
            trace!("ipcc: ch {}: wait for tx free", self.index as u8);
        }

        poll_fn(|cx| {
            IPCC::state().tx_waker_for(self.index).register(cx.waker());
            // If bit is set to 1 then interrupt is disabled; we want to enable the interrupt
            critical_section::with(|_| {
                regs.cpu(core.to_index())
                    .mr()
                    .modify(|w| w.set_chfm(self.index as usize, false))
            });

            compiler_fence(Ordering::SeqCst);

            if !regs.cpu(core.to_index()).sr().read().chf(self.index as usize) {
                // If bit is set to 1 then interrupt is disabled; we want to disable the interrupt
                critical_section::with(|_| {
                    regs.cpu(core.to_index())
                        .mr()
                        .modify(|w| w.set_chfm(self.index as usize, true))
                });

                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;
    }
}

/// IPCC RX Channel
pub struct IpccRxChannel<'a> {
    index: u8,
    _lifetime: PhantomData<&'a mut usize>,
}

impl<'a> IpccRxChannel<'a> {
    pub(crate) const fn new(index: u8) -> Self {
        core::assert!(index < 6);

        Self {
            index: index,
            _lifetime: PhantomData,
        }
    }

    /// Receive data from an IPCC channel. The closure is called to read the data when appropriate.
    pub async fn receive<R>(&mut self, mut f: impl FnMut() -> Option<R>) -> R {
        let _scoped_block_stop = IPCC::RCC_INFO.block_stop();
        let regs = IPCC::regs();
        let core = CoreId::current();

        loop {
            // This is a race, but is nice for debugging
            if !regs.cpu(core.other().to_index()).sr().read().chf(self.index as usize) {
                trace!("ipcc: ch {}: wait for rx occupied", self.index as u8);
            }

            poll_fn(|cx| {
                IPCC::state().rx_waker_for(self.index).register(cx.waker());
                // If bit is set to 1 then interrupt is disabled; we want to enable the interrupt
                critical_section::with(|_| {
                    regs.cpu(core.to_index())
                        .mr()
                        .modify(|w| w.set_chom(self.index as usize, false))
                });

                compiler_fence(Ordering::SeqCst);

                if regs.cpu(core.other().to_index()).sr().read().chf(self.index as usize) {
                    // If bit is set to 1 then interrupt is disabled; we want to disable the interrupt
                    critical_section::with(|_| {
                        regs.cpu(core.to_index())
                            .mr()
                            .modify(|w| w.set_chom(self.index as usize, true))
                    });

                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;

            trace!("ipcc: ch {}: read data", self.index as u8);

            let ret = f();

            trace!("ipcc: ch {}: clear rx", self.index as u8);
            compiler_fence(Ordering::SeqCst);
            // If the channel is clear and the read function returns none, fetch more data
            regs.cpu(core.to_index())
                .scr()
                .write(|w| w.set_chc(self.index as usize, true));
            match ret {
                Some(ret) => return ret,
                None => {}
            }
        }
    }
}

/// IPCC Channel
pub struct IpccChannel<'a> {
    index: u8,
    _lifetime: PhantomData<&'a mut usize>,
}

impl<'a> IpccChannel<'a> {
    pub(crate) const fn new(number: u8) -> Self {
        core::assert!(number > 0 && number <= 6);

        Self {
            index: number - 1,
            _lifetime: PhantomData,
        }
    }

    /// Split into a tx and rx channel
    pub const fn split(self) -> (IpccTxChannel<'a>, IpccRxChannel<'a>) {
        (IpccTxChannel::new(self.index), IpccRxChannel::new(self.index))
    }
}

/// IPCC driver.
pub struct Ipcc {
    _private: (),
}

impl Ipcc {
    /// Creates a new Ipcc instance.
    #[cfg(not(feature = "_core-cm0p"))]
    pub fn new<'d>(
        _peripheral: Peri<'d, crate::peripherals::IPCC>,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::IPCC_C1_RX, ReceiveInterruptHandler>
        + interrupt::typelevel::Binding<interrupt::typelevel::IPCC_C1_TX, TransmitInterruptHandler>
        + 'd,
        _config: Config,
    ) -> Self {
        rcc::enable_and_reset_without_stop::<IPCC>();
        IPCC::set_cpu2(_config.c2_boot);

        Self::init(_config);

        // enable interrupts
        crate::interrupt::typelevel::IPCC_C1_RX::unpend();
        crate::interrupt::typelevel::IPCC_C1_TX::unpend();

        unsafe { crate::interrupt::typelevel::IPCC_C1_RX::enable() };
        unsafe { crate::interrupt::typelevel::IPCC_C1_TX::enable() };

        Self { _private: () }
    }

    /// Creates a new Ipcc instance.
    #[cfg(feature = "_core-cm0p")]
    pub fn new<'d>(
        _peripheral: Peri<'d, crate::peripherals::IPCC>,
        _irq: impl interrupt::typelevel::Binding<interrupt::typelevel::IPCC_C2_RX_C2_TX, InterruptHandler> + 'd,
        _config: Config,
    ) -> Self {
        rcc::enable_and_reset_without_stop::<IPCC>();

        Self::init(_config);

        // enable interrupts
        crate::interrupt::typelevel::IPCC_C2_RX_C2_TX::unpend();
        unsafe { crate::interrupt::typelevel::IPCC_C2_RX_C2_TX::enable() };

        Self { _private: () }
    }

    fn init(_config: Config) {
        // Verify rfwkpsel is set
        let _ = IPCC::frequency();

        let regs = IPCC::regs();
        let core = CoreId::current();

        regs.cpu(core.to_index()).cr().modify(|w| {
            w.set_rxoie(true);
            w.set_txfie(true);
        });
    }

    /// Split into a tx and rx channel
    pub const fn split<'a>(self) -> [(IpccTxChannel<'a>, IpccRxChannel<'a>); 6] {
        [
            IpccChannel::new(1).split(),
            IpccChannel::new(2).split(),
            IpccChannel::new(3).split(),
            IpccChannel::new(4).split(),
            IpccChannel::new(5).split(),
            IpccChannel::new(6).split(),
        ]
    }

    /// Receive from a channel number
    pub async unsafe fn receive<R>(number: u8, f: impl FnMut() -> Option<R>) -> R {
        core::assert!(number > 0 && number <= 6);

        IpccRxChannel::new(number - 1).receive(f).await
    }

    /// Send to a channel number
    pub async unsafe fn send(number: u8, f: impl FnOnce()) {
        core::assert!(number > 0 && number <= 6);

        IpccTxChannel::new(number - 1).send(f).await
    }

    /// Send to a channel number
    pub async unsafe fn flush(number: u8) {
        core::assert!(number > 0 && number <= 6);

        IpccTxChannel::new(number - 1).flush().await
    }
}

impl SealedInstance for crate::peripherals::IPCC {
    fn regs() -> crate::pac::ipcc::Ipcc {
        crate::pac::IPCC
    }

    #[cfg(not(feature = "_core-cm0p"))]
    fn set_cpu2(enabled: bool) {
        crate::pac::PWR.cr4().modify(|w| w.set_c2boot(enabled));
    }

    fn state() -> &'static State {
        static STATE: State = State::new();
        &STATE
    }
}

struct State {
    rx_wakers: [AtomicWaker; 6],
    tx_wakers: [AtomicWaker; 6],
}

impl State {
    const fn new() -> Self {
        Self {
            rx_wakers: [const { AtomicWaker::new() }; 6],
            tx_wakers: [const { AtomicWaker::new() }; 6],
        }
    }

    const fn rx_waker_for(&self, index: u8) -> &AtomicWaker {
        &self.rx_wakers[index as usize]
    }

    const fn tx_waker_for(&self, index: u8) -> &AtomicWaker {
        &self.tx_wakers[index as usize]
    }
}

trait SealedInstance: crate::rcc::RccPeripheral {
    fn regs() -> crate::pac::ipcc::Ipcc;
    #[cfg(not(feature = "_core-cm0p"))]
    fn set_cpu2(enabled: bool);
    fn state() -> &'static State;
}

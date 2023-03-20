use core::future::poll_fn;
use core::ops::{Deref, DerefMut};
use core::task::Poll;

pub use bxcan;
use bxcan::Frame;
use embassy_hal_common::{into_ref, PeripheralRef};

use crate::gpio::sealed::AFType;
use crate::interrupt::{Interrupt, InterruptExt};
use crate::rcc::RccPeripheral;
use crate::{peripherals, Peripheral};

pub struct Can<'d, T: Instance> {
    can: bxcan::Can<BxcanInstance<'d, T>>,
}

impl<'d, T: Instance> Can<'d, T> {
    /// Creates a new Bxcan instance, blocking for 11 recessive bits to sync with the CAN bus.
    pub fn new(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
    ) -> Self {
        into_ref!(peri, rx, tx);

        unsafe {
            rx.set_as_af(rx.af_num(), AFType::Input);
            tx.set_as_af(tx.af_num(), AFType::OutputPushPull);
        }

        T::enable();
        T::reset();

        Self {
            can: bxcan::Can::builder(BxcanInstance(peri)).enable(),
        }
    }

    /// Creates a new Bxcan instance, keeping the peripheral in sleep mode.
    /// You must call [Can::enable_non_blocking] to use the peripheral.
    pub fn new_disabled(
        peri: impl Peripheral<P = T> + 'd,
        rx: impl Peripheral<P = impl RxPin<T>> + 'd,
        tx: impl Peripheral<P = impl TxPin<T>> + 'd,
        tx_irq: impl Peripheral<P = T::TXInterrupt> + 'd,
        rx0_irq: impl Peripheral<P = T::RX0Interrupt> + 'd,
        rx1_irq: impl Peripheral<P = T::RX1Interrupt> + 'd,
        sce_irq: impl Peripheral<P = T::SCEInterrupt> + 'd,
    ) -> Self {
        into_ref!(peri, rx, tx, tx_irq, rx0_irq, rx1_irq, sce_irq);

        unsafe {
            rx.set_as_af(rx.af_num(), AFType::Input);
            tx.set_as_af(tx.af_num(), AFType::OutputPushPull);
        }

        T::enable();
        T::reset();

        tx_irq.unpend();
        tx_irq.set_handler(Self::tx_interrupt);
        tx_irq.enable();

        rx0_irq.unpend();
        rx0_irq.set_handler(Self::rx0_interrupt);
        rx0_irq.enable();

        rx1_irq.unpend();
        rx1_irq.set_handler(Self::rx1_interrupt);
        rx1_irq.enable();

        sce_irq.unpend();
        sce_irq.set_handler(Self::sce_interrupt);
        sce_irq.enable();

        Self {
            can: bxcan::Can::builder(BxcanInstance(peri)).leave_disabled(),
        }
    }

    pub async fn transmit_async(&mut self, frame: &Frame) {
        let tx_status = self.queue_transmit(frame).await;
        self.wait_transission(tx_status.mailbox()).await;
    }

    async fn queue_transmit(&mut self, frame: &Frame) -> bxcan::TransmitStatus {
        poll_fn(|cx| {
            if let Ok(status) = self.can.transmit(frame) {
                return Poll::Ready(status);
            }
            T::state().tx_waker.register(cx.waker());
            Poll::Pending
        })
        .await
    }

    async fn wait_transission(&mut self, mb: bxcan::Mailbox) {
        poll_fn(|cx| unsafe {
            if T::regs().tsr().read().tme(mb.index()) {
                return Poll::Ready(());
            }
            T::state().tx_waker.register(cx.waker());
            Poll::Pending
        })
        .await;
    }

    unsafe fn tx_interrupt(_: *mut ()) {
        T::regs().tsr().write(|v| {
            v.set_rqcp(0, true);
            v.set_rqcp(1, true);
            v.set_rqcp(2, true);
        });
        T::state().tx_waker.wake();
    }

    unsafe fn rx0_interrupt(_: *mut ()) {}

    unsafe fn rx1_interrupt(_: *mut ()) {}

    unsafe fn sce_interrupt(_: *mut ()) {}
}

impl<'d, T: Instance> Drop for Can<'d, T> {
    fn drop(&mut self) {
        // Cannot call `free()` because it moves the instance.
        // Manually reset the peripheral.
        unsafe { T::regs().mcr().write(|w| w.set_reset(true)) }
        T::disable();
    }
}

impl<'d, T: Instance> Deref for Can<'d, T> {
    type Target = bxcan::Can<BxcanInstance<'d, T>>;

    fn deref(&self) -> &Self::Target {
        &self.can
    }
}

impl<'d, T: Instance> DerefMut for Can<'d, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.can
    }
}

pub(crate) mod sealed {
    use embassy_sync::waitqueue::AtomicWaker;
    pub struct State {
        pub tx_waker: AtomicWaker,
        pub rx0_waker: AtomicWaker,
        pub rx1_waker: AtomicWaker,
        pub sce_waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                tx_waker: AtomicWaker::new(),
                rx0_waker: AtomicWaker::new(),
                rx1_waker: AtomicWaker::new(),
                sce_waker: AtomicWaker::new(),
            }
        }
    }

    pub trait Instance {
        const REGISTERS: *mut bxcan::RegisterBlock;

        fn regs() -> &'static crate::pac::can::Can;
        fn state() -> &'static State;
    }
}

pub trait TXInstance {
    type TXInterrupt: crate::interrupt::Interrupt;
}

pub trait RX0Instance {
    type RX0Interrupt: crate::interrupt::Interrupt;
}

pub trait RX1Instance {
    type RX1Interrupt: crate::interrupt::Interrupt;
}

pub trait SCEInstance {
    type SCEInterrupt: crate::interrupt::Interrupt;
}

pub trait InterruptableInstance: TXInstance + RX0Instance + RX1Instance + SCEInstance {}
pub trait Instance: sealed::Instance + RccPeripheral + InterruptableInstance + 'static {}

pub struct BxcanInstance<'a, T>(PeripheralRef<'a, T>);

unsafe impl<'d, T: Instance> bxcan::Instance for BxcanInstance<'d, T> {
    const REGISTERS: *mut bxcan::RegisterBlock = T::REGISTERS;
}

foreach_peripheral!(
    (can, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            const REGISTERS: *mut bxcan::RegisterBlock = crate::pac::$inst.0 as *mut _;

            fn regs() -> &'static crate::pac::can::Can {
                &crate::pac::$inst
            }

            fn state() -> &'static sealed::State {
                static STATE: sealed::State = sealed::State::new();
                &STATE
            }
        }

        impl Instance for peripherals::$inst {}

        foreach_interrupt!(
            ($inst,can,CAN,TX,$irq:ident) => {
                impl TXInstance for peripherals::$inst {
                    type TXInterrupt = crate::interrupt::$irq;
                }
            };
            ($inst,can,CAN,RX0,$irq:ident) => {
                impl RX0Instance for peripherals::$inst {
                    type RX0Interrupt = crate::interrupt::$irq;
                }
            };
            ($inst,can,CAN,RX1,$irq:ident) => {
                impl RX1Instance for peripherals::$inst {
                    type RX1Interrupt = crate::interrupt::$irq;
                }
            };
            ($inst,can,CAN,SCE,$irq:ident) => {
                impl SCEInstance for peripherals::$inst {
                    type SCEInterrupt = crate::interrupt::$irq;
                }
            };
        );

        impl InterruptableInstance for peripherals::$inst {}
    };
);

foreach_peripheral!(
    (can, CAN) => {
        unsafe impl<'d> bxcan::FilterOwner for BxcanInstance<'d, peripherals::CAN> {
            const NUM_FILTER_BANKS: u8 = 14;
        }
    };
    // CAN1 and CAN2 is a combination of master and slave instance.
    // CAN1 owns the filter bank and needs to be enabled in order
    // for CAN2 to receive messages.
    (can, CAN1) => {
        cfg_if::cfg_if! {
            if #[cfg(all(
                any(stm32l4, stm32f72, stm32f73),
                not(any(stm32l49, stm32l4a))
            ))] {
                // Most L4 devices and some F7 devices use the name "CAN1"
                // even if there is no "CAN2" peripheral.
                unsafe impl<'d> bxcan::FilterOwner for BxcanInstance<'d, peripherals::CAN1> {
                    const NUM_FILTER_BANKS: u8 = 14;
                }
            } else {
                unsafe impl<'d> bxcan::FilterOwner for BxcanInstance<'d, peripherals::CAN1> {
                    const NUM_FILTER_BANKS: u8 = 28;
                }
                unsafe impl<'d> bxcan::MasterInstance for BxcanInstance<'d, peripherals::CAN1> {}
            }
        }
    };
    (can, CAN3) => {
        unsafe impl<'d> bxcan::FilterOwner for BxcanInstance<'d, peripherals::CAN3> {
            const NUM_FILTER_BANKS: u8 = 14;
        }
    };
);

pin_trait!(RxPin, Instance);
pin_trait!(TxPin, Instance);

trait Index {
    fn index(&self) -> usize;
}

impl Index for bxcan::Mailbox {
    fn index(&self) -> usize {
        match self {
            bxcan::Mailbox::Mailbox0 => 0,
            bxcan::Mailbox::Mailbox1 => 1,
            bxcan::Mailbox::Mailbox2 => 2,
        }
    }
}

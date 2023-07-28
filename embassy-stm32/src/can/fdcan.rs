pub use bxcan;
use embassy_hal_internal::PeripheralRef;

use crate::peripherals;

pub(crate) mod sealed {
    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::channel::Channel;
    use embassy_sync::waitqueue::AtomicWaker;

    pub struct State {
        pub tx_waker: AtomicWaker,
        pub err_waker: AtomicWaker,
        pub rx_queue: Channel<CriticalSectionRawMutex, (u16, bxcan::Frame), 32>,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                tx_waker: AtomicWaker::new(),
                err_waker: AtomicWaker::new(),
                rx_queue: Channel::new(),
            }
        }
    }

    pub trait Instance {
        const REGISTERS: *mut bxcan::RegisterBlock;

        fn regs() -> &'static crate::pac::can::Fdcan;
        fn state() -> &'static State;
    }
}

pub trait InterruptableInstance {}
pub trait Instance: sealed::Instance + InterruptableInstance + 'static {}

pub struct BxcanInstance<'a, T>(PeripheralRef<'a, T>);

unsafe impl<'d, T: Instance> bxcan::Instance for BxcanInstance<'d, T> {
    const REGISTERS: *mut bxcan::RegisterBlock = T::REGISTERS;
}

foreach_peripheral!(
    (can, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
            const REGISTERS: *mut bxcan::RegisterBlock = crate::pac::$inst.as_ptr() as *mut _;

            fn regs() -> &'static crate::pac::can::Fdcan {
                &crate::pac::$inst
            }

            fn state() -> &'static sealed::State {
                static STATE: sealed::State = sealed::State::new();
                &STATE
            }
        }

        impl Instance for peripherals::$inst {}

        impl InterruptableInstance for peripherals::$inst {}
    };
);

pin_trait!(RxPin, Instance);
pin_trait!(TxPin, Instance);

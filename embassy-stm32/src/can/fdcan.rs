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
        fn regs() -> &'static crate::pac::can::Fdcan;
        fn state() -> &'static State;
    }
}

/// Interruptable FDCAN instance.
pub trait InterruptableInstance {}
/// FDCAN instance.
pub trait Instance: sealed::Instance + InterruptableInstance + 'static {}

foreach_peripheral!(
    (can, $inst:ident) => {
        impl sealed::Instance for peripherals::$inst {
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

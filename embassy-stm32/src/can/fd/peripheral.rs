use embassy_hal_internal::PeripheralRef;

use super::{low_level::CanLowLevel, State};
use crate::interrupt::typelevel::Interrupt;
use crate::{can::Timestamp, peripherals, rcc::RccPeripheral};

#[allow(dead_code)]
pub(super) struct Info {
    pub low: CanLowLevel,
    pub interrupt0: crate::interrupt::Interrupt,
    pub _interrupt1: crate::interrupt::Interrupt,
    pub tx_waker: fn(),
}

unsafe impl Sync for Info {}

impl Info {
    pub fn calc_timestamp(&self, ns_per_timer_tick: u64, ts_val: u16) -> Timestamp {
        #[cfg(feature = "time")]
        {
            let now_embassy = embassy_time::Instant::now();
            if ns_per_timer_tick == 0 {
                return now_embassy;
            }
            let cantime = { self.low.regs.tscv().read().tsc() };
            let delta = cantime.overflowing_sub(ts_val).0 as u64;
            let ns = ns_per_timer_tick * delta as u64;
            now_embassy - embassy_time::Duration::from_nanos(ns)
        }

        #[cfg(not(feature = "time"))]
        {
            let _ = ns_per_timer_tick;
            ts_val
        }
    }
}

pub(super) trait SealedInstance {
    const MSG_RAM_OFFSET: usize;

    fn info() -> &'static Info;
    unsafe fn mut_info() -> &'static mut Info;
    fn state() -> &'static State;
    unsafe fn mut_state() -> &'static mut State;
}

/// Instance trait
#[allow(private_bounds)]
pub trait Instance: SealedInstance + RccPeripheral + 'static {
    /// Interrupt 0
    type IT0Interrupt: crate::interrupt::typelevel::Interrupt;
    /// Interrupt 1
    type IT1Interrupt: crate::interrupt::typelevel::Interrupt;
}

/// Fdcan Instance struct
pub struct FdcanInstance<'a, T>(PeripheralRef<'a, T>);

macro_rules! impl_fdcan {
    ($inst:ident,
        //$irq0:ident, $irq1:ident,
        $msg_ram_inst:ident, $msg_ram_offset:literal, $msg_ram_size:literal) => {

        #[allow(non_snake_case)]
        pub(crate) mod $inst {
            use super::*;

            static mut INFO: Info = Info {
                low: unsafe { CanLowLevel::new(crate::pac::$inst, crate::pac::$msg_ram_inst, $msg_ram_offset, $msg_ram_size) },
                interrupt0: crate::_generated::peripheral_interrupts::$inst::IT0::IRQ,
                _interrupt1: crate::_generated::peripheral_interrupts::$inst::IT1::IRQ,
                tx_waker: crate::_generated::peripheral_interrupts::$inst::IT0::pend,
            };

            impl SealedInstance for peripherals::$inst {
                const MSG_RAM_OFFSET: usize = $msg_ram_offset;

                unsafe fn mut_info() -> &'static mut Info {
                    &mut *core::ptr::addr_of_mut!(INFO)
                }

                fn info() -> &'static Info {
                    unsafe { peripherals::$inst::mut_info() }
                }

                unsafe fn mut_state() -> &'static mut State {
                    static mut STATE: State = State::new(unsafe { &*core::ptr::addr_of!(INFO) });
                    &mut *core::ptr::addr_of_mut!(STATE)
                }
                fn state() -> &'static State {
                    unsafe { peripherals::$inst::mut_state() }
                }

            }

            foreach_interrupt!(
                ($inst,can,FDCAN,IT0,$irq:ident) => {
                    pub type Interrupt0 = crate::interrupt::typelevel::$irq;
                };
                ($inst,can,FDCAN,IT1,$irq:ident) => {
                    pub type Interrupt1 = crate::interrupt::typelevel::$irq;
                };
            );

            impl Instance for peripherals::$inst {
                type IT0Interrupt = $inst::Interrupt0;
                type IT1Interrupt = $inst::Interrupt1;
            }
        }
    };
}

#[cfg(not(can_fdcan_h7))]
foreach_peripheral!(
    (can, FDCAN) => { impl_fdcan!(FDCAN, FDCANRAM, 0x0000, 0x03FF); };
    (can, FDCAN1) => { impl_fdcan!(FDCAN1, FDCANRAM1, 0x0000, 0x03FF); };
    (can, FDCAN2) => { impl_fdcan!(FDCAN2, FDCANRAM2, 0x0000, 0x03FF); };
    (can, FDCAN3) => { impl_fdcan!(FDCAN3, FDCANRAM3, 0x0000, 0x03FF); };
);

#[cfg(can_fdcan_h7)]
foreach_peripheral!(
    (can, FDCAN1) => { impl_fdcan!(FDCAN1, FDCANRAM, 0x0000, 0x0BFF); };
    (can, FDCAN2) => { impl_fdcan!(FDCAN2, FDCANRAM, 0x0C00, 0x0BFF); };
    (can, FDCAN3) => { impl_fdcan!(FDCAN3, FDCANRAM, 0x1800, 0x0BFF); };
);

pin_trait!(RxPin, Instance);
pin_trait!(TxPin, Instance);

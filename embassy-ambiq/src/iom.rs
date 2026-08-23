//! IOM (IO Master) peripheral driver

use apollo3_pac::interrupt;
use embassy_hal_internal::{Peri, PeripheralType};
use embassy_sync::waitqueue::AtomicWaker;

use crate::pac;

pub struct State {
    pub waker: AtomicWaker,
}

impl State {
    pub const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
        }
    }
}

pub trait Instance: PeripheralType + 'static + Send {
    fn regs() -> pac::iom0::Iom0;
    fn state() -> &'static State;
    fn enable_power();
    fn enable_interrupts();
    fn disable_interrupts();
}

macro_rules! impl_iom {
    ($inst:ident, $pac_inst:ident, $state:ident, $pwr:ident, $pwrstat:ident, $irq:ident) => {
        static $state: State = State::new();
        impl Instance for crate::peripherals::$inst {
            fn regs() -> pac::iom0::Iom0 {
                pac::$pac_inst
            }
            fn state() -> &'static State {
                &$state
            }
            fn enable_power() {
                pac::PWRCTRL.devpwren().modify(|w| w.$pwr(true));
                // Wait for the correct power domain to come up.
                // HCPB powers IOM0/1/2; HCPC powers IOM3/4/5. (HCPA is UART/IOSLAVE.)
                while !pac::PWRCTRL.devpwrstatus().read().$pwrstat() {}
            }
            fn enable_interrupts() {
                unsafe { cortex_m::peripheral::NVIC::unmask(pac::Interrupt::$irq) };
            }
            fn disable_interrupts() {
                cortex_m::peripheral::NVIC::mask(pac::Interrupt::$irq);
            }
        }

        #[pac::interrupt]
        fn $irq() {
            let regs = <crate::peripherals::$inst as Instance>::regs();
            let state = <crate::peripherals::$inst as Instance>::state();

            let intstat = regs.intstat().read();
            if intstat.cmdcmp() {
                regs.intclr().write(|w| w.set_cmdcmp(true));
                regs.inten().modify(|w| w.set_cmdcmp(false)); // Disable until next wait
                state.waker.wake();
            }
        }
    };
}

impl_iom!(IOM0, IOM0, IOM0_STATE, set_pwriom0, hcpb, IOMSTR0);
impl_iom!(IOM1, IOM1, IOM1_STATE, set_pwriom1, hcpb, IOMSTR1);
impl_iom!(IOM2, IOM2, IOM2_STATE, set_pwriom2, hcpb, IOMSTR2);
impl_iom!(IOM3, IOM3, IOM3_STATE, set_pwriom3, hcpc, IOMSTR3);
impl_iom!(IOM4, IOM4, IOM4_STATE, set_pwriom4, hcpc, IOMSTR4);
impl_iom!(IOM5, IOM5, IOM5_STATE, set_pwriom5, hcpc, IOMSTR5);

pub struct Iom<'d, T: Instance> {
    #[allow(dead_code)]
    peri: Peri<'d, T>,
}

impl<'d, T: Instance> Iom<'d, T> {
    pub fn new(peri: Peri<'d, T>) -> Self {
        T::enable_power();
        T::enable_interrupts();
        Self { peri }
    }
}

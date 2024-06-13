//! LTDC
use core::marker::PhantomData;

use crate::rcc::{self, RccPeripheral};
use crate::{peripherals, Peripheral};

/// LTDC driver.
pub struct Ltdc<'d, T: Instance> {
    _peri: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Ltdc<'d, T> {
    /// Note: Full-Duplex modes are not supported at this time
    pub fn new(
        _peri: impl Peripheral<P = T> + 'd,
        /*
        clk: impl Peripheral<P = impl ClkPin<T>> + 'd,
        hsync: impl Peripheral<P = impl HsyncPin<T>> + 'd,
        vsync: impl Peripheral<P = impl VsyncPin<T>> + 'd,
        b0: impl Peripheral<P = impl B0Pin<T>> + 'd,
        b1: impl Peripheral<P = impl B1Pin<T>> + 'd,
        b2: impl Peripheral<P = impl B2Pin<T>> + 'd,
        b3: impl Peripheral<P = impl B3Pin<T>> + 'd,
        b4: impl Peripheral<P = impl B4Pin<T>> + 'd,
        b5: impl Peripheral<P = impl B5Pin<T>> + 'd,
        b6: impl Peripheral<P = impl B6Pin<T>> + 'd,
        b7: impl Peripheral<P = impl B7Pin<T>> + 'd,
        g0: impl Peripheral<P = impl G0Pin<T>> + 'd,
        g1: impl Peripheral<P = impl G1Pin<T>> + 'd,
        g2: impl Peripheral<P = impl G2Pin<T>> + 'd,
        g3: impl Peripheral<P = impl G3Pin<T>> + 'd,
        g4: impl Peripheral<P = impl G4Pin<T>> + 'd,
        g5: impl Peripheral<P = impl G5Pin<T>> + 'd,
        g6: impl Peripheral<P = impl G6Pin<T>> + 'd,
        g7: impl Peripheral<P = impl G7Pin<T>> + 'd,
        r0: impl Peripheral<P = impl R0Pin<T>> + 'd,
        r1: impl Peripheral<P = impl R1Pin<T>> + 'd,
        r2: impl Peripheral<P = impl R2Pin<T>> + 'd,
        r3: impl Peripheral<P = impl R3Pin<T>> + 'd,
        r4: impl Peripheral<P = impl R4Pin<T>> + 'd,
        r5: impl Peripheral<P = impl R5Pin<T>> + 'd,
        r6: impl Peripheral<P = impl R6Pin<T>> + 'd,
        r7: impl Peripheral<P = impl R7Pin<T>> + 'd,
        */
    ) -> Self {
        //into_ref!(clk);

        critical_section::with(|_cs| {
            // RM says the pllsaidivr should only be changed when pllsai is off. But this could have other unintended side effects. So let's just give it a try like this.
            // According to the debugger, this bit gets set, anyway.
            #[cfg(stm32f7)]
            stm32_metapac::RCC
                .dckcfgr1()
                .modify(|w| w.set_pllsaidivr(stm32_metapac::rcc::vals::Pllsaidivr::DIV2));

            // It is set to RCC_PLLSAIDIVR_2 in ST's BSP example for the STM32469I-DISCO.
            #[cfg(not(any(stm32f7, stm32u5)))]
            stm32_metapac::RCC
                .dckcfgr()
                .modify(|w| w.set_pllsaidivr(stm32_metapac::rcc::vals::Pllsaidivr::DIV2));
        });

        rcc::enable_and_reset::<T>();

        //new_pin!(clk, AFType::OutputPushPull, Speed::VeryHigh,  Pull::None);

        // Set Tearing Enable pin according to CubeMx example
        //te.set_as_af_pull(te.af_num(), AFType::OutputPushPull, Pull::None);
        //te.set_speed(Speed::Low);
        /*
                T::regs().wcr().modify(|w| {
                    w.set_dsien(true);
                });
        */
        Self { _peri: PhantomData }
    }

    /// Set the enable bit in the control register and assert that it has been enabled
    pub fn enable(&mut self) {
        T::regs().gcr().modify(|w| w.set_ltdcen(true));
        assert!(T::regs().gcr().read().ltdcen())
    }

    /// Unset the enable bit in the control register and assert that it has been disabled
    pub fn disable(&mut self) {
        T::regs().gcr().modify(|w| w.set_ltdcen(false));
        assert!(!T::regs().gcr().read().ltdcen())
    }
}

impl<'d, T: Instance> Drop for Ltdc<'d, T> {
    fn drop(&mut self) {}
}

trait SealedInstance: crate::rcc::SealedRccPeripheral {
    fn regs() -> crate::pac::ltdc::Ltdc;
}

/// DSI instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + RccPeripheral + 'static {}

pin_trait!(ClkPin, Instance);
pin_trait!(HsyncPin, Instance);
pin_trait!(VsyncPin, Instance);
pin_trait!(DePin, Instance);
pin_trait!(R0Pin, Instance);
pin_trait!(R1Pin, Instance);
pin_trait!(R2Pin, Instance);
pin_trait!(R3Pin, Instance);
pin_trait!(R4Pin, Instance);
pin_trait!(R5Pin, Instance);
pin_trait!(R6Pin, Instance);
pin_trait!(R7Pin, Instance);
pin_trait!(G0Pin, Instance);
pin_trait!(G1Pin, Instance);
pin_trait!(G2Pin, Instance);
pin_trait!(G3Pin, Instance);
pin_trait!(G4Pin, Instance);
pin_trait!(G5Pin, Instance);
pin_trait!(G6Pin, Instance);
pin_trait!(G7Pin, Instance);
pin_trait!(B0Pin, Instance);
pin_trait!(B1Pin, Instance);
pin_trait!(B2Pin, Instance);
pin_trait!(B3Pin, Instance);
pin_trait!(B4Pin, Instance);
pin_trait!(B5Pin, Instance);
pin_trait!(B6Pin, Instance);
pin_trait!(B7Pin, Instance);

foreach_peripheral!(
    (ltdc, $inst:ident) => {
        impl crate::ltdc::SealedInstance for peripherals::$inst {
            fn regs() -> crate::pac::ltdc::Ltdc {
                crate::pac::$inst
            }
        }

        impl crate::ltdc::Instance for peripherals::$inst {}
    };
);

use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;

use crate::gpio::sealed::AFType;
use crate::gpio::{Pull, Speed};

mod pins;
pub use pins::*;

pub struct Fmc<'d, T: Instance> {
    peri: PhantomData<&'d mut T>,
}

unsafe impl<'d, T> Send for Fmc<'d, T> where T: Instance {}

unsafe impl<'d, T> stm32_fmc::FmcPeripheral for Fmc<'d, T>
where
    T: Instance,
{
    const REGISTERS: *const () = crate::pac::FMC.0 as *const _;

    fn enable(&mut self) {
        <T as crate::rcc::sealed::RccPeripheral>::enable();
        <T as crate::rcc::sealed::RccPeripheral>::reset();
    }

    fn memory_controller_enable(&mut self) {
        // The FMCEN bit of the FMC_BCR2..4 registers is don’t
        // care. It is only enabled through the FMC_BCR1 register.
        unsafe { T::regs().bcr1().modify(|r| r.set_fmcen(true)) };
    }

    fn source_clock_hz(&self) -> u32 {
        <T as crate::rcc::sealed::RccPeripheral>::frequency().0
    }
}

macro_rules! config_pins {
    ($($pin:ident),*) => {
        unborrow!($($pin),*);
        $(
            $pin.set_as_af_pull($pin.af_num(), AFType::OutputPushPull, Pull::Up);
            $pin.set_speed(Speed::VeryHigh);
        )*
    };
}

macro_rules! fmc_sdram_constructor {
    ($name:ident: (
        bank: $bank:expr,
        addr: [$(($addr_pin_name:ident: $addr_signal:ident)),*],
        ba: [$(($ba_pin_name:ident: $ba_signal:ident)),*],
        d: [$(($d_pin_name:ident: $d_signal:ident)),*],
        nbl: [$(($nbl_pin_name:ident: $nbl_signal:ident)),*],
        ctrl: [$(($ctrl_pin_name:ident: $ctrl_signal:ident)),*]
    )) => {
        pub fn $name<CHIP: stm32_fmc::SdramChip>(
            _instance: impl Unborrow<Target = T> + 'd,
            $($addr_pin_name: impl Unborrow<Target = impl $addr_signal<T>> + 'd),*,
            $($ba_pin_name: impl Unborrow<Target = impl $ba_signal<T>> + 'd),*,
            $($d_pin_name: impl Unborrow<Target = impl $d_signal<T>> + 'd),*,
            $($nbl_pin_name: impl Unborrow<Target = impl $nbl_signal<T>> + 'd),*,
            $($ctrl_pin_name: impl Unborrow<Target = impl $ctrl_signal<T>> + 'd),*,
            chip: CHIP
        ) -> stm32_fmc::Sdram<Fmc<'d, T>, CHIP> {

        critical_section::with(|_| unsafe {
            config_pins!(
                $($addr_pin_name),*,
                $($ba_pin_name),*,
                $($d_pin_name),*,
                $($nbl_pin_name),*,
                $($ctrl_pin_name),*
            );
        });

            let fmc = Self { peri: PhantomData };
            stm32_fmc::Sdram::new_unchecked(
                fmc,
                $bank,
                chip,
            )
        }
    };
}

impl<'d, T: Instance> Fmc<'d, T> {
    fmc_sdram_constructor!(sdram_a12bits_d32bits_4banks_bank1: (
        bank: stm32_fmc::SdramTargetBank::Bank1,
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin),
            (d16: D16Pin), (d17: D17Pin), (d18: D18Pin), (d19: D19Pin), (d20: D20Pin), (d21: D21Pin), (d22: D22Pin), (d23: D23Pin),
            (d24: D24Pin), (d25: D25Pin), (d26: D26Pin), (d27: D27Pin), (d28: D28Pin), (d29: D29Pin), (d30: D30Pin), (d31: D31Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin), (nbl2: NBL2Pin), (nbl3: NBL3Pin)
        ],
        ctrl: [
            (sdcke: SDCKE0Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne: SDNE0Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
        ]
    ));

    fmc_sdram_constructor!(sdram_a12bits_d32bits_4banks_bank2: (
        bank: stm32_fmc::SdramTargetBank::Bank2,
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin),
            (d16: D16Pin), (d17: D17Pin), (d18: D18Pin), (d19: D19Pin), (d20: D20Pin), (d21: D21Pin), (d22: D22Pin), (d23: D23Pin),
            (d24: D24Pin), (d25: D25Pin), (d26: D26Pin), (d27: D27Pin), (d28: D28Pin), (d29: D29Pin), (d30: D30Pin), (d31: D31Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin), (nbl2: NBL2Pin), (nbl3: NBL3Pin)
        ],
        ctrl: [
            (sdcke: SDCKE1Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne: SDNE1Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
        ]
    ));
}

foreach_peripheral!(
    (fmc, $inst:ident) => {
        impl crate::fmc::sealed::Instance for crate::peripherals::$inst {
            fn regs() -> stm32_metapac::fmc::Fmc {
                crate::pac::$inst
            }
        }
        impl crate::fmc::Instance for crate::peripherals::$inst {}
    };
);

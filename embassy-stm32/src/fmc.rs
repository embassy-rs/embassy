//! Flexible Memory Controller (FMC) / Flexible Static Memory Controller (FSMC)
use core::marker::PhantomData;

use embassy_hal_internal::into_ref;

use crate::gpio::{AFType, Pull, Speed};
use crate::{rcc, Peripheral};

/// FMC driver
pub struct Fmc<'d, T: Instance> {
    peri: PhantomData<&'d mut T>,
}

unsafe impl<'d, T> Send for Fmc<'d, T> where T: Instance {}

impl<'d, T> Fmc<'d, T>
where
    T: Instance,
{
    /// Create a raw FMC instance.
    ///
    /// **Note:** This is currently used to provide access to some basic FMC functions
    /// for manual configuration for memory types that stm32-fmc does not support.
    pub fn new_raw(_instance: impl Peripheral<P = T> + 'd) -> Self {
        Self { peri: PhantomData }
    }

    /// Enable the FMC peripheral and reset it.
    pub fn enable(&mut self) {
        rcc::enable_and_reset::<T>();
    }

    /// Enable the memory controller on applicable chips.
    pub fn memory_controller_enable(&mut self) {
        // fmc v1 and v2 does not have the fmcen bit
        // fsmc v1, v2 and v3 does not have the fmcen bit
        // This is a "not" because it is expected that all future versions have this bit
        #[cfg(not(any(fmc_v1x3, fmc_v2x1, fsmc_v1x0, fsmc_v1x3, fmc_v4)))]
        T::REGS.bcr1().modify(|r| r.set_fmcen(true));
        #[cfg(any(fmc_v4))]
        T::REGS.nor_psram().bcr1().modify(|r| r.set_fmcen(true));
    }

    /// Get the kernel clock currently in use for this FMC instance.
    pub fn source_clock_hz(&self) -> u32 {
        <T as crate::rcc::SealedRccPeripheral>::frequency().0
    }
}

unsafe impl<'d, T> stm32_fmc::FmcPeripheral for Fmc<'d, T>
where
    T: Instance,
{
    const REGISTERS: *const () = T::REGS.as_ptr() as *const _;

    fn enable(&mut self) {
        rcc::enable_and_reset::<T>();
    }

    fn memory_controller_enable(&mut self) {
        // fmc v1 and v2 does not have the fmcen bit
        // fsmc v1, v2 and v3 does not have the fmcen bit
        // This is a "not" because it is expected that all future versions have this bit
        #[cfg(not(any(fmc_v1x3, fmc_v2x1, fsmc_v1x0, fsmc_v1x3, fmc_v4)))]
        T::REGS.bcr1().modify(|r| r.set_fmcen(true));
        #[cfg(any(fmc_v4))]
        T::REGS.nor_psram().bcr1().modify(|r| r.set_fmcen(true));
    }

    fn source_clock_hz(&self) -> u32 {
        <T as crate::rcc::SealedRccPeripheral>::frequency().0
    }
}

macro_rules! config_pins {
    ($($pin:ident),*) => {
        into_ref!($($pin),*);
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
        /// Create a new FMC instance.
        pub fn $name<CHIP: stm32_fmc::SdramChip>(
            _instance: impl Peripheral<P = T> + 'd,
            $($addr_pin_name: impl Peripheral<P = impl $addr_signal<T>> + 'd),*,
            $($ba_pin_name: impl Peripheral<P = impl $ba_signal<T>> + 'd),*,
            $($d_pin_name: impl Peripheral<P = impl $d_signal<T>> + 'd),*,
            $($nbl_pin_name: impl Peripheral<P = impl $nbl_signal<T>> + 'd),*,
            $($ctrl_pin_name: impl Peripheral<P = impl $ctrl_signal<T>> + 'd),*,
            chip: CHIP
        ) -> stm32_fmc::Sdram<Fmc<'d, T>, CHIP> {

        critical_section::with(|_| {
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
    fmc_sdram_constructor!(sdram_a12bits_d16bits_4banks_bank1: (
        bank: stm32_fmc::SdramTargetBank::Bank1,
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin)
        ],
        ctrl: [
            (sdcke: SDCKE0Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne: SDNE0Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
        ]
    ));

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

    fmc_sdram_constructor!(sdram_a12bits_d16bits_4banks_bank2: (
        bank: stm32_fmc::SdramTargetBank::Bank2,
        addr: [
            (a0: A0Pin), (a1: A1Pin), (a2: A2Pin), (a3: A3Pin), (a4: A4Pin), (a5: A5Pin), (a6: A6Pin), (a7: A7Pin), (a8: A8Pin), (a9: A9Pin), (a10: A10Pin), (a11: A11Pin)
        ],
        ba: [(ba0: BA0Pin), (ba1: BA1Pin)],
        d: [
            (d0: D0Pin), (d1: D1Pin), (d2: D2Pin), (d3: D3Pin), (d4: D4Pin), (d5: D5Pin), (d6: D6Pin), (d7: D7Pin),
            (d8: D8Pin), (d9: D9Pin), (d10: D10Pin), (d11: D11Pin), (d12: D12Pin), (d13: D13Pin), (d14: D14Pin), (d15: D15Pin)
        ],
        nbl: [
            (nbl0: NBL0Pin), (nbl1: NBL1Pin)
        ],
        ctrl: [
            (sdcke: SDCKE1Pin), (sdclk: SDCLKPin), (sdncas: SDNCASPin), (sdne: SDNE1Pin), (sdnras: SDNRASPin), (sdnwe: SDNWEPin)
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

trait SealedInstance: crate::rcc::RccPeripheral {
    const REGS: crate::pac::fmc::Fmc;
}

/// FMC instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + 'static {}

foreach_peripheral!(
    (fmc, $inst:ident) => {
        impl crate::fmc::SealedInstance for crate::peripherals::$inst {
            const REGS: crate::pac::fmc::Fmc = crate::pac::$inst;
        }
        impl crate::fmc::Instance for crate::peripherals::$inst {}
    };
);

pin_trait!(SDNWEPin, Instance);
pin_trait!(SDNCASPin, Instance);
pin_trait!(SDNRASPin, Instance);

pin_trait!(SDNE0Pin, Instance);
pin_trait!(SDNE1Pin, Instance);

pin_trait!(SDCKE0Pin, Instance);
pin_trait!(SDCKE1Pin, Instance);

pin_trait!(SDCLKPin, Instance);

pin_trait!(NBL0Pin, Instance);
pin_trait!(NBL1Pin, Instance);
pin_trait!(NBL2Pin, Instance);
pin_trait!(NBL3Pin, Instance);

pin_trait!(INTPin, Instance);
pin_trait!(NLPin, Instance);
pin_trait!(NWaitPin, Instance);

pin_trait!(NE1Pin, Instance);
pin_trait!(NE2Pin, Instance);
pin_trait!(NE3Pin, Instance);
pin_trait!(NE4Pin, Instance);

pin_trait!(NCEPin, Instance);
pin_trait!(NOEPin, Instance);
pin_trait!(NWEPin, Instance);
pin_trait!(ClkPin, Instance);

pin_trait!(BA0Pin, Instance);
pin_trait!(BA1Pin, Instance);

pin_trait!(D0Pin, Instance);
pin_trait!(D1Pin, Instance);
pin_trait!(D2Pin, Instance);
pin_trait!(D3Pin, Instance);
pin_trait!(D4Pin, Instance);
pin_trait!(D5Pin, Instance);
pin_trait!(D6Pin, Instance);
pin_trait!(D7Pin, Instance);
pin_trait!(D8Pin, Instance);
pin_trait!(D9Pin, Instance);
pin_trait!(D10Pin, Instance);
pin_trait!(D11Pin, Instance);
pin_trait!(D12Pin, Instance);
pin_trait!(D13Pin, Instance);
pin_trait!(D14Pin, Instance);
pin_trait!(D15Pin, Instance);
pin_trait!(D16Pin, Instance);
pin_trait!(D17Pin, Instance);
pin_trait!(D18Pin, Instance);
pin_trait!(D19Pin, Instance);
pin_trait!(D20Pin, Instance);
pin_trait!(D21Pin, Instance);
pin_trait!(D22Pin, Instance);
pin_trait!(D23Pin, Instance);
pin_trait!(D24Pin, Instance);
pin_trait!(D25Pin, Instance);
pin_trait!(D26Pin, Instance);
pin_trait!(D27Pin, Instance);
pin_trait!(D28Pin, Instance);
pin_trait!(D29Pin, Instance);
pin_trait!(D30Pin, Instance);
pin_trait!(D31Pin, Instance);

pin_trait!(DA0Pin, Instance);
pin_trait!(DA1Pin, Instance);
pin_trait!(DA2Pin, Instance);
pin_trait!(DA3Pin, Instance);
pin_trait!(DA4Pin, Instance);
pin_trait!(DA5Pin, Instance);
pin_trait!(DA6Pin, Instance);
pin_trait!(DA7Pin, Instance);
pin_trait!(DA8Pin, Instance);
pin_trait!(DA9Pin, Instance);
pin_trait!(DA10Pin, Instance);
pin_trait!(DA11Pin, Instance);
pin_trait!(DA12Pin, Instance);
pin_trait!(DA13Pin, Instance);
pin_trait!(DA14Pin, Instance);
pin_trait!(DA15Pin, Instance);

pin_trait!(A0Pin, Instance);
pin_trait!(A1Pin, Instance);
pin_trait!(A2Pin, Instance);
pin_trait!(A3Pin, Instance);
pin_trait!(A4Pin, Instance);
pin_trait!(A5Pin, Instance);
pin_trait!(A6Pin, Instance);
pin_trait!(A7Pin, Instance);
pin_trait!(A8Pin, Instance);
pin_trait!(A9Pin, Instance);
pin_trait!(A10Pin, Instance);
pin_trait!(A11Pin, Instance);
pin_trait!(A12Pin, Instance);
pin_trait!(A13Pin, Instance);
pin_trait!(A14Pin, Instance);
pin_trait!(A15Pin, Instance);
pin_trait!(A16Pin, Instance);
pin_trait!(A17Pin, Instance);
pin_trait!(A18Pin, Instance);
pin_trait!(A19Pin, Instance);
pin_trait!(A20Pin, Instance);
pin_trait!(A21Pin, Instance);
pin_trait!(A22Pin, Instance);
pin_trait!(A23Pin, Instance);
pin_trait!(A24Pin, Instance);
pin_trait!(A25Pin, Instance);

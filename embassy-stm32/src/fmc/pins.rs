pub(crate) mod sealed {
    pub trait Instance: crate::rcc::sealed::RccPeripheral {
        fn regs() -> crate::pac::fmc::Fmc;
    }

    macro_rules! declare_pin {
        ($name:ident) => {
            pub trait $name {
                fn configure(&mut self);
            }
        };
    }

    declare_pin!(SDNWEPin);
    declare_pin!(SDNCASPin);
    declare_pin!(SDNRASPin);

    declare_pin!(SDNE0Pin);
    declare_pin!(SDNE1Pin);

    declare_pin!(SDCKE0Pin);
    declare_pin!(SDCKE1Pin);

    declare_pin!(SDCLKPin);

    declare_pin!(NBL0Pin);
    declare_pin!(NBL1Pin);
    declare_pin!(NBL2Pin);
    declare_pin!(NBL3Pin);

    declare_pin!(INTPin);
    declare_pin!(NLPin);
    declare_pin!(NWaitPin);

    declare_pin!(NE1Pin);
    declare_pin!(NE2Pin);
    declare_pin!(NE3Pin);
    declare_pin!(NE4Pin);

    declare_pin!(NCEPin);
    declare_pin!(NOEPin);
    declare_pin!(NWEPin);
    declare_pin!(ClkPin);

    declare_pin!(BA0Pin);
    declare_pin!(BA1Pin);

    declare_pin!(D0Pin);
    declare_pin!(D1Pin);
    declare_pin!(D2Pin);
    declare_pin!(D3Pin);
    declare_pin!(D4Pin);
    declare_pin!(D5Pin);
    declare_pin!(D6Pin);
    declare_pin!(D7Pin);
    declare_pin!(D8Pin);
    declare_pin!(D9Pin);
    declare_pin!(D10Pin);
    declare_pin!(D11Pin);
    declare_pin!(D12Pin);
    declare_pin!(D13Pin);
    declare_pin!(D14Pin);
    declare_pin!(D15Pin);
    declare_pin!(D16Pin);
    declare_pin!(D17Pin);
    declare_pin!(D18Pin);
    declare_pin!(D19Pin);
    declare_pin!(D20Pin);
    declare_pin!(D21Pin);
    declare_pin!(D22Pin);
    declare_pin!(D23Pin);
    declare_pin!(D24Pin);
    declare_pin!(D25Pin);
    declare_pin!(D26Pin);
    declare_pin!(D27Pin);
    declare_pin!(D28Pin);
    declare_pin!(D29Pin);
    declare_pin!(D30Pin);
    declare_pin!(D31Pin);

    declare_pin!(DA0Pin);
    declare_pin!(DA1Pin);
    declare_pin!(DA2Pin);
    declare_pin!(DA3Pin);
    declare_pin!(DA4Pin);
    declare_pin!(DA5Pin);
    declare_pin!(DA6Pin);
    declare_pin!(DA7Pin);
    declare_pin!(DA8Pin);
    declare_pin!(DA9Pin);
    declare_pin!(DA10Pin);
    declare_pin!(DA11Pin);
    declare_pin!(DA12Pin);
    declare_pin!(DA13Pin);
    declare_pin!(DA14Pin);
    declare_pin!(DA15Pin);

    declare_pin!(A0Pin);
    declare_pin!(A1Pin);
    declare_pin!(A2Pin);
    declare_pin!(A3Pin);
    declare_pin!(A4Pin);
    declare_pin!(A5Pin);
    declare_pin!(A6Pin);
    declare_pin!(A7Pin);
    declare_pin!(A8Pin);
    declare_pin!(A9Pin);
    declare_pin!(A10Pin);
    declare_pin!(A11Pin);
    declare_pin!(A12Pin);
    declare_pin!(A13Pin);
    declare_pin!(A14Pin);
    declare_pin!(A15Pin);
    declare_pin!(A16Pin);
    declare_pin!(A17Pin);
    declare_pin!(A18Pin);
    declare_pin!(A19Pin);
    declare_pin!(A20Pin);
    declare_pin!(A21Pin);
    declare_pin!(A22Pin);
    declare_pin!(A23Pin);
    declare_pin!(A24Pin);
    declare_pin!(A25Pin);
}

macro_rules! declare_pin {
    ($name:ident, $fmc_pin:ident) => {
        pub trait $name: sealed::$name + stm32_fmc::$fmc_pin + 'static {}
    };
}

declare_pin!(SDNWEPin, SDNWE);
declare_pin!(SDNCASPin, SDNCAS);
declare_pin!(SDNRASPin, SDNRAS);

declare_pin!(SDNE0Pin, SDNE0);
declare_pin!(SDNE1Pin, SDNE1);

declare_pin!(SDCKE0Pin, SDCKE0);
declare_pin!(SDCKE1Pin, SDCKE1);

declare_pin!(SDCLKPin, SDCLK);

declare_pin!(NBL0Pin, NBL0);
declare_pin!(NBL1Pin, NBL1);
declare_pin!(NBL2Pin, NBL2);
declare_pin!(NBL3Pin, NBL3);

declare_pin!(INTPin, INT);
declare_pin!(NLPin, NL);
declare_pin!(NWaitPin, NWAIT);

declare_pin!(NE1Pin, NE1);
declare_pin!(NE2Pin, NE2);
declare_pin!(NE3Pin, NE3);
declare_pin!(NE4Pin, NE4);

declare_pin!(NCEPin, NCE);
declare_pin!(NOEPin, NOE);
declare_pin!(NWEPin, NWE);
declare_pin!(ClkPin, CLK);

declare_pin!(BA0Pin, BA0);
declare_pin!(BA1Pin, BA1);

declare_pin!(D0Pin, D0);
declare_pin!(D1Pin, D1);
declare_pin!(D2Pin, D2);
declare_pin!(D3Pin, D3);
declare_pin!(D4Pin, D4);
declare_pin!(D5Pin, D5);
declare_pin!(D6Pin, D6);
declare_pin!(D7Pin, D7);
declare_pin!(D8Pin, D8);
declare_pin!(D9Pin, D9);
declare_pin!(D10Pin, D10);
declare_pin!(D11Pin, D11);
declare_pin!(D12Pin, D12);
declare_pin!(D13Pin, D13);
declare_pin!(D14Pin, D14);
declare_pin!(D15Pin, D15);
declare_pin!(D16Pin, D16);
declare_pin!(D17Pin, D17);
declare_pin!(D18Pin, D18);
declare_pin!(D19Pin, D19);
declare_pin!(D20Pin, D20);
declare_pin!(D21Pin, D21);
declare_pin!(D22Pin, D22);
declare_pin!(D23Pin, D23);
declare_pin!(D24Pin, D24);
declare_pin!(D25Pin, D25);
declare_pin!(D26Pin, D26);
declare_pin!(D27Pin, D27);
declare_pin!(D28Pin, D28);
declare_pin!(D29Pin, D29);
declare_pin!(D30Pin, D30);
declare_pin!(D31Pin, D31);

declare_pin!(DA0Pin, DA0);
declare_pin!(DA1Pin, DA1);
declare_pin!(DA2Pin, DA2);
declare_pin!(DA3Pin, DA3);
declare_pin!(DA4Pin, DA4);
declare_pin!(DA5Pin, DA5);
declare_pin!(DA6Pin, DA6);
declare_pin!(DA7Pin, DA7);
declare_pin!(DA8Pin, DA8);
declare_pin!(DA9Pin, DA9);
declare_pin!(DA10Pin, DA10);
declare_pin!(DA11Pin, DA11);
declare_pin!(DA12Pin, DA12);
declare_pin!(DA13Pin, DA13);
declare_pin!(DA14Pin, DA14);
declare_pin!(DA15Pin, DA15);

declare_pin!(A0Pin, A0);
declare_pin!(A1Pin, A1);
declare_pin!(A2Pin, A2);
declare_pin!(A3Pin, A3);
declare_pin!(A4Pin, A4);
declare_pin!(A5Pin, A5);
declare_pin!(A6Pin, A6);
declare_pin!(A7Pin, A7);
declare_pin!(A8Pin, A8);
declare_pin!(A9Pin, A9);
declare_pin!(A10Pin, A10);
declare_pin!(A11Pin, A11);
declare_pin!(A12Pin, A12);
declare_pin!(A13Pin, A13);
declare_pin!(A14Pin, A14);
declare_pin!(A15Pin, A15);
declare_pin!(A16Pin, A16);
declare_pin!(A17Pin, A17);
declare_pin!(A18Pin, A18);
declare_pin!(A19Pin, A19);
declare_pin!(A20Pin, A20);
declare_pin!(A21Pin, A21);
declare_pin!(A22Pin, A22);
declare_pin!(A23Pin, A23);
declare_pin!(A24Pin, A24);
declare_pin!(A25Pin, A25);

macro_rules! impl_pin {
    ($pin:ident, $signal:ident, $fmc_name:ident, $af:expr) => {
        impl sealed::$signal for crate::peripherals::$pin {
            fn configure(&mut self) {
                use crate::gpio::sealed::{AFType::OutputPushPull, Pin as SealedPin};
                use crate::gpio::Pin;
                use crate::gpio::Speed;
                use crate::pac::gpio::vals::Pupdr;

                critical_section::with(|_| unsafe {
                    self.set_as_af($af, OutputPushPull);
                    self.set_speed(Speed::VeryHigh);

                    self.block()
                        .pupdr()
                        .modify(|w| w.set_pupdr(self.pin() as usize, Pupdr::PULLUP));
                })
            }
        }

        impl stm32_fmc::$fmc_name for crate::peripherals::$pin {}

        impl $signal for crate::peripherals::$pin {}
    };
}

crate::pac::peripheral_pins!(
    ($inst:ident, fmc, FMC, $pin:ident, A0, $af:expr) => {
        impl_pin!($pin, A0Pin, A0, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A1, $af:expr) => {
        impl_pin!($pin, A1Pin, A1, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A2, $af:expr) => {
        impl_pin!($pin, A2Pin, A2, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A3, $af:expr) => {
        impl_pin!($pin, A3Pin, A3, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A4, $af:expr) => {
        impl_pin!($pin, A4Pin, A4, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A5, $af:expr) => {
        impl_pin!($pin, A5Pin, A5, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A6, $af:expr) => {
        impl_pin!($pin, A6Pin, A6, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A7, $af:expr) => {
        impl_pin!($pin, A7Pin, A7, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A8, $af:expr) => {
        impl_pin!($pin, A8Pin, A8, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A9, $af:expr) => {
        impl_pin!($pin, A9Pin, A9, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A10, $af:expr) => {
        impl_pin!($pin, A10Pin, A10, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A11, $af:expr) => {
        impl_pin!($pin, A11Pin, A11, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A12, $af:expr) => {
        impl_pin!($pin, A12Pin, A12, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A13, $af:expr) => {
        impl_pin!($pin, A13Pin, A13, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A14, $af:expr) => {
        impl_pin!($pin, A14Pin, A14, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A15, $af:expr) => {
        impl_pin!($pin, A15Pin, A15, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A16, $af:expr) => {
        impl_pin!($pin, A16Pin, A16, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A17, $af:expr) => {
        impl_pin!($pin, A17Pin, A17, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A18, $af:expr) => {
        impl_pin!($pin, A18Pin, A18, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A19, $af:expr) => {
        impl_pin!($pin, A19Pin, A19, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A20, $af:expr) => {
        impl_pin!($pin, A20Pin, A20, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A21, $af:expr) => {
        impl_pin!($pin, A21Pin, A21, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A22, $af:expr) => {
        impl_pin!($pin, A22Pin, A22, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A23, $af:expr) => {
        impl_pin!($pin, A23Pin, A23, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A24, $af:expr) => {
        impl_pin!($pin, A24Pin, A24, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, A25, $af:expr) => {
        impl_pin!($pin, A25Pin, A25, $af);
    };
);

crate::pac::peripheral_pins!(
    ($inst:ident, fmc, FMC, $pin:ident, D0, $af:expr) => {
        impl_pin!($pin, D0Pin, D0, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D1, $af:expr) => {
        impl_pin!($pin, D1Pin, D1, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D2, $af:expr) => {
        impl_pin!($pin, D2Pin, D2, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D3, $af:expr) => {
        impl_pin!($pin, D3Pin, D3, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D4, $af:expr) => {
        impl_pin!($pin, D4Pin, D4, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D5, $af:expr) => {
        impl_pin!($pin, D5Pin, D5, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D6, $af:expr) => {
        impl_pin!($pin, D6Pin, D6, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D7, $af:expr) => {
        impl_pin!($pin, D7Pin, D7, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D8, $af:expr) => {
        impl_pin!($pin, D8Pin, D8, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D9, $af:expr) => {
        impl_pin!($pin, D9Pin, D9, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D10, $af:expr) => {
        impl_pin!($pin, D10Pin, D10, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D11, $af:expr) => {
        impl_pin!($pin, D11Pin, D11, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D12, $af:expr) => {
        impl_pin!($pin, D12Pin, D12, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D13, $af:expr) => {
        impl_pin!($pin, D13Pin, D13, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D14, $af:expr) => {
        impl_pin!($pin, D14Pin, D14, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D15, $af:expr) => {
        impl_pin!($pin, D15Pin, D15, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D16, $af:expr) => {
        impl_pin!($pin, D16Pin, D16, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D17, $af:expr) => {
        impl_pin!($pin, D17Pin, D17, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D18, $af:expr) => {
        impl_pin!($pin, D18Pin, D18, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D19, $af:expr) => {
        impl_pin!($pin, D19Pin, D19, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D20, $af:expr) => {
        impl_pin!($pin, D20Pin, D20, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D21, $af:expr) => {
        impl_pin!($pin, D21Pin, D21, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D22, $af:expr) => {
        impl_pin!($pin, D22Pin, D22, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D23, $af:expr) => {
        impl_pin!($pin, D23Pin, D23, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D24, $af:expr) => {
        impl_pin!($pin, D24Pin, D24, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D25, $af:expr) => {
        impl_pin!($pin, D25Pin, D25, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D26, $af:expr) => {
        impl_pin!($pin, D26Pin, D26, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D27, $af:expr) => {
        impl_pin!($pin, D27Pin, D27, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D28, $af:expr) => {
        impl_pin!($pin, D28Pin, D28, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D29, $af:expr) => {
        impl_pin!($pin, D29Pin, D29, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D30, $af:expr) => {
        impl_pin!($pin, D30Pin, D30, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, D31, $af:expr) => {
        impl_pin!($pin, D31Pin, D31, $af);
    };
);

crate::pac::peripheral_pins!(
    ($inst:ident, fmc, FMC, $pin:ident, DA0, $af:expr) => {
        impl_pin!($pin, DA0Pin, DA0, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, DA1, $af:expr) => {
        impl_pin!($pin, DA1Pin, DA1, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, DA2, $af:expr) => {
        impl_pin!($pin, DA2Pin, DA2, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, DA3, $af:expr) => {
        impl_pin!($pin, DA3Pin, DA3, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, DA4, $af:expr) => {
        impl_pin!($pin, DA4Pin, DA4, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, DA5, $af:expr) => {
        impl_pin!($pin, DA5Pin, DA5, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, DA6, $af:expr) => {
        impl_pin!($pin, DA6Pin, DA6, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, DA7, $af:expr) => {
        impl_pin!($pin, DA7Pin, DA7, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, DA8, $af:expr) => {
        impl_pin!($pin, DA8Pin, DA8, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, DA9, $af:expr) => {
        impl_pin!($pin, DA9Pin, DA9, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, DA10, $af:expr) => {
        impl_pin!($pin, DA10Pin, DA10, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, DA11, $af:expr) => {
        impl_pin!($pin, DA11Pin, DA11, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, DA12, $af:expr) => {
        impl_pin!($pin, DA12Pin, DA12, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, DA13, $af:expr) => {
        impl_pin!($pin, DA13Pin, DA13, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, DA14, $af:expr) => {
        impl_pin!($pin, DA14Pin, DA14, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, DA15, $af:expr) => {
        impl_pin!($pin, DA15Pin, DA15, $af);
    };

);

crate::pac::peripheral_pins!(
    ($inst:ident, fmc, FMC, $pin:ident, SDNWE, $af:expr) => {
        impl_pin!($pin, SDNWEPin, SDNWE, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, SDNCAS, $af:expr) => {
        impl_pin!($pin, SDNCASPin, SDNCAS, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, SDNRAS, $af:expr) => {
        impl_pin!($pin, SDNRASPin, SDNRAS, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, SDNE0, $af:expr) => {
        impl_pin!($pin, SDNE0Pin, SDNE0, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, SDNE1, $af:expr) => {
        impl_pin!($pin, SDNE1Pin, SDNE1, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, SDCKE0, $af:expr) => {
        impl_pin!($pin, SDCKE0Pin, SDCKE0, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, SDCKE1, $af:expr) => {
        impl_pin!($pin, SDCKE1Pin, SDCKE1, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, SDCLK, $af:expr) => {
        impl_pin!($pin, SDCLKPin, SDCLK, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, NBL0, $af:expr) => {
        impl_pin!($pin, NBL0Pin, NBL0, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, NBL1, $af:expr) => {
        impl_pin!($pin, NBL1Pin, NBL1, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, NBL2, $af:expr) => {
        impl_pin!($pin, NBL2Pin, NBL2, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, NBL3, $af:expr) => {
        impl_pin!($pin, NBL3Pin, NBL3, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, INT, $af:expr) => {
        impl_pin!($pin, INTPin, INT, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, NL, $af:expr) => {
        impl_pin!($pin, NLPin, NL, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, NWAIT, $af:expr) => {
        impl_pin!($pin, NWaitPin, NWAIT, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, NE1, $af:expr) => {
        impl_pin!($pin, NE1Pin, NE1, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, NE2, $af:expr) => {
        impl_pin!($pin, NE2Pin, NE2, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, NE3, $af:expr) => {
        impl_pin!($pin, NE3Pin, NE3, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, NE4, $af:expr) => {
        impl_pin!($pin, NE4Pin, NE4, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, NCE, $af:expr) => {
        impl_pin!($pin, NCEPin, NCE, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, NOE, $af:expr) => {
        impl_pin!($pin, NOEPin, NOE, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, NWE, $af:expr) => {
        impl_pin!($pin, NWEPin, NWE, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, Clk, $af:expr) => {
        impl_pin!($pin, ClkPin, CLK, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, BA0, $af:expr) => {
        impl_pin!($pin, BA0Pin, BA0, $af);
    };

    ($inst:ident, fmc, FMC, $pin:ident, BA1, $af:expr) => {
        impl_pin!($pin, BA1Pin, BA1, $af);
    };
);

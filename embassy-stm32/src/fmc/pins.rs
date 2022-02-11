pub(crate) mod sealed {
    pub trait Instance: crate::rcc::sealed::RccPeripheral {
        fn regs() -> crate::pac::fmc::Fmc;
    }
}

pub trait Instance: sealed::Instance + 'static {}

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

crate::pac::peripheral_pins!(
    ($inst:ident, fmc, FMC, $pin:ident, A0, $af:expr) => {
        pin_trait_impl!(A0Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A1, $af:expr) => {
        pin_trait_impl!(A1Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A2, $af:expr) => {
        pin_trait_impl!(A2Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A3, $af:expr) => {
        pin_trait_impl!(A3Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A4, $af:expr) => {
        pin_trait_impl!(A4Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A5, $af:expr) => {
        pin_trait_impl!(A5Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A6, $af:expr) => {
        pin_trait_impl!(A6Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A7, $af:expr) => {
        pin_trait_impl!(A7Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A8, $af:expr) => {
        pin_trait_impl!(A8Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A9, $af:expr) => {
        pin_trait_impl!(A9Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A10, $af:expr) => {
        pin_trait_impl!(A10Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A11, $af:expr) => {
        pin_trait_impl!(A11Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A12, $af:expr) => {
        pin_trait_impl!(A12Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A13, $af:expr) => {
        pin_trait_impl!(A13Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A14, $af:expr) => {
        pin_trait_impl!(A14Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A15, $af:expr) => {
        pin_trait_impl!(A15Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A16, $af:expr) => {
        pin_trait_impl!(A16Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A17, $af:expr) => {
        pin_trait_impl!(A17Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A18, $af:expr) => {
        pin_trait_impl!(A18Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A19, $af:expr) => {
        pin_trait_impl!(A19Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A20, $af:expr) => {
        pin_trait_impl!(A20Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A21, $af:expr) => {
        pin_trait_impl!(A21Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A22, $af:expr) => {
        pin_trait_impl!(A22Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A23, $af:expr) => {
        pin_trait_impl!(A23Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A24, $af:expr) => {
        pin_trait_impl!(A24Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, A25, $af:expr) => {
        pin_trait_impl!(A25Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D0, $af:expr) => {
        pin_trait_impl!(D0Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D1, $af:expr) => {
        pin_trait_impl!(D1Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D2, $af:expr) => {
        pin_trait_impl!(D2Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D3, $af:expr) => {
        pin_trait_impl!(D3Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D4, $af:expr) => {
        pin_trait_impl!(D4Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D5, $af:expr) => {
        pin_trait_impl!(D5Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D6, $af:expr) => {
        pin_trait_impl!(D6Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D7, $af:expr) => {
        pin_trait_impl!(D7Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D8, $af:expr) => {
        pin_trait_impl!(D8Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D9, $af:expr) => {
        pin_trait_impl!(D9Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D10, $af:expr) => {
        pin_trait_impl!(D10Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D11, $af:expr) => {
        pin_trait_impl!(D11Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D12, $af:expr) => {
        pin_trait_impl!(D12Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D13, $af:expr) => {
        pin_trait_impl!(D13Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D14, $af:expr) => {
        pin_trait_impl!(D14Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D15, $af:expr) => {
        pin_trait_impl!(D15Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D16, $af:expr) => {
        pin_trait_impl!(D16Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D17, $af:expr) => {
        pin_trait_impl!(D17Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D18, $af:expr) => {
        pin_trait_impl!(D18Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D19, $af:expr) => {
        pin_trait_impl!(D19Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D20, $af:expr) => {
        pin_trait_impl!(D20Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D21, $af:expr) => {
        pin_trait_impl!(D21Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D22, $af:expr) => {
        pin_trait_impl!(D22Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D23, $af:expr) => {
        pin_trait_impl!(D23Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D24, $af:expr) => {
        pin_trait_impl!(D24Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D25, $af:expr) => {
        pin_trait_impl!(D25Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D26, $af:expr) => {
        pin_trait_impl!(D26Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D27, $af:expr) => {
        pin_trait_impl!(D27Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D28, $af:expr) => {
        pin_trait_impl!(D28Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D29, $af:expr) => {
        pin_trait_impl!(D29Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D30, $af:expr) => {
        pin_trait_impl!(D30Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, D31, $af:expr) => {
        pin_trait_impl!(D31Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA0, $af:expr) => {
        pin_trait_impl!(DA0Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA1, $af:expr) => {
        pin_trait_impl!(DA1Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA2, $af:expr) => {
        pin_trait_impl!(DA2Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA3, $af:expr) => {
        pin_trait_impl!(DA3Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA4, $af:expr) => {
        pin_trait_impl!(DA4Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA5, $af:expr) => {
        pin_trait_impl!(DA5Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA6, $af:expr) => {
        pin_trait_impl!(DA6Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA7, $af:expr) => {
        pin_trait_impl!(DA7Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA8, $af:expr) => {
        pin_trait_impl!(DA8Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA9, $af:expr) => {
        pin_trait_impl!(DA9Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA10, $af:expr) => {
        pin_trait_impl!(DA10Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA11, $af:expr) => {
        pin_trait_impl!(DA11Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA12, $af:expr) => {
        pin_trait_impl!(DA12Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA13, $af:expr) => {
        pin_trait_impl!(DA13Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA14, $af:expr) => {
        pin_trait_impl!(DA14Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, DA15, $af:expr) => {
        pin_trait_impl!(DA15Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, SDNWE, $af:expr) => {
        pin_trait_impl!(SDNWEPin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, SDNCAS, $af:expr) => {
        pin_trait_impl!(SDNCASPin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, SDNRAS, $af:expr) => {
        pin_trait_impl!(SDNRASPin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, SDNE0, $af:expr) => {
        pin_trait_impl!(SDNE0Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, SDNE1, $af:expr) => {
        pin_trait_impl!(SDNE1Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, SDCKE0, $af:expr) => {
        pin_trait_impl!(SDCKE0Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, SDCKE1, $af:expr) => {
        pin_trait_impl!(SDCKE1Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, SDCLK, $af:expr) => {
        pin_trait_impl!(SDCLKPin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, NBL0, $af:expr) => {
        pin_trait_impl!(NBL0Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, NBL1, $af:expr) => {
        pin_trait_impl!(NBL1Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, NBL2, $af:expr) => {
        pin_trait_impl!(NBL2Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, NBL3, $af:expr) => {
        pin_trait_impl!(NBL3Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, INT, $af:expr) => {
        pin_trait_impl!(INTPin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, NL, $af:expr) => {
        pin_trait_impl!(NLPin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, NWAIT, $af:expr) => {
        pin_trait_impl!(NWaitPin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, NE1, $af:expr) => {
        pin_trait_impl!(NE1Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, NE2, $af:expr) => {
        pin_trait_impl!(NE2Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, NE3, $af:expr) => {
        pin_trait_impl!(NE3Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, NE4, $af:expr) => {
        pin_trait_impl!(NE4Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, NCE, $af:expr) => {
        pin_trait_impl!(NCEPin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, NOE, $af:expr) => {
        pin_trait_impl!(NOEPin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, NWE, $af:expr) => {
        pin_trait_impl!(NWEPin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, Clk, $af:expr) => {
        pin_trait_impl!(ClkPin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, BA0, $af:expr) => {
        pin_trait_impl!(BA0Pin, $inst, $pin, $af);
    };
    ($inst:ident, fmc, FMC, $pin:ident, BA1, $af:expr) => {
        pin_trait_impl!(BA1Pin, $inst, $pin, $af);
    };
);

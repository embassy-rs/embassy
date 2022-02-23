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

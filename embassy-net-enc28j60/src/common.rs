#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Register {
    ECON1 = 0x1f,
    ECON2 = 0x1e,
    EIE = 0x1b,
    EIR = 0x1c,
    ESTAT = 0x1d,
}

impl Register {
    pub(crate) fn addr(&self) -> u8 {
        *self as u8
    }

    pub(crate) fn is_eth_register(&self) -> bool {
        match *self {
            Register::ECON1 => true,
            Register::ECON2 => true,
            Register::EIE => true,
            Register::EIR => true,
            Register::ESTAT => true,
        }
    }
}

impl Into<super::Register> for Register {
    fn into(self) -> super::Register {
        super::Register::Common(self)
    }
}

register!(EIE, 0, u8, {
    #[doc = "Receive Error Interrupt Enable bit"]
    rxerie @ 0,
    #[doc = "Transmit Error Interrupt Enable bit"]
    txerie @ 1,
    #[doc = "Transmit Enable bit"]
    txie @ 3,
    #[doc = "Link Status Change Interrupt Enable bit"]
    linkie @ 4,
    #[doc = "DMA Interrupt Enable bit"]
    dmaie @ 5,
    #[doc = "Receive Packet Pending Interrupt Enable bit"]
    pktie @ 6,
    #[doc = "Global INT Interrupt Enable bit"]
    intie @ 7,
});

register!(EIR, 0, u8, {
    #[doc = "Receive Error Interrupt Flag bit"]
    rxerif @ 0,
    #[doc = "Transmit Error Interrupt Flag bit"]
    txerif @ 1,
    #[doc = "Transmit Interrupt Flag bit"]
    txif @ 3,
    #[doc = "Link Change Interrupt Flag bit"]
    linkif @ 4,
    #[doc = "DMA Interrupt Flag bit"]
    dmaif @ 5,
    #[doc = "Receive Packet Pending Interrupt Flag bit"]
    pktif @ 6,
});

register!(ESTAT, 0, u8, {
    #[doc = "Clock Ready bit"]
    clkrdy @ 0,
    #[doc = "Transmit Abort Error bit"]
    txabrt @ 1,
    #[doc = "Receive Busy bit"]
    rxbusy @ 2,
    #[doc = "Late Collision Error bit"]
    latecol @ 4,
    #[doc = "Ethernet Buffer Error Status bit"]
    bufer @ 6,
    #[doc = "INT Interrupt Flag bit"]
    int @ 7,
});

register!(ECON2, 0b1000_0000, u8, {
    #[doc = "Voltage Regulator Power Save Enable bit"]
    vrps @ 3,
    #[doc = "Power Save Enable bit"]
    pwrsv @ 5,
    #[doc = "Packet Decrement bit"]
    pktdec @ 6,
    #[doc = "Automatic Buffer Pointer Increment Enable bit"]
    autoinc @ 7,
});

register!(ECON1, 0, u8, {
    #[doc = "Bank Select bits"]
    bsel @ 0..1,
    #[doc = "Receive Enable bi"]
    rxen @ 2,
    #[doc = "Transmit Request to Send bit"]
    txrts @ 3,
    #[doc = "DMA Checksum Enable bit"]
    csumen @ 4,
    #[doc = "DMA Start and Busy Status bit"]
    dmast @ 5,
    #[doc = "Receive Logic Reset bit"]
    rxrst @ 6,
    #[doc = "Transmit Logic Reset bit"]
    txrst @ 7,
});

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Register {
    MACON1 = 0x00,
    MACON3 = 0x02,
    MACON4 = 0x03,
    MABBIPG = 0x04,
    MAIPGL = 0x06,
    MAIPGH = 0x07,
    MACLCON1 = 0x08,
    MACLCON2 = 0x09,
    MAMXFLL = 0x0a,
    MAMXFLH = 0x0b,
    MICMD = 0x12,
    MIREGADR = 0x14,
    MIWRL = 0x16,
    MIWRH = 0x17,
    MIRDL = 0x18,
    MIRDH = 0x19,
}

impl Register {
    pub(crate) fn addr(&self) -> u8 {
        *self as u8
    }

    pub(crate) fn is_eth_register(&self) -> bool {
        match *self {
            Register::MACON1 => false,
            Register::MACON3 => false,
            Register::MACON4 => false,
            Register::MABBIPG => false,
            Register::MAIPGL => false,
            Register::MAIPGH => false,
            Register::MACLCON1 => false,
            Register::MACLCON2 => false,
            Register::MAMXFLL => false,
            Register::MAMXFLH => false,
            Register::MICMD => false,
            Register::MIREGADR => false,
            Register::MIWRL => false,
            Register::MIWRH => false,
            Register::MIRDL => false,
            Register::MIRDH => false,
        }
    }
}

impl Into<super::Register> for Register {
    fn into(self) -> super::Register {
        super::Register::Bank2(self)
    }
}

register!(MACON1, 0, u8, {
    #[doc = "Enable packets to be received by the MAC"]
    marxen @ 0,
    #[doc = "Control frames will be discarded after being processed by the MAC"]
    passall @ 1,
    #[doc = "Inhibit transmissions when pause control frames are received"]
    rxpaus @ 2,
    #[doc = "Allow the MAC to transmit pause control frames"]
    txpaus @ 3,
});

register!(MACON3, 0, u8, {
    #[doc = "MAC will operate in Full-Duplex mode"]
    fuldpx @ 0,
    #[doc = "The type/length field of transmitted and received frames will be checked"]
    frmlnen @ 1,
    #[doc = "Frames bigger than MAMXFL will be aborted when transmitted or received"]
    hfrmen @ 2,
    #[doc = "No proprietary header is present"]
    phdren @ 3,
    #[doc = "MAC will append a valid CRC to all frames transmitted regardless of PADCFG bit"]
    txcrcen @ 4,
    #[doc = "All short frames will be zero-padded to 64 bytes and a valid CRC will then be appended"]
    padcfg @ 5..7,
});

register!(MICMD, 0, u8, {
    #[doc = "MII Read Enable bit"]
    miird @ 0,
    #[doc = "MII Scan Enable bit"]
    miiscan @ 1,
});

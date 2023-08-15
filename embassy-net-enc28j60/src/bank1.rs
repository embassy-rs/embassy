#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Register {
    EHT0 = 0x00,
    EHT1 = 0x01,
    EHT2 = 0x02,
    EHT3 = 0x03,
    EHT4 = 0x04,
    EHT5 = 0x05,
    EHT6 = 0x06,
    EHT7 = 0x07,
    EPMM0 = 0x08,
    EPMM1 = 0x09,
    EPMM2 = 0x0a,
    EPMM3 = 0x0b,
    EPMM4 = 0x0c,
    EPMM5 = 0x0d,
    EPMM6 = 0x0e,
    EPMM7 = 0x0f,
    EPMCSL = 0x10,
    EPMCSH = 0x11,
    EPMOL = 0x14,
    EPMOH = 0x15,
    ERXFCON = 0x18,
    EPKTCNT = 0x19,
}

impl Register {
    pub(crate) fn addr(&self) -> u8 {
        *self as u8
    }

    pub(crate) fn is_eth_register(&self) -> bool {
        match *self {
            Register::EHT0 => true,
            Register::EHT1 => true,
            Register::EHT2 => true,
            Register::EHT3 => true,
            Register::EHT4 => true,
            Register::EHT5 => true,
            Register::EHT6 => true,
            Register::EHT7 => true,
            Register::EPMM0 => true,
            Register::EPMM1 => true,
            Register::EPMM2 => true,
            Register::EPMM3 => true,
            Register::EPMM4 => true,
            Register::EPMM5 => true,
            Register::EPMM6 => true,
            Register::EPMM7 => true,
            Register::EPMCSL => true,
            Register::EPMCSH => true,
            Register::EPMOL => true,
            Register::EPMOH => true,
            Register::ERXFCON => true,
            Register::EPKTCNT => true,
        }
    }
}

impl Into<super::Register> for Register {
    fn into(self) -> super::Register {
        super::Register::Bank1(self)
    }
}

register!(ERXFCON, 0b1010_0001, u8, {
    #[doc = "Broadcast Filter Enable bit"]
    bcen @ 0,
    #[doc = "Multicast Filter Enable bit"]
    mcen @ 1,
    #[doc = "Hash Table Filter Enable bit"]
    hten @ 2,
    #[doc = "Magic Packet™ Filter Enable bit"]
    mpen @ 3,
    #[doc = "Pattern Match Filter Enable bit"]
    pmen @ 4,
    #[doc = "Post-Filter CRC Check Enable bit"]
    crcen @ 5,
    #[doc = "AND/OR Filter Select bit"]
    andor @ 6,
    #[doc = "Unicast Filter Enable bit"]
    ucen @ 7,
});

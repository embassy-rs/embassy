#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Register {
    ERDPTL = 0x00,
    ERDPTH = 0x01,
    EWRPTL = 0x02,
    EWRPTH = 0x03,
    ETXSTL = 0x04,
    ETXSTH = 0x05,
    ETXNDL = 0x06,
    ETXNDH = 0x07,
    ERXSTL = 0x08,
    ERXSTH = 0x09,
    ERXNDL = 0x0a,
    ERXNDH = 0x0b,
    ERXRDPTL = 0x0c,
    ERXRDPTH = 0x0d,
    ERXWRPTL = 0x0e,
    ERXWRPTH = 0x0f,
    EDMASTL = 0x10,
    EDMASTH = 0x11,
    EDMANDL = 0x12,
    EDMANDH = 0x13,
    EDMADSTL = 0x14,
    EDMADSTH = 0x15,
    EDMACSL = 0x16,
    EDMACSH = 0x17,
}

impl Register {
    pub(crate) fn addr(&self) -> u8 {
        *self as u8
    }

    pub(crate) fn is_eth_register(&self) -> bool {
        match *self {
            Register::ERDPTL => true,
            Register::ERDPTH => true,
            Register::EWRPTL => true,
            Register::EWRPTH => true,
            Register::ETXSTL => true,
            Register::ETXSTH => true,
            Register::ETXNDL => true,
            Register::ETXNDH => true,
            Register::ERXSTL => true,
            Register::ERXSTH => true,
            Register::ERXNDL => true,
            Register::ERXNDH => true,
            Register::ERXRDPTL => true,
            Register::ERXRDPTH => true,
            Register::ERXWRPTL => true,
            Register::ERXWRPTH => true,
            Register::EDMASTL => true,
            Register::EDMASTH => true,
            Register::EDMANDL => true,
            Register::EDMANDH => true,
            Register::EDMADSTL => true,
            Register::EDMADSTH => true,
            Register::EDMACSL => true,
            Register::EDMACSH => true,
        }
    }
}

impl Into<super::Register> for Register {
    fn into(self) -> super::Register {
        super::Register::Bank0(self)
    }
}

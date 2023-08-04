#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Register {
    MAADR5 = 0x00,
    MAADR6 = 0x01,
    MAADR3 = 0x02,
    MAADR4 = 0x03,
    MAADR1 = 0x04,
    MAADR2 = 0x05,
    EBSTSD = 0x06,
    EBSTCON = 0x07,
    EBSTCSL = 0x08,
    EBSTCSH = 0x09,
    MISTAT = 0x0a,
    EREVID = 0x12,
    ECOCON = 0x15,
    EFLOCON = 0x17,
    EPAUSL = 0x18,
    EPAUSH = 0x19,
}

impl Register {
    pub(crate) fn addr(&self) -> u8 {
        *self as u8
    }

    pub(crate) fn is_eth_register(&self) -> bool {
        match *self {
            Register::MAADR5 => false,
            Register::MAADR6 => false,
            Register::MAADR3 => false,
            Register::MAADR4 => false,
            Register::MAADR1 => false,
            Register::MAADR2 => false,
            Register::EBSTSD => true,
            Register::EBSTCON => true,
            Register::EBSTCSL => true,
            Register::EBSTCSH => true,
            Register::MISTAT => false,
            Register::EREVID => true,
            Register::ECOCON => true,
            Register::EFLOCON => true,
            Register::EPAUSL => true,
            Register::EPAUSH => true,
        }
    }
}

impl Into<super::Register> for Register {
    fn into(self) -> super::Register {
        super::Register::Bank3(self)
    }
}

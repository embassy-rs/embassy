#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Register {
    PHCON1 = 0x00,
    PHSTAT1 = 0x01,
    PHID1 = 0x02,
    PHID2 = 0x03,
    PHCON2 = 0x10,
    PHSTAT2 = 0x11,
    PHIE = 0x12,
    PHIR = 0x13,
    PHLCON = 0x14,
}

impl Register {
    pub(crate) fn addr(&self) -> u8 {
        *self as u8
    }
}

register!(PHCON2, 0, u16, {
    #[doc = "PHY Half-Duplex Loopback Disable bit"]
    hdldis @ 8,
    #[doc = "Jabber Correction Disable bit"]
    jabber @ 10,
    #[doc = "Twisted-Pair Transmitter Disable bit"]
    txdis @ 13,
    #[doc = "PHY Force Linkup bit"]
    frclnk @ 14,
});

register!(PHSTAT2, 0, u16, {
    #[doc = "Link Status bit"]
    lstat @ 10,
});

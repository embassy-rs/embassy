#[doc = "Register `SCAPABILITIES2` reader"]
pub type R = crate::R<Scapabilities2Spec>;
#[doc = "Field `MAPCNT` reader - Map Count"]
pub type MapcntR = crate::FieldReader;
#[doc = "I2C 10-bit Address\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum I2c10b {
    #[doc = "0: Not supported"]
    Disable = 0,
    #[doc = "1: Supported"]
    Enable = 1,
}
impl From<I2c10b> for bool {
    #[inline(always)]
    fn from(variant: I2c10b) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `I2C10B` reader - I2C 10-bit Address"]
pub type I2c10bR = crate::BitReader<I2c10b>;
impl I2c10bR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> I2c10b {
        match self.bits {
            false => I2c10b::Disable,
            true => I2c10b::Enable,
        }
    }
    #[doc = "Not supported"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == I2c10b::Disable
    }
    #[doc = "Supported"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == I2c10b::Enable
    }
}
#[doc = "I2C Device ID\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum I2cdevid {
    #[doc = "0: Not supported"]
    Disable = 0,
    #[doc = "1: Supported"]
    Enable = 1,
}
impl From<I2cdevid> for bool {
    #[inline(always)]
    fn from(variant: I2cdevid) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `I2CDEVID` reader - I2C Device ID"]
pub type I2cdevidR = crate::BitReader<I2cdevid>;
impl I2cdevidR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> I2cdevid {
        match self.bits {
            false => I2cdevid::Disable,
            true => I2cdevid::Enable,
        }
    }
    #[doc = "Not supported"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == I2cdevid::Disable
    }
    #[doc = "Supported"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == I2cdevid::Enable
    }
}
#[doc = "In-Band Interrupt EXTDATA\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ibiext {
    #[doc = "0: Not supported"]
    Disable = 0,
    #[doc = "1: Supported"]
    Enable = 1,
}
impl From<Ibiext> for bool {
    #[inline(always)]
    fn from(variant: Ibiext) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IBIEXT` reader - In-Band Interrupt EXTDATA"]
pub type IbiextR = crate::BitReader<Ibiext>;
impl IbiextR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ibiext {
        match self.bits {
            false => Ibiext::Disable,
            true => Ibiext::Enable,
        }
    }
    #[doc = "Not supported"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Ibiext::Disable
    }
    #[doc = "Supported"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Ibiext::Enable
    }
}
#[doc = "In-Band Interrupt Extended Register\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ibixreg {
    #[doc = "0: Not supported"]
    Disable = 0,
    #[doc = "1: Supported"]
    Enable = 1,
}
impl From<Ibixreg> for bool {
    #[inline(always)]
    fn from(variant: Ibixreg) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IBIXREG` reader - In-Band Interrupt Extended Register"]
pub type IbixregR = crate::BitReader<Ibixreg>;
impl IbixregR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ibixreg {
        match self.bits {
            false => Ibixreg::Disable,
            true => Ibixreg::Enable,
        }
    }
    #[doc = "Not supported"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Ibixreg::Disable
    }
    #[doc = "Supported"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Ibixreg::Enable
    }
}
#[doc = "Target Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slvrst {
    #[doc = "0: Not supported"]
    Disable = 0,
    #[doc = "1: Supported"]
    Enable = 1,
}
impl From<Slvrst> for bool {
    #[inline(always)]
    fn from(variant: Slvrst) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SLVRST` reader - Target Reset"]
pub type SlvrstR = crate::BitReader<Slvrst>;
impl SlvrstR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Slvrst {
        match self.bits {
            false => Slvrst::Disable,
            true => Slvrst::Enable,
        }
    }
    #[doc = "Not supported"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Slvrst::Disable
    }
    #[doc = "Supported"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Slvrst::Enable
    }
}
#[doc = "Group\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Group {
    #[doc = "0: v1.1 group addressing not supported"]
    Notsupported = 0,
    #[doc = "1: One group supported"]
    One = 1,
    #[doc = "2: Two groups supported"]
    Two = 2,
    #[doc = "3: Three groups supported"]
    Three = 3,
}
impl From<Group> for u8 {
    #[inline(always)]
    fn from(variant: Group) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Group {
    type Ux = u8;
}
impl crate::IsEnum for Group {}
#[doc = "Field `GROUP` reader - Group"]
pub type GroupR = crate::FieldReader<Group>;
impl GroupR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Group {
        match self.bits {
            0 => Group::Notsupported,
            1 => Group::One,
            2 => Group::Two,
            3 => Group::Three,
            _ => unreachable!(),
        }
    }
    #[doc = "v1.1 group addressing not supported"]
    #[inline(always)]
    pub fn is_notsupported(&self) -> bool {
        *self == Group::Notsupported
    }
    #[doc = "One group supported"]
    #[inline(always)]
    pub fn is_one(&self) -> bool {
        *self == Group::One
    }
    #[doc = "Two groups supported"]
    #[inline(always)]
    pub fn is_two(&self) -> bool {
        *self == Group::Two
    }
    #[doc = "Three groups supported"]
    #[inline(always)]
    pub fn is_three(&self) -> bool {
        *self == Group::Three
    }
}
#[doc = "SETAASA\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aasa {
    #[doc = "0: SETAASA not supported"]
    Notsupported = 0,
    #[doc = "1: SETAASA supported"]
    Supported = 1,
}
impl From<Aasa> for bool {
    #[inline(always)]
    fn from(variant: Aasa) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AASA` reader - SETAASA"]
pub type AasaR = crate::BitReader<Aasa>;
impl AasaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Aasa {
        match self.bits {
            false => Aasa::Notsupported,
            true => Aasa::Supported,
        }
    }
    #[doc = "SETAASA not supported"]
    #[inline(always)]
    pub fn is_notsupported(&self) -> bool {
        *self == Aasa::Notsupported
    }
    #[doc = "SETAASA supported"]
    #[inline(always)]
    pub fn is_supported(&self) -> bool {
        *self == Aasa::Supported
    }
}
#[doc = "Target-Target(s)-Tunnel Subscriber Capable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sstsub {
    #[doc = "0: Not subscriber capable"]
    Notsupported = 0,
    #[doc = "1: Subscriber capable"]
    Supported = 1,
}
impl From<Sstsub> for bool {
    #[inline(always)]
    fn from(variant: Sstsub) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SSTSUB` reader - Target-Target(s)-Tunnel Subscriber Capable"]
pub type SstsubR = crate::BitReader<Sstsub>;
impl SstsubR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sstsub {
        match self.bits {
            false => Sstsub::Notsupported,
            true => Sstsub::Supported,
        }
    }
    #[doc = "Not subscriber capable"]
    #[inline(always)]
    pub fn is_notsupported(&self) -> bool {
        *self == Sstsub::Notsupported
    }
    #[doc = "Subscriber capable"]
    #[inline(always)]
    pub fn is_supported(&self) -> bool {
        *self == Sstsub::Supported
    }
}
#[doc = "Target-Target(s)-Tunnel Write Capable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sstwr {
    #[doc = "0: Not write capable"]
    Notsupported = 0,
    #[doc = "1: Write capable"]
    Supported = 1,
}
impl From<Sstwr> for bool {
    #[inline(always)]
    fn from(variant: Sstwr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SSTWR` reader - Target-Target(s)-Tunnel Write Capable"]
pub type SstwrR = crate::BitReader<Sstwr>;
impl SstwrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sstwr {
        match self.bits {
            false => Sstwr::Notsupported,
            true => Sstwr::Supported,
        }
    }
    #[doc = "Not write capable"]
    #[inline(always)]
    pub fn is_notsupported(&self) -> bool {
        *self == Sstwr::Notsupported
    }
    #[doc = "Write capable"]
    #[inline(always)]
    pub fn is_supported(&self) -> bool {
        *self == Sstwr::Supported
    }
}
impl R {
    #[doc = "Bits 0:3 - Map Count"]
    #[inline(always)]
    pub fn mapcnt(&self) -> MapcntR {
        MapcntR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bit 4 - I2C 10-bit Address"]
    #[inline(always)]
    pub fn i2c10b(&self) -> I2c10bR {
        I2c10bR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 6 - I2C Device ID"]
    #[inline(always)]
    pub fn i2cdevid(&self) -> I2cdevidR {
        I2cdevidR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 8 - In-Band Interrupt EXTDATA"]
    #[inline(always)]
    pub fn ibiext(&self) -> IbiextR {
        IbiextR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - In-Band Interrupt Extended Register"]
    #[inline(always)]
    pub fn ibixreg(&self) -> IbixregR {
        IbixregR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 17 - Target Reset"]
    #[inline(always)]
    pub fn slvrst(&self) -> SlvrstR {
        SlvrstR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bits 18:19 - Group"]
    #[inline(always)]
    pub fn group(&self) -> GroupR {
        GroupR::new(((self.bits >> 18) & 3) as u8)
    }
    #[doc = "Bit 21 - SETAASA"]
    #[inline(always)]
    pub fn aasa(&self) -> AasaR {
        AasaR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Target-Target(s)-Tunnel Subscriber Capable"]
    #[inline(always)]
    pub fn sstsub(&self) -> SstsubR {
        SstsubR::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Target-Target(s)-Tunnel Write Capable"]
    #[inline(always)]
    pub fn sstwr(&self) -> SstwrR {
        SstwrR::new(((self.bits >> 23) & 1) != 0)
    }
}
#[doc = "Target Capabilities 2\n\nYou can [`read`](crate::Reg::read) this register and get [`scapabilities2::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Scapabilities2Spec;
impl crate::RegisterSpec for Scapabilities2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`scapabilities2::R`](R) reader structure"]
impl crate::Readable for Scapabilities2Spec {}
#[doc = "`reset()` method sets SCAPABILITIES2 to value 0x0300"]
impl crate::Resettable for Scapabilities2Spec {
    const RESET_VALUE: u32 = 0x0300;
}

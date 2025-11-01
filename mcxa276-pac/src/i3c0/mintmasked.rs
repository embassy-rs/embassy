#[doc = "Register `MINTMASKED` reader"]
pub type R = crate::R<MintmaskedSpec>;
#[doc = "SLVSTART Interrupt Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slvstart {
    #[doc = "0: Disabled"]
    NotEnabled = 0,
    #[doc = "1: Enabled"]
    Enabled = 1,
}
impl From<Slvstart> for bool {
    #[inline(always)]
    fn from(variant: Slvstart) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SLVSTART` reader - SLVSTART Interrupt Mask"]
pub type SlvstartR = crate::BitReader<Slvstart>;
impl SlvstartR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Slvstart {
        match self.bits {
            false => Slvstart::NotEnabled,
            true => Slvstart::Enabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_not_enabled(&self) -> bool {
        *self == Slvstart::NotEnabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Slvstart::Enabled
    }
}
#[doc = "MCTRLDONE Interrupt Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mctrldone {
    #[doc = "0: Disabled"]
    NotEnabled = 0,
    #[doc = "1: Enabled"]
    Enabled = 1,
}
impl From<Mctrldone> for bool {
    #[inline(always)]
    fn from(variant: Mctrldone) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MCTRLDONE` reader - MCTRLDONE Interrupt Mask"]
pub type MctrldoneR = crate::BitReader<Mctrldone>;
impl MctrldoneR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mctrldone {
        match self.bits {
            false => Mctrldone::NotEnabled,
            true => Mctrldone::Enabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_not_enabled(&self) -> bool {
        *self == Mctrldone::NotEnabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Mctrldone::Enabled
    }
}
#[doc = "COMPLETE Interrupt Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Complete {
    #[doc = "0: Disabled"]
    NotEnabled = 0,
    #[doc = "1: Enabled"]
    Enabled = 1,
}
impl From<Complete> for bool {
    #[inline(always)]
    fn from(variant: Complete) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `COMPLETE` reader - COMPLETE Interrupt Mask"]
pub type CompleteR = crate::BitReader<Complete>;
impl CompleteR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Complete {
        match self.bits {
            false => Complete::NotEnabled,
            true => Complete::Enabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_not_enabled(&self) -> bool {
        *self == Complete::NotEnabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Complete::Enabled
    }
}
#[doc = "Field `RXPEND` reader - RXPEND Interrupt Mask"]
pub type RxpendR = crate::BitReader;
#[doc = "TXNOTFULL Interrupt Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Txnotfull {
    #[doc = "0: Disabled"]
    NotEnabled = 0,
    #[doc = "1: Enabled"]
    Enabled = 1,
}
impl From<Txnotfull> for bool {
    #[inline(always)]
    fn from(variant: Txnotfull) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TXNOTFULL` reader - TXNOTFULL Interrupt Mask"]
pub type TxnotfullR = crate::BitReader<Txnotfull>;
impl TxnotfullR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Txnotfull {
        match self.bits {
            false => Txnotfull::NotEnabled,
            true => Txnotfull::Enabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_not_enabled(&self) -> bool {
        *self == Txnotfull::NotEnabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Txnotfull::Enabled
    }
}
#[doc = "IBIWON Interrupt Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ibiwon {
    #[doc = "0: Disabled"]
    NotEnabled = 0,
    #[doc = "1: Enabled"]
    Enabled = 1,
}
impl From<Ibiwon> for bool {
    #[inline(always)]
    fn from(variant: Ibiwon) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IBIWON` reader - IBIWON Interrupt Mask"]
pub type IbiwonR = crate::BitReader<Ibiwon>;
impl IbiwonR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ibiwon {
        match self.bits {
            false => Ibiwon::NotEnabled,
            true => Ibiwon::Enabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_not_enabled(&self) -> bool {
        *self == Ibiwon::NotEnabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ibiwon::Enabled
    }
}
#[doc = "ERRWARN Interrupt Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Errwarn {
    #[doc = "0: Disabled"]
    NotEnabled = 0,
    #[doc = "1: Enabled"]
    Enabled = 1,
}
impl From<Errwarn> for bool {
    #[inline(always)]
    fn from(variant: Errwarn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERRWARN` reader - ERRWARN Interrupt Mask"]
pub type ErrwarnR = crate::BitReader<Errwarn>;
impl ErrwarnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Errwarn {
        match self.bits {
            false => Errwarn::NotEnabled,
            true => Errwarn::Enabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_not_enabled(&self) -> bool {
        *self == Errwarn::NotEnabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Errwarn::Enabled
    }
}
#[doc = "NOWCONTROLLER Interrupt Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nowmaster {
    #[doc = "0: Disabled"]
    NotEnabled = 0,
    #[doc = "1: Enabled"]
    Enabled = 1,
}
impl From<Nowmaster> for bool {
    #[inline(always)]
    fn from(variant: Nowmaster) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NOWMASTER` reader - NOWCONTROLLER Interrupt Mask"]
pub type NowmasterR = crate::BitReader<Nowmaster>;
impl NowmasterR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nowmaster {
        match self.bits {
            false => Nowmaster::NotEnabled,
            true => Nowmaster::Enabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_not_enabled(&self) -> bool {
        *self == Nowmaster::NotEnabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Nowmaster::Enabled
    }
}
impl R {
    #[doc = "Bit 8 - SLVSTART Interrupt Mask"]
    #[inline(always)]
    pub fn slvstart(&self) -> SlvstartR {
        SlvstartR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - MCTRLDONE Interrupt Mask"]
    #[inline(always)]
    pub fn mctrldone(&self) -> MctrldoneR {
        MctrldoneR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - COMPLETE Interrupt Mask"]
    #[inline(always)]
    pub fn complete(&self) -> CompleteR {
        CompleteR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - RXPEND Interrupt Mask"]
    #[inline(always)]
    pub fn rxpend(&self) -> RxpendR {
        RxpendR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - TXNOTFULL Interrupt Mask"]
    #[inline(always)]
    pub fn txnotfull(&self) -> TxnotfullR {
        TxnotfullR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - IBIWON Interrupt Mask"]
    #[inline(always)]
    pub fn ibiwon(&self) -> IbiwonR {
        IbiwonR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 15 - ERRWARN Interrupt Mask"]
    #[inline(always)]
    pub fn errwarn(&self) -> ErrwarnR {
        ErrwarnR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 19 - NOWCONTROLLER Interrupt Mask"]
    #[inline(always)]
    pub fn nowmaster(&self) -> NowmasterR {
        NowmasterR::new(((self.bits >> 19) & 1) != 0)
    }
}
#[doc = "Controller Interrupt Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`mintmasked::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MintmaskedSpec;
impl crate::RegisterSpec for MintmaskedSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mintmasked::R`](R) reader structure"]
impl crate::Readable for MintmaskedSpec {}
#[doc = "`reset()` method sets MINTMASKED to value 0"]
impl crate::Resettable for MintmaskedSpec {}

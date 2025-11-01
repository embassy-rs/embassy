#[doc = "Register `PARAM` reader"]
pub type R = crate::R<ParamSpec>;
#[doc = "SOSC Clock Present\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Soscclkpres {
    #[doc = "0: SOSC clock source is not present"]
    Nopres = 0,
    #[doc = "1: SOSC clock source is present"]
    Pres = 1,
}
impl From<Soscclkpres> for bool {
    #[inline(always)]
    fn from(variant: Soscclkpres) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOSCCLKPRES` reader - SOSC Clock Present"]
pub type SoscclkpresR = crate::BitReader<Soscclkpres>;
impl SoscclkpresR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Soscclkpres {
        match self.bits {
            false => Soscclkpres::Nopres,
            true => Soscclkpres::Pres,
        }
    }
    #[doc = "SOSC clock source is not present"]
    #[inline(always)]
    pub fn is_nopres(&self) -> bool {
        *self == Soscclkpres::Nopres
    }
    #[doc = "SOSC clock source is present"]
    #[inline(always)]
    pub fn is_pres(&self) -> bool {
        *self == Soscclkpres::Pres
    }
}
#[doc = "SIRC Clock Present\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sircclkpres {
    #[doc = "0: SIRC clock source is not present"]
    Nopres = 0,
    #[doc = "1: SIRC clock source is present"]
    Pres = 1,
}
impl From<Sircclkpres> for bool {
    #[inline(always)]
    fn from(variant: Sircclkpres) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SIRCCLKPRES` reader - SIRC Clock Present"]
pub type SircclkpresR = crate::BitReader<Sircclkpres>;
impl SircclkpresR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sircclkpres {
        match self.bits {
            false => Sircclkpres::Nopres,
            true => Sircclkpres::Pres,
        }
    }
    #[doc = "SIRC clock source is not present"]
    #[inline(always)]
    pub fn is_nopres(&self) -> bool {
        *self == Sircclkpres::Nopres
    }
    #[doc = "SIRC clock source is present"]
    #[inline(always)]
    pub fn is_pres(&self) -> bool {
        *self == Sircclkpres::Pres
    }
}
#[doc = "FIRC Clock Present\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fircclkpres {
    #[doc = "0: FIRC clock source is not present"]
    Nopres = 0,
    #[doc = "1: FIRC clock source is present"]
    Pres = 1,
}
impl From<Fircclkpres> for bool {
    #[inline(always)]
    fn from(variant: Fircclkpres) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FIRCCLKPRES` reader - FIRC Clock Present"]
pub type FircclkpresR = crate::BitReader<Fircclkpres>;
impl FircclkpresR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fircclkpres {
        match self.bits {
            false => Fircclkpres::Nopres,
            true => Fircclkpres::Pres,
        }
    }
    #[doc = "FIRC clock source is not present"]
    #[inline(always)]
    pub fn is_nopres(&self) -> bool {
        *self == Fircclkpres::Nopres
    }
    #[doc = "FIRC clock source is present"]
    #[inline(always)]
    pub fn is_pres(&self) -> bool {
        *self == Fircclkpres::Pres
    }
}
#[doc = "ROSC Clock Present\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Roscclkpres {
    #[doc = "0: ROSC clock source is not present"]
    Nopres = 0,
    #[doc = "1: ROSC clock source is present"]
    Pres = 1,
}
impl From<Roscclkpres> for bool {
    #[inline(always)]
    fn from(variant: Roscclkpres) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ROSCCLKPRES` reader - ROSC Clock Present"]
pub type RoscclkpresR = crate::BitReader<Roscclkpres>;
impl RoscclkpresR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Roscclkpres {
        match self.bits {
            false => Roscclkpres::Nopres,
            true => Roscclkpres::Pres,
        }
    }
    #[doc = "ROSC clock source is not present"]
    #[inline(always)]
    pub fn is_nopres(&self) -> bool {
        *self == Roscclkpres::Nopres
    }
    #[doc = "ROSC clock source is present"]
    #[inline(always)]
    pub fn is_pres(&self) -> bool {
        *self == Roscclkpres::Pres
    }
}
#[doc = "SPLL Clock Present\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spllclkpres {
    #[doc = "0: SPLL clock source is not present"]
    Nopres = 0,
    #[doc = "1: SPLL clock source is present"]
    Pres = 1,
}
impl From<Spllclkpres> for bool {
    #[inline(always)]
    fn from(variant: Spllclkpres) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPLLCLKPRES` reader - SPLL Clock Present"]
pub type SpllclkpresR = crate::BitReader<Spllclkpres>;
impl SpllclkpresR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Spllclkpres {
        match self.bits {
            false => Spllclkpres::Nopres,
            true => Spllclkpres::Pres,
        }
    }
    #[doc = "SPLL clock source is not present"]
    #[inline(always)]
    pub fn is_nopres(&self) -> bool {
        *self == Spllclkpres::Nopres
    }
    #[doc = "SPLL clock source is present"]
    #[inline(always)]
    pub fn is_pres(&self) -> bool {
        *self == Spllclkpres::Pres
    }
}
impl R {
    #[doc = "Bit 1 - SOSC Clock Present"]
    #[inline(always)]
    pub fn soscclkpres(&self) -> SoscclkpresR {
        SoscclkpresR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - SIRC Clock Present"]
    #[inline(always)]
    pub fn sircclkpres(&self) -> SircclkpresR {
        SircclkpresR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - FIRC Clock Present"]
    #[inline(always)]
    pub fn fircclkpres(&self) -> FircclkpresR {
        FircclkpresR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - ROSC Clock Present"]
    #[inline(always)]
    pub fn roscclkpres(&self) -> RoscclkpresR {
        RoscclkpresR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 6 - SPLL Clock Present"]
    #[inline(always)]
    pub fn spllclkpres(&self) -> SpllclkpresR {
        SpllclkpresR::new(((self.bits >> 6) & 1) != 0)
    }
}
#[doc = "Parameter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`param::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ParamSpec;
impl crate::RegisterSpec for ParamSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`param::R`](R) reader structure"]
impl crate::Readable for ParamSpec {}
#[doc = "`reset()` method sets PARAM to value 0x5e"]
impl crate::Resettable for ParamSpec {
    const RESET_VALUE: u32 = 0x5e;
}

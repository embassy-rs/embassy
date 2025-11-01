#[doc = "Register `SMSGMAPADDR` reader"]
pub type R = crate::R<SmsgmapaddrSpec>;
#[doc = "Field `MAPLAST` reader - Matched Address Index"]
pub type MaplastR = crate::FieldReader;
#[doc = "Last Static Address Matched\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Laststatic {
    #[doc = "0: I3C dynamic address"]
    I3c = 0,
    #[doc = "1: I2C static address"]
    I2c = 1,
}
impl From<Laststatic> for bool {
    #[inline(always)]
    fn from(variant: Laststatic) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LASTSTATIC` reader - Last Static Address Matched"]
pub type LaststaticR = crate::BitReader<Laststatic>;
impl LaststaticR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Laststatic {
        match self.bits {
            false => Laststatic::I3c,
            true => Laststatic::I2c,
        }
    }
    #[doc = "I3C dynamic address"]
    #[inline(always)]
    pub fn is_i3c(&self) -> bool {
        *self == Laststatic::I3c
    }
    #[doc = "I2C static address"]
    #[inline(always)]
    pub fn is_i2c(&self) -> bool {
        *self == Laststatic::I2c
    }
}
#[doc = "Field `MAPLASTM1` reader - Matched Previous Address Index 1"]
pub type Maplastm1R = crate::FieldReader;
#[doc = "Field `MAPLASTM2` reader - Matched Previous Index 2"]
pub type Maplastm2R = crate::FieldReader;
impl R {
    #[doc = "Bits 0:3 - Matched Address Index"]
    #[inline(always)]
    pub fn maplast(&self) -> MaplastR {
        MaplastR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bit 4 - Last Static Address Matched"]
    #[inline(always)]
    pub fn laststatic(&self) -> LaststaticR {
        LaststaticR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bits 8:11 - Matched Previous Address Index 1"]
    #[inline(always)]
    pub fn maplastm1(&self) -> Maplastm1R {
        Maplastm1R::new(((self.bits >> 8) & 0x0f) as u8)
    }
    #[doc = "Bits 16:19 - Matched Previous Index 2"]
    #[inline(always)]
    pub fn maplastm2(&self) -> Maplastm2R {
        Maplastm2R::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
#[doc = "Target Message Map Address\n\nYou can [`read`](crate::Reg::read) this register and get [`smsgmapaddr::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SmsgmapaddrSpec;
impl crate::RegisterSpec for SmsgmapaddrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`smsgmapaddr::R`](R) reader structure"]
impl crate::Readable for SmsgmapaddrSpec {}
#[doc = "`reset()` method sets SMSGMAPADDR to value 0"]
impl crate::Resettable for SmsgmapaddrSpec {}

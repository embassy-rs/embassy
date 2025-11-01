#[doc = "Register `OTGSTAT` reader"]
pub type R = crate::R<OtgstatSpec>;
#[doc = "Line State Stable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Linestatestable {
    #[doc = "0: Unstable"]
    LinestNotStable = 0,
    #[doc = "1: Stable"]
    LinestStable = 1,
}
impl From<Linestatestable> for bool {
    #[inline(always)]
    fn from(variant: Linestatestable) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LINESTATESTABLE` reader - Line State Stable"]
pub type LinestatestableR = crate::BitReader<Linestatestable>;
impl LinestatestableR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Linestatestable {
        match self.bits {
            false => Linestatestable::LinestNotStable,
            true => Linestatestable::LinestStable,
        }
    }
    #[doc = "Unstable"]
    #[inline(always)]
    pub fn is_linest_not_stable(&self) -> bool {
        *self == Linestatestable::LinestNotStable
    }
    #[doc = "Stable"]
    #[inline(always)]
    pub fn is_linest_stable(&self) -> bool {
        *self == Linestatestable::LinestStable
    }
}
#[doc = "Field `ONEMSEC` reader - Reserved for 1 ms count"]
pub type OnemsecR = crate::BitReader;
impl R {
    #[doc = "Bit 5 - Line State Stable"]
    #[inline(always)]
    pub fn linestatestable(&self) -> LinestatestableR {
        LinestatestableR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Reserved for 1 ms count"]
    #[inline(always)]
    pub fn onemsec(&self) -> OnemsecR {
        OnemsecR::new(((self.bits >> 6) & 1) != 0)
    }
}
#[doc = "OTG Status\n\nYou can [`read`](crate::Reg::read) this register and get [`otgstat::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct OtgstatSpec;
impl crate::RegisterSpec for OtgstatSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`otgstat::R`](R) reader structure"]
impl crate::Readable for OtgstatSpec {}
#[doc = "`reset()` method sets OTGSTAT to value 0"]
impl crate::Resettable for OtgstatSpec {}

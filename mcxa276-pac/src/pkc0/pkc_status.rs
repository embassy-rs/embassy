#[doc = "Register `PKC_STATUS` reader"]
pub type R = crate::R<PkcStatusSpec>;
#[doc = "Field `ACTIV` reader - PKC ACTIV"]
pub type ActivR = crate::BitReader;
#[doc = "Field `CARRY` reader - Carry overflow flag"]
pub type CarryR = crate::BitReader;
#[doc = "Field `ZERO` reader - Zero result flag"]
pub type ZeroR = crate::BitReader;
#[doc = "Field `GOANY` reader - Combined GO status flag"]
pub type GoanyR = crate::BitReader;
#[doc = "Field `LOCKED` reader - Parameter set locked"]
pub type LockedR = crate::FieldReader;
impl R {
    #[doc = "Bit 0 - PKC ACTIV"]
    #[inline(always)]
    pub fn activ(&self) -> ActivR {
        ActivR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Carry overflow flag"]
    #[inline(always)]
    pub fn carry(&self) -> CarryR {
        CarryR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Zero result flag"]
    #[inline(always)]
    pub fn zero(&self) -> ZeroR {
        ZeroR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Combined GO status flag"]
    #[inline(always)]
    pub fn goany(&self) -> GoanyR {
        GoanyR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bits 5:6 - Parameter set locked"]
    #[inline(always)]
    pub fn locked(&self) -> LockedR {
        LockedR::new(((self.bits >> 5) & 3) as u8)
    }
}
#[doc = "Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_status::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcStatusSpec;
impl crate::RegisterSpec for PkcStatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_status::R`](R) reader structure"]
impl crate::Readable for PkcStatusSpec {}
#[doc = "`reset()` method sets PKC_STATUS to value 0"]
impl crate::Resettable for PkcStatusSpec {}

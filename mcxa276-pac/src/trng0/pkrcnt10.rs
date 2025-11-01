#[doc = "Register `PKRCNT10` reader"]
pub type R = crate::R<Pkrcnt10Spec>;
#[doc = "Field `PKR_0_CT` reader - Poker 0h Count"]
pub type Pkr0CtR = crate::FieldReader<u16>;
#[doc = "Field `PKR_1_CT` reader - Poker 1h Count"]
pub type Pkr1CtR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Poker 0h Count"]
    #[inline(always)]
    pub fn pkr_0_ct(&self) -> Pkr0CtR {
        Pkr0CtR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - Poker 1h Count"]
    #[inline(always)]
    pub fn pkr_1_ct(&self) -> Pkr1CtR {
        Pkr1CtR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
#[doc = "Statistical Check Poker Count 1 and 0 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrcnt10::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pkrcnt10Spec;
impl crate::RegisterSpec for Pkrcnt10Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkrcnt10::R`](R) reader structure"]
impl crate::Readable for Pkrcnt10Spec {}
#[doc = "`reset()` method sets PKRCNT10 to value 0"]
impl crate::Resettable for Pkrcnt10Spec {}

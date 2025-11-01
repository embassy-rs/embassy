#[doc = "Register `PKRCNT32` reader"]
pub type R = crate::R<Pkrcnt32Spec>;
#[doc = "Field `PKR_2_CT` reader - Poker 2h Count"]
pub type Pkr2CtR = crate::FieldReader<u16>;
#[doc = "Field `PKR_3_CT` reader - Poker 3h Count"]
pub type Pkr3CtR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Poker 2h Count"]
    #[inline(always)]
    pub fn pkr_2_ct(&self) -> Pkr2CtR {
        Pkr2CtR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - Poker 3h Count"]
    #[inline(always)]
    pub fn pkr_3_ct(&self) -> Pkr3CtR {
        Pkr3CtR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
#[doc = "Statistical Check Poker Count 3 and 2 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrcnt32::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pkrcnt32Spec;
impl crate::RegisterSpec for Pkrcnt32Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkrcnt32::R`](R) reader structure"]
impl crate::Readable for Pkrcnt32Spec {}
#[doc = "`reset()` method sets PKRCNT32 to value 0"]
impl crate::Resettable for Pkrcnt32Spec {}

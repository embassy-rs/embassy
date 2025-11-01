#[doc = "Register `PKRCNT76` reader"]
pub type R = crate::R<Pkrcnt76Spec>;
#[doc = "Field `PKR_6_CT` reader - Poker 6h Count"]
pub type Pkr6CtR = crate::FieldReader<u16>;
#[doc = "Field `PKR_7_CT` reader - Poker 7h Count"]
pub type Pkr7CtR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Poker 6h Count"]
    #[inline(always)]
    pub fn pkr_6_ct(&self) -> Pkr6CtR {
        Pkr6CtR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - Poker 7h Count"]
    #[inline(always)]
    pub fn pkr_7_ct(&self) -> Pkr7CtR {
        Pkr7CtR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
#[doc = "Statistical Check Poker Count 7 and 6 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrcnt76::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pkrcnt76Spec;
impl crate::RegisterSpec for Pkrcnt76Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkrcnt76::R`](R) reader structure"]
impl crate::Readable for Pkrcnt76Spec {}
#[doc = "`reset()` method sets PKRCNT76 to value 0"]
impl crate::Resettable for Pkrcnt76Spec {}

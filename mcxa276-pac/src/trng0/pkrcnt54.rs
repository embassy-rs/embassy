#[doc = "Register `PKRCNT54` reader"]
pub type R = crate::R<Pkrcnt54Spec>;
#[doc = "Field `PKR_4_CT` reader - Poker 4h Count"]
pub type Pkr4CtR = crate::FieldReader<u16>;
#[doc = "Field `PKR_5_CT` reader - Poker 5h Count"]
pub type Pkr5CtR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Poker 4h Count"]
    #[inline(always)]
    pub fn pkr_4_ct(&self) -> Pkr4CtR {
        Pkr4CtR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - Poker 5h Count"]
    #[inline(always)]
    pub fn pkr_5_ct(&self) -> Pkr5CtR {
        Pkr5CtR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
#[doc = "Statistical Check Poker Count 5 and 4 Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrcnt54::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pkrcnt54Spec;
impl crate::RegisterSpec for Pkrcnt54Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkrcnt54::R`](R) reader structure"]
impl crate::Readable for Pkrcnt54Spec {}
#[doc = "`reset()` method sets PKRCNT54 to value 0"]
impl crate::Resettable for Pkrcnt54Spec {}

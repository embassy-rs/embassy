#[doc = "Register `UPOSH3` reader"]
pub type R = crate::R<Uposh3Uposh3Spec>;
#[doc = "Field `UPOSH3` reader - UPOSH3"]
pub type Uposh3R = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - UPOSH3"]
    #[inline(always)]
    pub fn uposh3(&self) -> Uposh3R {
        Uposh3R::new(self.bits)
    }
}
#[doc = "Upper Position Holder Register 3\n\nYou can [`read`](crate::Reg::read) this register and get [`uposh3_uposh3::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Uposh3Uposh3Spec;
impl crate::RegisterSpec for Uposh3Uposh3Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`uposh3_uposh3::R`](R) reader structure"]
impl crate::Readable for Uposh3Uposh3Spec {}
#[doc = "`reset()` method sets UPOSH3 to value 0"]
impl crate::Resettable for Uposh3Uposh3Spec {}

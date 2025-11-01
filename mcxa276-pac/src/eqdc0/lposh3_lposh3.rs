#[doc = "Register `LPOSH3` reader"]
pub type R = crate::R<Lposh3Lposh3Spec>;
#[doc = "Field `LPOSH3` reader - LPOSH3"]
pub type Lposh3R = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - LPOSH3"]
    #[inline(always)]
    pub fn lposh3(&self) -> Lposh3R {
        Lposh3R::new(self.bits)
    }
}
#[doc = "Lower Position Holder Register 3\n\nYou can [`read`](crate::Reg::read) this register and get [`lposh3_lposh3::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Lposh3Lposh3Spec;
impl crate::RegisterSpec for Lposh3Lposh3Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`lposh3_lposh3::R`](R) reader structure"]
impl crate::Readable for Lposh3Lposh3Spec {}
#[doc = "`reset()` method sets LPOSH3 to value 0"]
impl crate::Resettable for Lposh3Lposh3Spec {}

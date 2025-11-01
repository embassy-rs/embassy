#[doc = "Register `LPOSH1` reader"]
pub type R = crate::R<Lposh1Lposh1Spec>;
#[doc = "Field `LPOSH1` reader - LPOSH1"]
pub type Lposh1R = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - LPOSH1"]
    #[inline(always)]
    pub fn lposh1(&self) -> Lposh1R {
        Lposh1R::new(self.bits)
    }
}
#[doc = "Lower Position Holder Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`lposh1_lposh1::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Lposh1Lposh1Spec;
impl crate::RegisterSpec for Lposh1Lposh1Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`lposh1_lposh1::R`](R) reader structure"]
impl crate::Readable for Lposh1Lposh1Spec {}
#[doc = "`reset()` method sets LPOSH1 to value 0"]
impl crate::Resettable for Lposh1Lposh1Spec {}

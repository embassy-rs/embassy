#[doc = "Register `FCTRL0` reader"]
pub type R = crate::R<Fctrl0Spec>;
#[doc = "Register `FCTRL0` writer"]
pub type W = crate::W<Fctrl0Spec>;
#[doc = "Field `FCOUNT` reader - Result FIFO Counter"]
pub type FcountR = crate::FieldReader;
#[doc = "Field `FWMARK` reader - Watermark Level Selection"]
pub type FwmarkR = crate::FieldReader;
#[doc = "Field `FWMARK` writer - Watermark Level Selection"]
pub type FwmarkW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
impl R {
    #[doc = "Bits 0:3 - Result FIFO Counter"]
    #[inline(always)]
    pub fn fcount(&self) -> FcountR {
        FcountR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 16:18 - Watermark Level Selection"]
    #[inline(always)]
    pub fn fwmark(&self) -> FwmarkR {
        FwmarkR::new(((self.bits >> 16) & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 16:18 - Watermark Level Selection"]
    #[inline(always)]
    pub fn fwmark(&mut self) -> FwmarkW<Fctrl0Spec> {
        FwmarkW::new(self, 16)
    }
}
#[doc = "FIFO Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`fctrl0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fctrl0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Fctrl0Spec;
impl crate::RegisterSpec for Fctrl0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fctrl0::R`](R) reader structure"]
impl crate::Readable for Fctrl0Spec {}
#[doc = "`write(|w| ..)` method takes [`fctrl0::W`](W) writer structure"]
impl crate::Writable for Fctrl0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FCTRL0 to value 0"]
impl crate::Resettable for Fctrl0Spec {}

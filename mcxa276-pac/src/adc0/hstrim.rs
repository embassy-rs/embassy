#[doc = "Register `HSTRIM` reader"]
pub type R = crate::R<HstrimSpec>;
#[doc = "Register `HSTRIM` writer"]
pub type W = crate::W<HstrimSpec>;
#[doc = "Field `HSTRIM` reader - Trim for High Speed Conversions"]
pub type HstrimR = crate::FieldReader;
#[doc = "Field `HSTRIM` writer - Trim for High Speed Conversions"]
pub type HstrimW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
impl R {
    #[doc = "Bits 0:4 - Trim for High Speed Conversions"]
    #[inline(always)]
    pub fn hstrim(&self) -> HstrimR {
        HstrimR::new((self.bits & 0x1f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:4 - Trim for High Speed Conversions"]
    #[inline(always)]
    pub fn hstrim(&mut self) -> HstrimW<HstrimSpec> {
        HstrimW::new(self, 0)
    }
}
#[doc = "High Speed Trim Register\n\nYou can [`read`](crate::Reg::read) this register and get [`hstrim::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`hstrim::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct HstrimSpec;
impl crate::RegisterSpec for HstrimSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`hstrim::R`](R) reader structure"]
impl crate::Readable for HstrimSpec {}
#[doc = "`write(|w| ..)` method takes [`hstrim::W`](W) writer structure"]
impl crate::Writable for HstrimSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets HSTRIM to value 0"]
impl crate::Resettable for HstrimSpec {}

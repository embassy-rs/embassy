#[doc = "Register `RNR` reader"]
pub type R = crate::R<RnrSpec>;
#[doc = "Register `RNR` writer"]
pub type W = crate::W<RnrSpec>;
#[doc = "Field `REGION` reader - Region number."]
pub type RegionR = crate::FieldReader;
#[doc = "Field `REGION` writer - Region number."]
pub type RegionW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Region number."]
    #[inline(always)]
    pub fn region(&self) -> RegionR {
        RegionR::new((self.bits & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Region number."]
    #[inline(always)]
    pub fn region(&mut self) -> RegionW<RnrSpec> {
        RegionW::new(self, 0)
    }
}
#[doc = "Security Attribution Unit Region Number Register\n\nYou can [`read`](crate::Reg::read) this register and get [`rnr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rnr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RnrSpec;
impl crate::RegisterSpec for RnrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rnr::R`](R) reader structure"]
impl crate::Readable for RnrSpec {}
#[doc = "`write(|w| ..)` method takes [`rnr::W`](W) writer structure"]
impl crate::Writable for RnrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RNR to value 0"]
impl crate::Resettable for RnrSpec {}

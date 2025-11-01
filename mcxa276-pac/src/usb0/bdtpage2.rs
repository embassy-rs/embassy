#[doc = "Register `BDTPAGE2` reader"]
pub type R = crate::R<Bdtpage2Spec>;
#[doc = "Register `BDTPAGE2` writer"]
pub type W = crate::W<Bdtpage2Spec>;
#[doc = "Field `BDTBA` reader - BDT Base Address"]
pub type BdtbaR = crate::FieldReader;
#[doc = "Field `BDTBA` writer - BDT Base Address"]
pub type BdtbaW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - BDT Base Address"]
    #[inline(always)]
    pub fn bdtba(&self) -> BdtbaR {
        BdtbaR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:7 - BDT Base Address"]
    #[inline(always)]
    pub fn bdtba(&mut self) -> BdtbaW<Bdtpage2Spec> {
        BdtbaW::new(self, 0)
    }
}
#[doc = "BDT Page 2\n\nYou can [`read`](crate::Reg::read) this register and get [`bdtpage2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bdtpage2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Bdtpage2Spec;
impl crate::RegisterSpec for Bdtpage2Spec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`bdtpage2::R`](R) reader structure"]
impl crate::Readable for Bdtpage2Spec {}
#[doc = "`write(|w| ..)` method takes [`bdtpage2::W`](W) writer structure"]
impl crate::Writable for Bdtpage2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets BDTPAGE2 to value 0"]
impl crate::Resettable for Bdtpage2Spec {}

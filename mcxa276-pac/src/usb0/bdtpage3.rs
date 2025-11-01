#[doc = "Register `BDTPAGE3` reader"]
pub type R = crate::R<Bdtpage3Spec>;
#[doc = "Register `BDTPAGE3` writer"]
pub type W = crate::W<Bdtpage3Spec>;
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
    pub fn bdtba(&mut self) -> BdtbaW<Bdtpage3Spec> {
        BdtbaW::new(self, 0)
    }
}
#[doc = "BDT Page 3\n\nYou can [`read`](crate::Reg::read) this register and get [`bdtpage3::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bdtpage3::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Bdtpage3Spec;
impl crate::RegisterSpec for Bdtpage3Spec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`bdtpage3::R`](R) reader structure"]
impl crate::Readable for Bdtpage3Spec {}
#[doc = "`write(|w| ..)` method takes [`bdtpage3::W`](W) writer structure"]
impl crate::Writable for Bdtpage3Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets BDTPAGE3 to value 0"]
impl crate::Resettable for Bdtpage3Spec {}

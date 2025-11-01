#[doc = "Register `BDTPAGE1` reader"]
pub type R = crate::R<Bdtpage1Spec>;
#[doc = "Register `BDTPAGE1` writer"]
pub type W = crate::W<Bdtpage1Spec>;
#[doc = "Field `BDTBA` reader - BDT Base Address"]
pub type BdtbaR = crate::FieldReader;
#[doc = "Field `BDTBA` writer - BDT Base Address"]
pub type BdtbaW<'a, REG> = crate::FieldWriter<'a, REG, 7>;
impl R {
    #[doc = "Bits 1:7 - BDT Base Address"]
    #[inline(always)]
    pub fn bdtba(&self) -> BdtbaR {
        BdtbaR::new((self.bits >> 1) & 0x7f)
    }
}
impl W {
    #[doc = "Bits 1:7 - BDT Base Address"]
    #[inline(always)]
    pub fn bdtba(&mut self) -> BdtbaW<Bdtpage1Spec> {
        BdtbaW::new(self, 1)
    }
}
#[doc = "BDT Page 1\n\nYou can [`read`](crate::Reg::read) this register and get [`bdtpage1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bdtpage1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Bdtpage1Spec;
impl crate::RegisterSpec for Bdtpage1Spec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`bdtpage1::R`](R) reader structure"]
impl crate::Readable for Bdtpage1Spec {}
#[doc = "`write(|w| ..)` method takes [`bdtpage1::W`](W) writer structure"]
impl crate::Writable for Bdtpage1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets BDTPAGE1 to value 0"]
impl crate::Resettable for Bdtpage1Spec {}

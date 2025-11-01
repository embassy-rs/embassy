#[doc = "Register `sgi_datouta` reader"]
pub type R = crate::R<SgiDatoutaSpec>;
#[doc = "Register `sgi_datouta` writer"]
pub type W = crate::W<SgiDatoutaSpec>;
#[doc = "Field `datouta` reader - Output Data register"]
pub type DatoutaR = crate::FieldReader<u32>;
#[doc = "Field `datouta` writer - Output Data register"]
pub type DatoutaW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Output Data register"]
    #[inline(always)]
    pub fn datouta(&self) -> DatoutaR {
        DatoutaR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Output Data register"]
    #[inline(always)]
    pub fn datouta(&mut self) -> DatoutaW<SgiDatoutaSpec> {
        DatoutaW::new(self, 0)
    }
}
#[doc = "Output Data register - Word-3\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datouta::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datouta::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatoutaSpec;
impl crate::RegisterSpec for SgiDatoutaSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datouta::R`](R) reader structure"]
impl crate::Readable for SgiDatoutaSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datouta::W`](W) writer structure"]
impl crate::Writable for SgiDatoutaSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datouta to value 0"]
impl crate::Resettable for SgiDatoutaSpec {}

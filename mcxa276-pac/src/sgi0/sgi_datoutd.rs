#[doc = "Register `sgi_datoutd` reader"]
pub type R = crate::R<SgiDatoutdSpec>;
#[doc = "Register `sgi_datoutd` writer"]
pub type W = crate::W<SgiDatoutdSpec>;
#[doc = "Field `datoutd` reader - Output Data register"]
pub type DatoutdR = crate::FieldReader<u32>;
#[doc = "Field `datoutd` writer - Output Data register"]
pub type DatoutdW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Output Data register"]
    #[inline(always)]
    pub fn datoutd(&self) -> DatoutdR {
        DatoutdR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Output Data register"]
    #[inline(always)]
    pub fn datoutd(&mut self) -> DatoutdW<SgiDatoutdSpec> {
        DatoutdW::new(self, 0)
    }
}
#[doc = "Output Data register - Word-0\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datoutd::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datoutd::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatoutdSpec;
impl crate::RegisterSpec for SgiDatoutdSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datoutd::R`](R) reader structure"]
impl crate::Readable for SgiDatoutdSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datoutd::W`](W) writer structure"]
impl crate::Writable for SgiDatoutdSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datoutd to value 0"]
impl crate::Resettable for SgiDatoutdSpec {}

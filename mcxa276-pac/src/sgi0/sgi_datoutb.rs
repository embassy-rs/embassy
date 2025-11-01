#[doc = "Register `sgi_datoutb` reader"]
pub type R = crate::R<SgiDatoutbSpec>;
#[doc = "Register `sgi_datoutb` writer"]
pub type W = crate::W<SgiDatoutbSpec>;
#[doc = "Field `datoutb` reader - Output Data register"]
pub type DatoutbR = crate::FieldReader<u32>;
#[doc = "Field `datoutb` writer - Output Data register"]
pub type DatoutbW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Output Data register"]
    #[inline(always)]
    pub fn datoutb(&self) -> DatoutbR {
        DatoutbR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Output Data register"]
    #[inline(always)]
    pub fn datoutb(&mut self) -> DatoutbW<SgiDatoutbSpec> {
        DatoutbW::new(self, 0)
    }
}
#[doc = "Output Data register - Word-2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datoutb::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datoutb::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatoutbSpec;
impl crate::RegisterSpec for SgiDatoutbSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datoutb::R`](R) reader structure"]
impl crate::Readable for SgiDatoutbSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datoutb::W`](W) writer structure"]
impl crate::Writable for SgiDatoutbSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datoutb to value 0"]
impl crate::Resettable for SgiDatoutbSpec {}

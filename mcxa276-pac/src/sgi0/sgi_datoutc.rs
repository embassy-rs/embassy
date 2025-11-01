#[doc = "Register `sgi_datoutc` reader"]
pub type R = crate::R<SgiDatoutcSpec>;
#[doc = "Register `sgi_datoutc` writer"]
pub type W = crate::W<SgiDatoutcSpec>;
#[doc = "Field `datoutc` reader - Output Data register"]
pub type DatoutcR = crate::FieldReader<u32>;
#[doc = "Field `datoutc` writer - Output Data register"]
pub type DatoutcW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Output Data register"]
    #[inline(always)]
    pub fn datoutc(&self) -> DatoutcR {
        DatoutcR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Output Data register"]
    #[inline(always)]
    pub fn datoutc(&mut self) -> DatoutcW<SgiDatoutcSpec> {
        DatoutcW::new(self, 0)
    }
}
#[doc = "Output Data register - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datoutc::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datoutc::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatoutcSpec;
impl crate::RegisterSpec for SgiDatoutcSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datoutc::R`](R) reader structure"]
impl crate::Readable for SgiDatoutcSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datoutc::W`](W) writer structure"]
impl crate::Writable for SgiDatoutcSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datoutc to value 0"]
impl crate::Resettable for SgiDatoutcSpec {}

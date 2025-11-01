#[doc = "Register `sgi_datin1c` reader"]
pub type R = crate::R<SgiDatin1cSpec>;
#[doc = "Register `sgi_datin1c` writer"]
pub type W = crate::W<SgiDatin1cSpec>;
#[doc = "Field `datin1c` reader - Input Data register"]
pub type Datin1cR = crate::FieldReader<u32>;
#[doc = "Field `datin1c` writer - Input Data register"]
pub type Datin1cW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin1c(&self) -> Datin1cR {
        Datin1cR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin1c(&mut self) -> Datin1cW<SgiDatin1cSpec> {
        Datin1cW::new(self, 0)
    }
}
#[doc = "Input Data register 1 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin1c::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin1c::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin1cSpec;
impl crate::RegisterSpec for SgiDatin1cSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin1c::R`](R) reader structure"]
impl crate::Readable for SgiDatin1cSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin1c::W`](W) writer structure"]
impl crate::Writable for SgiDatin1cSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin1c to value 0"]
impl crate::Resettable for SgiDatin1cSpec {}

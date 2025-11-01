#[doc = "Register `sgi_datin2c` reader"]
pub type R = crate::R<SgiDatin2cSpec>;
#[doc = "Register `sgi_datin2c` writer"]
pub type W = crate::W<SgiDatin2cSpec>;
#[doc = "Field `datin2c` reader - Input Data register"]
pub type Datin2cR = crate::FieldReader<u32>;
#[doc = "Field `datin2c` writer - Input Data register"]
pub type Datin2cW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin2c(&self) -> Datin2cR {
        Datin2cR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Input Data register"]
    #[inline(always)]
    pub fn datin2c(&mut self) -> Datin2cW<SgiDatin2cSpec> {
        Datin2cW::new(self, 0)
    }
}
#[doc = "Input Data register 2 - Word-1\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_datin2c::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_datin2c::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiDatin2cSpec;
impl crate::RegisterSpec for SgiDatin2cSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_datin2c::R`](R) reader structure"]
impl crate::Readable for SgiDatin2cSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_datin2c::W`](W) writer structure"]
impl crate::Writable for SgiDatin2cSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_datin2c to value 0"]
impl crate::Resettable for SgiDatin2cSpec {}

#[doc = "Register `PKC_MCDATA` reader"]
pub type R = crate::R<PkcMcdataSpec>;
#[doc = "Register `PKC_MCDATA` writer"]
pub type W = crate::W<PkcMcdataSpec>;
#[doc = "Field `MCDATA` reader - Microcode read/write data"]
pub type McdataR = crate::FieldReader<u32>;
#[doc = "Field `MCDATA` writer - Microcode read/write data"]
pub type McdataW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Microcode read/write data"]
    #[inline(always)]
    pub fn mcdata(&self) -> McdataR {
        McdataR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Microcode read/write data"]
    #[inline(always)]
    pub fn mcdata(&mut self) -> McdataW<PkcMcdataSpec> {
        McdataW::new(self, 0)
    }
}
#[doc = "MC pattern data interface\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_mcdata::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_mcdata::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcMcdataSpec;
impl crate::RegisterSpec for PkcMcdataSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_mcdata::R`](R) reader structure"]
impl crate::Readable for PkcMcdataSpec {}
#[doc = "`write(|w| ..)` method takes [`pkc_mcdata::W`](W) writer structure"]
impl crate::Writable for PkcMcdataSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_MCDATA to value 0"]
impl crate::Resettable for PkcMcdataSpec {}

#[doc = "Register `SIRCSTAT` reader"]
pub type R = crate::R<SircstatSpec>;
#[doc = "Register `SIRCSTAT` writer"]
pub type W = crate::W<SircstatSpec>;
#[doc = "Field `CCOTRIM` reader - CCO Trim"]
pub type CcotrimR = crate::FieldReader;
#[doc = "Field `CCOTRIM` writer - CCO Trim"]
pub type CcotrimW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `CLTRIM` reader - CL Trim"]
pub type CltrimR = crate::FieldReader;
#[doc = "Field `CLTRIM` writer - CL Trim"]
pub type CltrimW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
impl R {
    #[doc = "Bits 0:5 - CCO Trim"]
    #[inline(always)]
    pub fn ccotrim(&self) -> CcotrimR {
        CcotrimR::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bits 8:13 - CL Trim"]
    #[inline(always)]
    pub fn cltrim(&self) -> CltrimR {
        CltrimR::new(((self.bits >> 8) & 0x3f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:5 - CCO Trim"]
    #[inline(always)]
    pub fn ccotrim(&mut self) -> CcotrimW<SircstatSpec> {
        CcotrimW::new(self, 0)
    }
    #[doc = "Bits 8:13 - CL Trim"]
    #[inline(always)]
    pub fn cltrim(&mut self) -> CltrimW<SircstatSpec> {
        CltrimW::new(self, 8)
    }
}
#[doc = "SIRC Auto-trimming Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sircstat::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sircstat::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SircstatSpec;
impl crate::RegisterSpec for SircstatSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sircstat::R`](R) reader structure"]
impl crate::Readable for SircstatSpec {}
#[doc = "`write(|w| ..)` method takes [`sircstat::W`](W) writer structure"]
impl crate::Writable for SircstatSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SIRCSTAT to value 0"]
impl crate::Resettable for SircstatSpec {}

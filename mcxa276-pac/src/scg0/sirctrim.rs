#[doc = "Register `SIRCTRIM` reader"]
pub type R = crate::R<SirctrimSpec>;
#[doc = "Register `SIRCTRIM` writer"]
pub type W = crate::W<SirctrimSpec>;
#[doc = "Field `CCOTRIM` reader - CCO Trim"]
pub type CcotrimR = crate::FieldReader;
#[doc = "Field `CCOTRIM` writer - CCO Trim"]
pub type CcotrimW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `CLTRIM` reader - CL Trim"]
pub type CltrimR = crate::FieldReader;
#[doc = "Field `CLTRIM` writer - CL Trim"]
pub type CltrimW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `TCTRIM` reader - Trim Temp"]
pub type TctrimR = crate::FieldReader;
#[doc = "Field `TCTRIM` writer - Trim Temp"]
pub type TctrimW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Field `FVCHTRIM` reader - Calibrates the replica voltage in FSU for CCO to get well frequency at initial period"]
pub type FvchtrimR = crate::FieldReader;
#[doc = "Field `FVCHTRIM` writer - Calibrates the replica voltage in FSU for CCO to get well frequency at initial period"]
pub type FvchtrimW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
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
    #[doc = "Bits 16:20 - Trim Temp"]
    #[inline(always)]
    pub fn tctrim(&self) -> TctrimR {
        TctrimR::new(((self.bits >> 16) & 0x1f) as u8)
    }
    #[doc = "Bits 24:28 - Calibrates the replica voltage in FSU for CCO to get well frequency at initial period"]
    #[inline(always)]
    pub fn fvchtrim(&self) -> FvchtrimR {
        FvchtrimR::new(((self.bits >> 24) & 0x1f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:5 - CCO Trim"]
    #[inline(always)]
    pub fn ccotrim(&mut self) -> CcotrimW<SirctrimSpec> {
        CcotrimW::new(self, 0)
    }
    #[doc = "Bits 8:13 - CL Trim"]
    #[inline(always)]
    pub fn cltrim(&mut self) -> CltrimW<SirctrimSpec> {
        CltrimW::new(self, 8)
    }
    #[doc = "Bits 16:20 - Trim Temp"]
    #[inline(always)]
    pub fn tctrim(&mut self) -> TctrimW<SirctrimSpec> {
        TctrimW::new(self, 16)
    }
    #[doc = "Bits 24:28 - Calibrates the replica voltage in FSU for CCO to get well frequency at initial period"]
    #[inline(always)]
    pub fn fvchtrim(&mut self) -> FvchtrimW<SirctrimSpec> {
        FvchtrimW::new(self, 24)
    }
}
#[doc = "SIRC Trim Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sirctrim::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sirctrim::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SirctrimSpec;
impl crate::RegisterSpec for SirctrimSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sirctrim::R`](R) reader structure"]
impl crate::Readable for SirctrimSpec {}
#[doc = "`write(|w| ..)` method takes [`sirctrim::W`](W) writer structure"]
impl crate::Writable for SirctrimSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SIRCTRIM to value 0"]
impl crate::Resettable for SirctrimSpec {}

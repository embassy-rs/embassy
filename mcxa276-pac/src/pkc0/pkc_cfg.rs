#[doc = "Register `PKC_CFG` reader"]
pub type R = crate::R<PkcCfgSpec>;
#[doc = "Register `PKC_CFG` writer"]
pub type W = crate::W<PkcCfgSpec>;
#[doc = "Field `IDLEOP` reader - Idle operation feature not available in this version (flag is don't care)."]
pub type IdleopR = crate::BitReader;
#[doc = "Field `IDLEOP` writer - Idle operation feature not available in this version (flag is don't care)."]
pub type IdleopW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RFU1` reader - RFU"]
pub type Rfu1R = crate::BitReader;
#[doc = "Field `RFU1` writer - RFU"]
pub type Rfu1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RFU2` reader - RFU"]
pub type Rfu2R = crate::BitReader;
#[doc = "Field `RFU2` writer - RFU"]
pub type Rfu2W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CLKRND` reader - Clock randomization feature not available in this version (flag is don't care)."]
pub type ClkrndR = crate::BitReader;
#[doc = "Field `CLKRND` writer - Clock randomization feature not available in this version (flag is don't care)."]
pub type ClkrndW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `REDMULNOISE` reader - Noise in reduced multiplier mode feature not available in this version (flag is don't care)."]
pub type RedmulnoiseR = crate::BitReader;
#[doc = "Field `REDMULNOISE` writer - Noise in reduced multiplier mode feature not available in this version (flag is don't care)."]
pub type RedmulnoiseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RNDDLY` reader - Random delay feature not available in this version (flag is don't care)."]
pub type RnddlyR = crate::FieldReader;
#[doc = "Field `RNDDLY` writer - Random delay feature not available in this version (flag is don't care)."]
pub type RnddlyW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `SBXNOISE` reader - Noise feature not available in this version (flag is don't care)."]
pub type SbxnoiseR = crate::BitReader;
#[doc = "Field `SBXNOISE` writer - Noise feature not available in this version (flag is don't care)."]
pub type SbxnoiseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ALPNOISE` reader - Noise feature not available in this version (flag is don't care)."]
pub type AlpnoiseR = crate::BitReader;
#[doc = "Field `ALPNOISE` writer - Noise feature not available in this version (flag is don't care)."]
pub type AlpnoiseW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `FMULNOISE` reader - Noise feature not available in this version (flag is don't care)."]
pub type FmulnoiseR = crate::BitReader;
#[doc = "Field `FMULNOISE` writer - Noise feature not available in this version (flag is don't care)."]
pub type FmulnoiseW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - Idle operation feature not available in this version (flag is don't care)."]
    #[inline(always)]
    pub fn idleop(&self) -> IdleopR {
        IdleopR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - RFU"]
    #[inline(always)]
    pub fn rfu1(&self) -> Rfu1R {
        Rfu1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - RFU"]
    #[inline(always)]
    pub fn rfu2(&self) -> Rfu2R {
        Rfu2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Clock randomization feature not available in this version (flag is don't care)."]
    #[inline(always)]
    pub fn clkrnd(&self) -> ClkrndR {
        ClkrndR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Noise in reduced multiplier mode feature not available in this version (flag is don't care)."]
    #[inline(always)]
    pub fn redmulnoise(&self) -> RedmulnoiseR {
        RedmulnoiseR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bits 5:7 - Random delay feature not available in this version (flag is don't care)."]
    #[inline(always)]
    pub fn rnddly(&self) -> RnddlyR {
        RnddlyR::new(((self.bits >> 5) & 7) as u8)
    }
    #[doc = "Bit 8 - Noise feature not available in this version (flag is don't care)."]
    #[inline(always)]
    pub fn sbxnoise(&self) -> SbxnoiseR {
        SbxnoiseR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Noise feature not available in this version (flag is don't care)."]
    #[inline(always)]
    pub fn alpnoise(&self) -> AlpnoiseR {
        AlpnoiseR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Noise feature not available in this version (flag is don't care)."]
    #[inline(always)]
    pub fn fmulnoise(&self) -> FmulnoiseR {
        FmulnoiseR::new(((self.bits >> 10) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Idle operation feature not available in this version (flag is don't care)."]
    #[inline(always)]
    pub fn idleop(&mut self) -> IdleopW<PkcCfgSpec> {
        IdleopW::new(self, 0)
    }
    #[doc = "Bit 1 - RFU"]
    #[inline(always)]
    pub fn rfu1(&mut self) -> Rfu1W<PkcCfgSpec> {
        Rfu1W::new(self, 1)
    }
    #[doc = "Bit 2 - RFU"]
    #[inline(always)]
    pub fn rfu2(&mut self) -> Rfu2W<PkcCfgSpec> {
        Rfu2W::new(self, 2)
    }
    #[doc = "Bit 3 - Clock randomization feature not available in this version (flag is don't care)."]
    #[inline(always)]
    pub fn clkrnd(&mut self) -> ClkrndW<PkcCfgSpec> {
        ClkrndW::new(self, 3)
    }
    #[doc = "Bit 4 - Noise in reduced multiplier mode feature not available in this version (flag is don't care)."]
    #[inline(always)]
    pub fn redmulnoise(&mut self) -> RedmulnoiseW<PkcCfgSpec> {
        RedmulnoiseW::new(self, 4)
    }
    #[doc = "Bits 5:7 - Random delay feature not available in this version (flag is don't care)."]
    #[inline(always)]
    pub fn rnddly(&mut self) -> RnddlyW<PkcCfgSpec> {
        RnddlyW::new(self, 5)
    }
    #[doc = "Bit 8 - Noise feature not available in this version (flag is don't care)."]
    #[inline(always)]
    pub fn sbxnoise(&mut self) -> SbxnoiseW<PkcCfgSpec> {
        SbxnoiseW::new(self, 8)
    }
    #[doc = "Bit 9 - Noise feature not available in this version (flag is don't care)."]
    #[inline(always)]
    pub fn alpnoise(&mut self) -> AlpnoiseW<PkcCfgSpec> {
        AlpnoiseW::new(self, 9)
    }
    #[doc = "Bit 10 - Noise feature not available in this version (flag is don't care)."]
    #[inline(always)]
    pub fn fmulnoise(&mut self) -> FmulnoiseW<PkcCfgSpec> {
        FmulnoiseW::new(self, 10)
    }
}
#[doc = "Configuration register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_cfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_cfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcCfgSpec;
impl crate::RegisterSpec for PkcCfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_cfg::R`](R) reader structure"]
impl crate::Readable for PkcCfgSpec {}
#[doc = "`write(|w| ..)` method takes [`pkc_cfg::W`](W) writer structure"]
impl crate::Writable for PkcCfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_CFG to value 0x0719"]
impl crate::Resettable for PkcCfgSpec {
    const RESET_VALUE: u32 = 0x0719;
}

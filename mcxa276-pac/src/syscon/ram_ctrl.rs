#[doc = "Register `RAM_CTRL` reader"]
pub type R = crate::R<RamCtrlSpec>;
#[doc = "Register `RAM_CTRL` writer"]
pub type W = crate::W<RamCtrlSpec>;
#[doc = "RAMA ECC enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RamaEccEnable {
    #[doc = "0: ECC is disabled"]
    Disable = 0,
    #[doc = "1: ECC is enabled"]
    Enable = 1,
}
impl From<RamaEccEnable> for bool {
    #[inline(always)]
    fn from(variant: RamaEccEnable) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RAMA_ECC_ENABLE` reader - RAMA ECC enable"]
pub type RamaEccEnableR = crate::BitReader<RamaEccEnable>;
impl RamaEccEnableR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RamaEccEnable {
        match self.bits {
            false => RamaEccEnable::Disable,
            true => RamaEccEnable::Enable,
        }
    }
    #[doc = "ECC is disabled"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RamaEccEnable::Disable
    }
    #[doc = "ECC is enabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RamaEccEnable::Enable
    }
}
#[doc = "Field `RAMA_ECC_ENABLE` writer - RAMA ECC enable"]
pub type RamaEccEnableW<'a, REG> = crate::BitWriter<'a, REG, RamaEccEnable>;
impl<'a, REG> RamaEccEnableW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "ECC is disabled"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RamaEccEnable::Disable)
    }
    #[doc = "ECC is enabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RamaEccEnable::Enable)
    }
}
#[doc = "RAMA bank clock gating control, only avaiable when RAMA_ECC_ENABLE = 0.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RamaCgOverride {
    #[doc = "0: Memory bank clock is gated automatically if no access more than 16 clock cycles"]
    Disable = 0,
    #[doc = "1: Auto clock gating feature is disabled"]
    Enable = 1,
}
impl From<RamaCgOverride> for bool {
    #[inline(always)]
    fn from(variant: RamaCgOverride) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RAMA_CG_OVERRIDE` reader - RAMA bank clock gating control, only avaiable when RAMA_ECC_ENABLE = 0."]
pub type RamaCgOverrideR = crate::BitReader<RamaCgOverride>;
impl RamaCgOverrideR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RamaCgOverride {
        match self.bits {
            false => RamaCgOverride::Disable,
            true => RamaCgOverride::Enable,
        }
    }
    #[doc = "Memory bank clock is gated automatically if no access more than 16 clock cycles"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RamaCgOverride::Disable
    }
    #[doc = "Auto clock gating feature is disabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RamaCgOverride::Enable
    }
}
#[doc = "Field `RAMA_CG_OVERRIDE` writer - RAMA bank clock gating control, only avaiable when RAMA_ECC_ENABLE = 0."]
pub type RamaCgOverrideW<'a, REG> = crate::BitWriter<'a, REG, RamaCgOverride>;
impl<'a, REG> RamaCgOverrideW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Memory bank clock is gated automatically if no access more than 16 clock cycles"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RamaCgOverride::Disable)
    }
    #[doc = "Auto clock gating feature is disabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RamaCgOverride::Enable)
    }
}
#[doc = "RAMX bank clock gating control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RamxCgOverride {
    #[doc = "0: Memory bank clock is gated automatically if no access more than 16 clock cycles"]
    Disable = 0,
    #[doc = "1: Auto clock gating feature is disabled"]
    Enable = 1,
}
impl From<RamxCgOverride> for bool {
    #[inline(always)]
    fn from(variant: RamxCgOverride) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RAMX_CG_OVERRIDE` reader - RAMX bank clock gating control"]
pub type RamxCgOverrideR = crate::BitReader<RamxCgOverride>;
impl RamxCgOverrideR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RamxCgOverride {
        match self.bits {
            false => RamxCgOverride::Disable,
            true => RamxCgOverride::Enable,
        }
    }
    #[doc = "Memory bank clock is gated automatically if no access more than 16 clock cycles"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RamxCgOverride::Disable
    }
    #[doc = "Auto clock gating feature is disabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RamxCgOverride::Enable
    }
}
#[doc = "Field `RAMX_CG_OVERRIDE` writer - RAMX bank clock gating control"]
pub type RamxCgOverrideW<'a, REG> = crate::BitWriter<'a, REG, RamxCgOverride>;
impl<'a, REG> RamxCgOverrideW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Memory bank clock is gated automatically if no access more than 16 clock cycles"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RamxCgOverride::Disable)
    }
    #[doc = "Auto clock gating feature is disabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RamxCgOverride::Enable)
    }
}
#[doc = "RAMB bank clock gating control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RambCgOverride {
    #[doc = "0: Memory bank clock is gated automatically if no access more than 16 clock cycles"]
    Disable = 0,
    #[doc = "1: Auto clock gating feature is disabled"]
    Enable = 1,
}
impl From<RambCgOverride> for bool {
    #[inline(always)]
    fn from(variant: RambCgOverride) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RAMB_CG_OVERRIDE` reader - RAMB bank clock gating control"]
pub type RambCgOverrideR = crate::BitReader<RambCgOverride>;
impl RambCgOverrideR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RambCgOverride {
        match self.bits {
            false => RambCgOverride::Disable,
            true => RambCgOverride::Enable,
        }
    }
    #[doc = "Memory bank clock is gated automatically if no access more than 16 clock cycles"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RambCgOverride::Disable
    }
    #[doc = "Auto clock gating feature is disabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RambCgOverride::Enable
    }
}
#[doc = "Field `RAMB_CG_OVERRIDE` writer - RAMB bank clock gating control"]
pub type RambCgOverrideW<'a, REG> = crate::BitWriter<'a, REG, RambCgOverride>;
impl<'a, REG> RambCgOverrideW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Memory bank clock is gated automatically if no access more than 16 clock cycles"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RambCgOverride::Disable)
    }
    #[doc = "Auto clock gating feature is disabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RambCgOverride::Enable)
    }
}
#[doc = "RAMC bank clock gating control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RamcCgOverride {
    #[doc = "0: Memory bank clock is gated automatically if no access more than 16 clock cycles"]
    Disable = 0,
    #[doc = "1: Auto clock gating feature is disabled"]
    Enable = 1,
}
impl From<RamcCgOverride> for bool {
    #[inline(always)]
    fn from(variant: RamcCgOverride) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RAMC_CG_OVERRIDE` reader - RAMC bank clock gating control"]
pub type RamcCgOverrideR = crate::BitReader<RamcCgOverride>;
impl RamcCgOverrideR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RamcCgOverride {
        match self.bits {
            false => RamcCgOverride::Disable,
            true => RamcCgOverride::Enable,
        }
    }
    #[doc = "Memory bank clock is gated automatically if no access more than 16 clock cycles"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RamcCgOverride::Disable
    }
    #[doc = "Auto clock gating feature is disabled"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RamcCgOverride::Enable
    }
}
#[doc = "Field `RAMC_CG_OVERRIDE` writer - RAMC bank clock gating control"]
pub type RamcCgOverrideW<'a, REG> = crate::BitWriter<'a, REG, RamcCgOverride>;
impl<'a, REG> RamcCgOverrideW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Memory bank clock is gated automatically if no access more than 16 clock cycles"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RamcCgOverride::Disable)
    }
    #[doc = "Auto clock gating feature is disabled"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RamcCgOverride::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - RAMA ECC enable"]
    #[inline(always)]
    pub fn rama_ecc_enable(&self) -> RamaEccEnableR {
        RamaEccEnableR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 16 - RAMA bank clock gating control, only avaiable when RAMA_ECC_ENABLE = 0."]
    #[inline(always)]
    pub fn rama_cg_override(&self) -> RamaCgOverrideR {
        RamaCgOverrideR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - RAMX bank clock gating control"]
    #[inline(always)]
    pub fn ramx_cg_override(&self) -> RamxCgOverrideR {
        RamxCgOverrideR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - RAMB bank clock gating control"]
    #[inline(always)]
    pub fn ramb_cg_override(&self) -> RambCgOverrideR {
        RambCgOverrideR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - RAMC bank clock gating control"]
    #[inline(always)]
    pub fn ramc_cg_override(&self) -> RamcCgOverrideR {
        RamcCgOverrideR::new(((self.bits >> 19) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - RAMA ECC enable"]
    #[inline(always)]
    pub fn rama_ecc_enable(&mut self) -> RamaEccEnableW<RamCtrlSpec> {
        RamaEccEnableW::new(self, 0)
    }
    #[doc = "Bit 16 - RAMA bank clock gating control, only avaiable when RAMA_ECC_ENABLE = 0."]
    #[inline(always)]
    pub fn rama_cg_override(&mut self) -> RamaCgOverrideW<RamCtrlSpec> {
        RamaCgOverrideW::new(self, 16)
    }
    #[doc = "Bit 17 - RAMX bank clock gating control"]
    #[inline(always)]
    pub fn ramx_cg_override(&mut self) -> RamxCgOverrideW<RamCtrlSpec> {
        RamxCgOverrideW::new(self, 17)
    }
    #[doc = "Bit 18 - RAMB bank clock gating control"]
    #[inline(always)]
    pub fn ramb_cg_override(&mut self) -> RambCgOverrideW<RamCtrlSpec> {
        RambCgOverrideW::new(self, 18)
    }
    #[doc = "Bit 19 - RAMC bank clock gating control"]
    #[inline(always)]
    pub fn ramc_cg_override(&mut self) -> RamcCgOverrideW<RamCtrlSpec> {
        RamcCgOverrideW::new(self, 19)
    }
}
#[doc = "RAM Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ram_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ram_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RamCtrlSpec;
impl crate::RegisterSpec for RamCtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ram_ctrl::R`](R) reader structure"]
impl crate::Readable for RamCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`ram_ctrl::W`](W) writer structure"]
impl crate::Writable for RamCtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RAM_CTRL to value 0x01"]
impl crate::Resettable for RamCtrlSpec {
    const RESET_VALUE: u32 = 0x01;
}

#[doc = "Register `LP_CFG` reader"]
pub type R = crate::R<LpCfgSpec>;
#[doc = "Register `LP_CFG` writer"]
pub type W = crate::W<LpCfgSpec>;
#[doc = "LDO_CORE VDD Drive Strength\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreldoVddDs {
    #[doc = "0: Low"]
    Low = 0,
    #[doc = "1: Normal"]
    Normal = 1,
}
impl From<CoreldoVddDs> for bool {
    #[inline(always)]
    fn from(variant: CoreldoVddDs) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CORELDO_VDD_DS` reader - LDO_CORE VDD Drive Strength"]
pub type CoreldoVddDsR = crate::BitReader<CoreldoVddDs>;
impl CoreldoVddDsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CoreldoVddDs {
        match self.bits {
            false => CoreldoVddDs::Low,
            true => CoreldoVddDs::Normal,
        }
    }
    #[doc = "Low"]
    #[inline(always)]
    pub fn is_low(&self) -> bool {
        *self == CoreldoVddDs::Low
    }
    #[doc = "Normal"]
    #[inline(always)]
    pub fn is_normal(&self) -> bool {
        *self == CoreldoVddDs::Normal
    }
}
#[doc = "Field `CORELDO_VDD_DS` writer - LDO_CORE VDD Drive Strength"]
pub type CoreldoVddDsW<'a, REG> = crate::BitWriter<'a, REG, CoreldoVddDs>;
impl<'a, REG> CoreldoVddDsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Low"]
    #[inline(always)]
    pub fn low(self) -> &'a mut crate::W<REG> {
        self.variant(CoreldoVddDs::Low)
    }
    #[doc = "Normal"]
    #[inline(always)]
    pub fn normal(self) -> &'a mut crate::W<REG> {
        self.variant(CoreldoVddDs::Normal)
    }
}
#[doc = "LDO_CORE VDD Regulator Voltage Level\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CoreldoVddLvl {
    #[doc = "1: Mid voltage (1.0 V)"]
    Mid = 1,
    #[doc = "2: Normal voltage (1.1 V)"]
    Normal = 2,
    #[doc = "3: Overdrive voltage (1.15 V)"]
    Over = 3,
}
impl From<CoreldoVddLvl> for u8 {
    #[inline(always)]
    fn from(variant: CoreldoVddLvl) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for CoreldoVddLvl {
    type Ux = u8;
}
impl crate::IsEnum for CoreldoVddLvl {}
#[doc = "Field `CORELDO_VDD_LVL` reader - LDO_CORE VDD Regulator Voltage Level"]
pub type CoreldoVddLvlR = crate::FieldReader<CoreldoVddLvl>;
impl CoreldoVddLvlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<CoreldoVddLvl> {
        match self.bits {
            1 => Some(CoreldoVddLvl::Mid),
            2 => Some(CoreldoVddLvl::Normal),
            3 => Some(CoreldoVddLvl::Over),
            _ => None,
        }
    }
    #[doc = "Mid voltage (1.0 V)"]
    #[inline(always)]
    pub fn is_mid(&self) -> bool {
        *self == CoreldoVddLvl::Mid
    }
    #[doc = "Normal voltage (1.1 V)"]
    #[inline(always)]
    pub fn is_normal(&self) -> bool {
        *self == CoreldoVddLvl::Normal
    }
    #[doc = "Overdrive voltage (1.15 V)"]
    #[inline(always)]
    pub fn is_over(&self) -> bool {
        *self == CoreldoVddLvl::Over
    }
}
#[doc = "Field `CORELDO_VDD_LVL` writer - LDO_CORE VDD Regulator Voltage Level"]
pub type CoreldoVddLvlW<'a, REG> = crate::FieldWriter<'a, REG, 2, CoreldoVddLvl>;
impl<'a, REG> CoreldoVddLvlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Mid voltage (1.0 V)"]
    #[inline(always)]
    pub fn mid(self) -> &'a mut crate::W<REG> {
        self.variant(CoreldoVddLvl::Mid)
    }
    #[doc = "Normal voltage (1.1 V)"]
    #[inline(always)]
    pub fn normal(self) -> &'a mut crate::W<REG> {
        self.variant(CoreldoVddLvl::Normal)
    }
    #[doc = "Overdrive voltage (1.15 V)"]
    #[inline(always)]
    pub fn over(self) -> &'a mut crate::W<REG> {
        self.variant(CoreldoVddLvl::Over)
    }
}
#[doc = "SRAM_LDO Deep Power Low Power IREF Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SramldoDpdOn {
    #[doc = "0: Low Power IREF is disabled for power saving in Deep Power Down mode"]
    Disabled = 0,
    #[doc = "1: Low Power IREF is enabled"]
    Enabled = 1,
}
impl From<SramldoDpdOn> for bool {
    #[inline(always)]
    fn from(variant: SramldoDpdOn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SRAMLDO_DPD_ON` reader - SRAM_LDO Deep Power Low Power IREF Enable"]
pub type SramldoDpdOnR = crate::BitReader<SramldoDpdOn>;
impl SramldoDpdOnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SramldoDpdOn {
        match self.bits {
            false => SramldoDpdOn::Disabled,
            true => SramldoDpdOn::Enabled,
        }
    }
    #[doc = "Low Power IREF is disabled for power saving in Deep Power Down mode"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == SramldoDpdOn::Disabled
    }
    #[doc = "Low Power IREF is enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == SramldoDpdOn::Enabled
    }
}
#[doc = "Field `SRAMLDO_DPD_ON` writer - SRAM_LDO Deep Power Low Power IREF Enable"]
pub type SramldoDpdOnW<'a, REG> = crate::BitWriter<'a, REG, SramldoDpdOn>;
impl<'a, REG> SramldoDpdOnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Low Power IREF is disabled for power saving in Deep Power Down mode"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(SramldoDpdOn::Disabled)
    }
    #[doc = "Low Power IREF is enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(SramldoDpdOn::Enabled)
    }
}
#[doc = "Bandgap Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Bgmode {
    #[doc = "0: Bandgap disabled"]
    Bgmode0 = 0,
    #[doc = "1: Bandgap enabled, buffer disabled"]
    Bgmode01 = 1,
    #[doc = "2: Bandgap enabled, buffer enabled"]
    Bgmode10 = 2,
}
impl From<Bgmode> for u8 {
    #[inline(always)]
    fn from(variant: Bgmode) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Bgmode {
    type Ux = u8;
}
impl crate::IsEnum for Bgmode {}
#[doc = "Field `BGMODE` reader - Bandgap Mode"]
pub type BgmodeR = crate::FieldReader<Bgmode>;
impl BgmodeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Bgmode> {
        match self.bits {
            0 => Some(Bgmode::Bgmode0),
            1 => Some(Bgmode::Bgmode01),
            2 => Some(Bgmode::Bgmode10),
            _ => None,
        }
    }
    #[doc = "Bandgap disabled"]
    #[inline(always)]
    pub fn is_bgmode0(&self) -> bool {
        *self == Bgmode::Bgmode0
    }
    #[doc = "Bandgap enabled, buffer disabled"]
    #[inline(always)]
    pub fn is_bgmode01(&self) -> bool {
        *self == Bgmode::Bgmode01
    }
    #[doc = "Bandgap enabled, buffer enabled"]
    #[inline(always)]
    pub fn is_bgmode10(&self) -> bool {
        *self == Bgmode::Bgmode10
    }
}
#[doc = "Field `BGMODE` writer - Bandgap Mode"]
pub type BgmodeW<'a, REG> = crate::FieldWriter<'a, REG, 2, Bgmode>;
impl<'a, REG> BgmodeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Bandgap disabled"]
    #[inline(always)]
    pub fn bgmode0(self) -> &'a mut crate::W<REG> {
        self.variant(Bgmode::Bgmode0)
    }
    #[doc = "Bandgap enabled, buffer disabled"]
    #[inline(always)]
    pub fn bgmode01(self) -> &'a mut crate::W<REG> {
        self.variant(Bgmode::Bgmode01)
    }
    #[doc = "Bandgap enabled, buffer enabled"]
    #[inline(always)]
    pub fn bgmode10(self) -> &'a mut crate::W<REG> {
        self.variant(Bgmode::Bgmode10)
    }
}
#[doc = "Low-Power IREF Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LpIrefen {
    #[doc = "0: Disable for power saving in Deep Power Down mode"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<LpIrefen> for bool {
    #[inline(always)]
    fn from(variant: LpIrefen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LP_IREFEN` reader - Low-Power IREF Enable"]
pub type LpIrefenR = crate::BitReader<LpIrefen>;
impl LpIrefenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> LpIrefen {
        match self.bits {
            false => LpIrefen::Disable,
            true => LpIrefen::Enable,
        }
    }
    #[doc = "Disable for power saving in Deep Power Down mode"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == LpIrefen::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == LpIrefen::Enable
    }
}
#[doc = "Field `LP_IREFEN` writer - Low-Power IREF Enable"]
pub type LpIrefenW<'a, REG> = crate::BitWriter<'a, REG, LpIrefen>;
impl<'a, REG> LpIrefenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable for power saving in Deep Power Down mode"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(LpIrefen::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(LpIrefen::Enable)
    }
}
#[doc = "Core Low Voltage Detect Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreLvde {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<CoreLvde> for bool {
    #[inline(always)]
    fn from(variant: CoreLvde) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CORE_LVDE` reader - Core Low Voltage Detect Enable"]
pub type CoreLvdeR = crate::BitReader<CoreLvde>;
impl CoreLvdeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CoreLvde {
        match self.bits {
            false => CoreLvde::Disable,
            true => CoreLvde::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == CoreLvde::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == CoreLvde::Enable
    }
}
#[doc = "Field `CORE_LVDE` writer - Core Low Voltage Detect Enable"]
pub type CoreLvdeW<'a, REG> = crate::BitWriter<'a, REG, CoreLvde>;
impl<'a, REG> CoreLvdeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(CoreLvde::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(CoreLvde::Enable)
    }
}
#[doc = "System Low Voltage Detect Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SysLvde {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<SysLvde> for bool {
    #[inline(always)]
    fn from(variant: SysLvde) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SYS_LVDE` reader - System Low Voltage Detect Enable"]
pub type SysLvdeR = crate::BitReader<SysLvde>;
impl SysLvdeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SysLvde {
        match self.bits {
            false => SysLvde::Disable,
            true => SysLvde::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == SysLvde::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == SysLvde::Enable
    }
}
#[doc = "Field `SYS_LVDE` writer - System Low Voltage Detect Enable"]
pub type SysLvdeW<'a, REG> = crate::BitWriter<'a, REG, SysLvde>;
impl<'a, REG> SysLvdeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(SysLvde::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(SysLvde::Enable)
    }
}
#[doc = "System High Voltage Detect Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SysHvde {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<SysHvde> for bool {
    #[inline(always)]
    fn from(variant: SysHvde) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SYS_HVDE` reader - System High Voltage Detect Enable"]
pub type SysHvdeR = crate::BitReader<SysHvde>;
impl SysHvdeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SysHvde {
        match self.bits {
            false => SysHvde::Disable,
            true => SysHvde::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == SysHvde::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == SysHvde::Enable
    }
}
#[doc = "Field `SYS_HVDE` writer - System High Voltage Detect Enable"]
pub type SysHvdeW<'a, REG> = crate::BitWriter<'a, REG, SysHvde>;
impl<'a, REG> SysHvdeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(SysHvde::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(SysHvde::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - LDO_CORE VDD Drive Strength"]
    #[inline(always)]
    pub fn coreldo_vdd_ds(&self) -> CoreldoVddDsR {
        CoreldoVddDsR::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 2:3 - LDO_CORE VDD Regulator Voltage Level"]
    #[inline(always)]
    pub fn coreldo_vdd_lvl(&self) -> CoreldoVddLvlR {
        CoreldoVddLvlR::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bit 19 - SRAM_LDO Deep Power Low Power IREF Enable"]
    #[inline(always)]
    pub fn sramldo_dpd_on(&self) -> SramldoDpdOnR {
        SramldoDpdOnR::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bits 20:21 - Bandgap Mode"]
    #[inline(always)]
    pub fn bgmode(&self) -> BgmodeR {
        BgmodeR::new(((self.bits >> 20) & 3) as u8)
    }
    #[doc = "Bit 23 - Low-Power IREF Enable"]
    #[inline(always)]
    pub fn lp_irefen(&self) -> LpIrefenR {
        LpIrefenR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - Core Low Voltage Detect Enable"]
    #[inline(always)]
    pub fn core_lvde(&self) -> CoreLvdeR {
        CoreLvdeR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - System Low Voltage Detect Enable"]
    #[inline(always)]
    pub fn sys_lvde(&self) -> SysLvdeR {
        SysLvdeR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 28 - System High Voltage Detect Enable"]
    #[inline(always)]
    pub fn sys_hvde(&self) -> SysHvdeR {
        SysHvdeR::new(((self.bits >> 28) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - LDO_CORE VDD Drive Strength"]
    #[inline(always)]
    pub fn coreldo_vdd_ds(&mut self) -> CoreldoVddDsW<LpCfgSpec> {
        CoreldoVddDsW::new(self, 0)
    }
    #[doc = "Bits 2:3 - LDO_CORE VDD Regulator Voltage Level"]
    #[inline(always)]
    pub fn coreldo_vdd_lvl(&mut self) -> CoreldoVddLvlW<LpCfgSpec> {
        CoreldoVddLvlW::new(self, 2)
    }
    #[doc = "Bit 19 - SRAM_LDO Deep Power Low Power IREF Enable"]
    #[inline(always)]
    pub fn sramldo_dpd_on(&mut self) -> SramldoDpdOnW<LpCfgSpec> {
        SramldoDpdOnW::new(self, 19)
    }
    #[doc = "Bits 20:21 - Bandgap Mode"]
    #[inline(always)]
    pub fn bgmode(&mut self) -> BgmodeW<LpCfgSpec> {
        BgmodeW::new(self, 20)
    }
    #[doc = "Bit 23 - Low-Power IREF Enable"]
    #[inline(always)]
    pub fn lp_irefen(&mut self) -> LpIrefenW<LpCfgSpec> {
        LpIrefenW::new(self, 23)
    }
    #[doc = "Bit 24 - Core Low Voltage Detect Enable"]
    #[inline(always)]
    pub fn core_lvde(&mut self) -> CoreLvdeW<LpCfgSpec> {
        CoreLvdeW::new(self, 24)
    }
    #[doc = "Bit 25 - System Low Voltage Detect Enable"]
    #[inline(always)]
    pub fn sys_lvde(&mut self) -> SysLvdeW<LpCfgSpec> {
        SysLvdeW::new(self, 25)
    }
    #[doc = "Bit 28 - System High Voltage Detect Enable"]
    #[inline(always)]
    pub fn sys_hvde(&mut self) -> SysHvdeW<LpCfgSpec> {
        SysHvdeW::new(self, 28)
    }
}
#[doc = "Low-Power Mode Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`lp_cfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lp_cfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LpCfgSpec;
impl crate::RegisterSpec for LpCfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`lp_cfg::R`](R) reader structure"]
impl crate::Readable for LpCfgSpec {}
#[doc = "`write(|w| ..)` method takes [`lp_cfg::W`](W) writer structure"]
impl crate::Writable for LpCfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LP_CFG to value 0x0008_0004"]
impl crate::Resettable for LpCfgSpec {
    const RESET_VALUE: u32 = 0x0008_0004;
}

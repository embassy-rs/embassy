#[doc = "Register `CFG` reader"]
pub type R = crate::R<CfgSpec>;
#[doc = "Register `CFG` writer"]
pub type W = crate::W<CfgSpec>;
#[doc = "ADC Trigger Priority Control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tprictrl {
    #[doc = "0: If a higher priority trigger is detected during command processing, the current conversion is aborted and the new command specified by the trigger is started."]
    AbortCurrentOnPriority = 0,
    #[doc = "1: If a higher priority trigger is received during command processing, the current command is stopped after completing the current conversion. If averaging is enabled, the averaging loop will be completed. However, CMDHa\\[LOOP\\] will be ignored and the higher priority trigger will be serviced."]
    FinishCurrentOnPriority = 1,
    #[doc = "2: If a higher priority trigger is received during command processing, the current command will be completed (averaging, looping, compare) before servicing the higher priority trigger."]
    FinishSequenceOnPriority = 2,
}
impl From<Tprictrl> for u8 {
    #[inline(always)]
    fn from(variant: Tprictrl) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Tprictrl {
    type Ux = u8;
}
impl crate::IsEnum for Tprictrl {}
#[doc = "Field `TPRICTRL` reader - ADC Trigger Priority Control"]
pub type TprictrlR = crate::FieldReader<Tprictrl>;
impl TprictrlR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Tprictrl> {
        match self.bits {
            0 => Some(Tprictrl::AbortCurrentOnPriority),
            1 => Some(Tprictrl::FinishCurrentOnPriority),
            2 => Some(Tprictrl::FinishSequenceOnPriority),
            _ => None,
        }
    }
    #[doc = "If a higher priority trigger is detected during command processing, the current conversion is aborted and the new command specified by the trigger is started."]
    #[inline(always)]
    pub fn is_abort_current_on_priority(&self) -> bool {
        *self == Tprictrl::AbortCurrentOnPriority
    }
    #[doc = "If a higher priority trigger is received during command processing, the current command is stopped after completing the current conversion. If averaging is enabled, the averaging loop will be completed. However, CMDHa\\[LOOP\\] will be ignored and the higher priority trigger will be serviced."]
    #[inline(always)]
    pub fn is_finish_current_on_priority(&self) -> bool {
        *self == Tprictrl::FinishCurrentOnPriority
    }
    #[doc = "If a higher priority trigger is received during command processing, the current command will be completed (averaging, looping, compare) before servicing the higher priority trigger."]
    #[inline(always)]
    pub fn is_finish_sequence_on_priority(&self) -> bool {
        *self == Tprictrl::FinishSequenceOnPriority
    }
}
#[doc = "Field `TPRICTRL` writer - ADC Trigger Priority Control"]
pub type TprictrlW<'a, REG> = crate::FieldWriter<'a, REG, 2, Tprictrl>;
impl<'a, REG> TprictrlW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "If a higher priority trigger is detected during command processing, the current conversion is aborted and the new command specified by the trigger is started."]
    #[inline(always)]
    pub fn abort_current_on_priority(self) -> &'a mut crate::W<REG> {
        self.variant(Tprictrl::AbortCurrentOnPriority)
    }
    #[doc = "If a higher priority trigger is received during command processing, the current command is stopped after completing the current conversion. If averaging is enabled, the averaging loop will be completed. However, CMDHa\\[LOOP\\] will be ignored and the higher priority trigger will be serviced."]
    #[inline(always)]
    pub fn finish_current_on_priority(self) -> &'a mut crate::W<REG> {
        self.variant(Tprictrl::FinishCurrentOnPriority)
    }
    #[doc = "If a higher priority trigger is received during command processing, the current command will be completed (averaging, looping, compare) before servicing the higher priority trigger."]
    #[inline(always)]
    pub fn finish_sequence_on_priority(self) -> &'a mut crate::W<REG> {
        self.variant(Tprictrl::FinishSequenceOnPriority)
    }
}
#[doc = "Power Configuration Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pwrsel {
    #[doc = "0: Low power"]
    Lowest = 0,
    #[doc = "1: High power"]
    Highest = 1,
}
impl From<Pwrsel> for bool {
    #[inline(always)]
    fn from(variant: Pwrsel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PWRSEL` reader - Power Configuration Select"]
pub type PwrselR = crate::BitReader<Pwrsel>;
impl PwrselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pwrsel {
        match self.bits {
            false => Pwrsel::Lowest,
            true => Pwrsel::Highest,
        }
    }
    #[doc = "Low power"]
    #[inline(always)]
    pub fn is_lowest(&self) -> bool {
        *self == Pwrsel::Lowest
    }
    #[doc = "High power"]
    #[inline(always)]
    pub fn is_highest(&self) -> bool {
        *self == Pwrsel::Highest
    }
}
#[doc = "Field `PWRSEL` writer - Power Configuration Select"]
pub type PwrselW<'a, REG> = crate::BitWriter<'a, REG, Pwrsel>;
impl<'a, REG> PwrselW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Low power"]
    #[inline(always)]
    pub fn lowest(self) -> &'a mut crate::W<REG> {
        self.variant(Pwrsel::Lowest)
    }
    #[doc = "High power"]
    #[inline(always)]
    pub fn highest(self) -> &'a mut crate::W<REG> {
        self.variant(Pwrsel::Highest)
    }
}
#[doc = "Voltage Reference Selection\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Refsel {
    #[doc = "0: (Default) Option 1 setting."]
    Option1 = 0,
    #[doc = "1: Option 2 setting."]
    Option2 = 1,
    #[doc = "2: Option 3 setting."]
    Option3 = 2,
}
impl From<Refsel> for u8 {
    #[inline(always)]
    fn from(variant: Refsel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Refsel {
    type Ux = u8;
}
impl crate::IsEnum for Refsel {}
#[doc = "Field `REFSEL` reader - Voltage Reference Selection"]
pub type RefselR = crate::FieldReader<Refsel>;
impl RefselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Refsel> {
        match self.bits {
            0 => Some(Refsel::Option1),
            1 => Some(Refsel::Option2),
            2 => Some(Refsel::Option3),
            _ => None,
        }
    }
    #[doc = "(Default) Option 1 setting."]
    #[inline(always)]
    pub fn is_option_1(&self) -> bool {
        *self == Refsel::Option1
    }
    #[doc = "Option 2 setting."]
    #[inline(always)]
    pub fn is_option_2(&self) -> bool {
        *self == Refsel::Option2
    }
    #[doc = "Option 3 setting."]
    #[inline(always)]
    pub fn is_option_3(&self) -> bool {
        *self == Refsel::Option3
    }
}
#[doc = "Field `REFSEL` writer - Voltage Reference Selection"]
pub type RefselW<'a, REG> = crate::FieldWriter<'a, REG, 2, Refsel>;
impl<'a, REG> RefselW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "(Default) Option 1 setting."]
    #[inline(always)]
    pub fn option_1(self) -> &'a mut crate::W<REG> {
        self.variant(Refsel::Option1)
    }
    #[doc = "Option 2 setting."]
    #[inline(always)]
    pub fn option_2(self) -> &'a mut crate::W<REG> {
        self.variant(Refsel::Option2)
    }
    #[doc = "Option 3 setting."]
    #[inline(always)]
    pub fn option_3(self) -> &'a mut crate::W<REG> {
        self.variant(Refsel::Option3)
    }
}
#[doc = "Trigger Resume Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tres {
    #[doc = "0: Trigger sequences interrupted by a high priority trigger exception are not automatically resumed or restarted."]
    Disabled = 0,
    #[doc = "1: Trigger sequences interrupted by a high priority trigger exception are automatically resumed or restarted."]
    Enabled = 1,
}
impl From<Tres> for bool {
    #[inline(always)]
    fn from(variant: Tres) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TRES` reader - Trigger Resume Enable"]
pub type TresR = crate::BitReader<Tres>;
impl TresR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tres {
        match self.bits {
            false => Tres::Disabled,
            true => Tres::Enabled,
        }
    }
    #[doc = "Trigger sequences interrupted by a high priority trigger exception are not automatically resumed or restarted."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Tres::Disabled
    }
    #[doc = "Trigger sequences interrupted by a high priority trigger exception are automatically resumed or restarted."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Tres::Enabled
    }
}
#[doc = "Field `TRES` writer - Trigger Resume Enable"]
pub type TresW<'a, REG> = crate::BitWriter<'a, REG, Tres>;
impl<'a, REG> TresW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Trigger sequences interrupted by a high priority trigger exception are not automatically resumed or restarted."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tres::Disabled)
    }
    #[doc = "Trigger sequences interrupted by a high priority trigger exception are automatically resumed or restarted."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tres::Enabled)
    }
}
#[doc = "Trigger Command Resume\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tcmdres {
    #[doc = "0: Trigger sequences interrupted by a high priority trigger exception is automatically restarted."]
    Disabled = 0,
    #[doc = "1: Trigger sequences interrupted by a high priority trigger exception is resumed from the command executing before the exception."]
    Enabled = 1,
}
impl From<Tcmdres> for bool {
    #[inline(always)]
    fn from(variant: Tcmdres) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TCMDRES` reader - Trigger Command Resume"]
pub type TcmdresR = crate::BitReader<Tcmdres>;
impl TcmdresR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tcmdres {
        match self.bits {
            false => Tcmdres::Disabled,
            true => Tcmdres::Enabled,
        }
    }
    #[doc = "Trigger sequences interrupted by a high priority trigger exception is automatically restarted."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Tcmdres::Disabled
    }
    #[doc = "Trigger sequences interrupted by a high priority trigger exception is resumed from the command executing before the exception."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Tcmdres::Enabled
    }
}
#[doc = "Field `TCMDRES` writer - Trigger Command Resume"]
pub type TcmdresW<'a, REG> = crate::BitWriter<'a, REG, Tcmdres>;
impl<'a, REG> TcmdresW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Trigger sequences interrupted by a high priority trigger exception is automatically restarted."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tcmdres::Disabled)
    }
    #[doc = "Trigger sequences interrupted by a high priority trigger exception is resumed from the command executing before the exception."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tcmdres::Enabled)
    }
}
#[doc = "High Priority Trigger Exception Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HptExdi {
    #[doc = "0: High priority trigger exceptions are enabled."]
    Enabled = 0,
    #[doc = "1: High priority trigger exceptions are disabled."]
    Disabled = 1,
}
impl From<HptExdi> for bool {
    #[inline(always)]
    fn from(variant: HptExdi) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HPT_EXDI` reader - High Priority Trigger Exception Disable"]
pub type HptExdiR = crate::BitReader<HptExdi>;
impl HptExdiR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> HptExdi {
        match self.bits {
            false => HptExdi::Enabled,
            true => HptExdi::Disabled,
        }
    }
    #[doc = "High priority trigger exceptions are enabled."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == HptExdi::Enabled
    }
    #[doc = "High priority trigger exceptions are disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == HptExdi::Disabled
    }
}
#[doc = "Field `HPT_EXDI` writer - High Priority Trigger Exception Disable"]
pub type HptExdiW<'a, REG> = crate::BitWriter<'a, REG, HptExdi>;
impl<'a, REG> HptExdiW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "High priority trigger exceptions are enabled."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(HptExdi::Enabled)
    }
    #[doc = "High priority trigger exceptions are disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(HptExdi::Disabled)
    }
}
#[doc = "Field `PUDLY` reader - Power Up Delay"]
pub type PudlyR = crate::FieldReader;
#[doc = "Field `PUDLY` writer - Power Up Delay"]
pub type PudlyW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "ADC Analog Pre-Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pwren {
    #[doc = "0: ADC analog circuits are only enabled while conversions are active. Performance is affected due to analog startup delays."]
    NotPreEnabled = 0,
    #[doc = "1: ADC analog circuits are pre-enabled and ready to execute conversions without startup delays (at the cost of higher DC current consumption). Note that a single power up delay (CFG\\[PUDLY\\]) is executed immediately once PWREN is set, and any detected trigger does not begin ADC operation until the power up delay time has passed. After this initial delay expires the analog remains pre-enabled and no additional delays are executed."]
    PreEnabled = 1,
}
impl From<Pwren> for bool {
    #[inline(always)]
    fn from(variant: Pwren) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PWREN` reader - ADC Analog Pre-Enable"]
pub type PwrenR = crate::BitReader<Pwren>;
impl PwrenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pwren {
        match self.bits {
            false => Pwren::NotPreEnabled,
            true => Pwren::PreEnabled,
        }
    }
    #[doc = "ADC analog circuits are only enabled while conversions are active. Performance is affected due to analog startup delays."]
    #[inline(always)]
    pub fn is_not_pre_enabled(&self) -> bool {
        *self == Pwren::NotPreEnabled
    }
    #[doc = "ADC analog circuits are pre-enabled and ready to execute conversions without startup delays (at the cost of higher DC current consumption). Note that a single power up delay (CFG\\[PUDLY\\]) is executed immediately once PWREN is set, and any detected trigger does not begin ADC operation until the power up delay time has passed. After this initial delay expires the analog remains pre-enabled and no additional delays are executed."]
    #[inline(always)]
    pub fn is_pre_enabled(&self) -> bool {
        *self == Pwren::PreEnabled
    }
}
#[doc = "Field `PWREN` writer - ADC Analog Pre-Enable"]
pub type PwrenW<'a, REG> = crate::BitWriter<'a, REG, Pwren>;
impl<'a, REG> PwrenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "ADC analog circuits are only enabled while conversions are active. Performance is affected due to analog startup delays."]
    #[inline(always)]
    pub fn not_pre_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Pwren::NotPreEnabled)
    }
    #[doc = "ADC analog circuits are pre-enabled and ready to execute conversions without startup delays (at the cost of higher DC current consumption). Note that a single power up delay (CFG\\[PUDLY\\]) is executed immediately once PWREN is set, and any detected trigger does not begin ADC operation until the power up delay time has passed. After this initial delay expires the analog remains pre-enabled and no additional delays are executed."]
    #[inline(always)]
    pub fn pre_enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Pwren::PreEnabled)
    }
}
impl R {
    #[doc = "Bits 0:1 - ADC Trigger Priority Control"]
    #[inline(always)]
    pub fn tprictrl(&self) -> TprictrlR {
        TprictrlR::new((self.bits & 3) as u8)
    }
    #[doc = "Bit 5 - Power Configuration Select"]
    #[inline(always)]
    pub fn pwrsel(&self) -> PwrselR {
        PwrselR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bits 6:7 - Voltage Reference Selection"]
    #[inline(always)]
    pub fn refsel(&self) -> RefselR {
        RefselR::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bit 8 - Trigger Resume Enable"]
    #[inline(always)]
    pub fn tres(&self) -> TresR {
        TresR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Trigger Command Resume"]
    #[inline(always)]
    pub fn tcmdres(&self) -> TcmdresR {
        TcmdresR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - High Priority Trigger Exception Disable"]
    #[inline(always)]
    pub fn hpt_exdi(&self) -> HptExdiR {
        HptExdiR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bits 16:23 - Power Up Delay"]
    #[inline(always)]
    pub fn pudly(&self) -> PudlyR {
        PudlyR::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bit 28 - ADC Analog Pre-Enable"]
    #[inline(always)]
    pub fn pwren(&self) -> PwrenR {
        PwrenR::new(((self.bits >> 28) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:1 - ADC Trigger Priority Control"]
    #[inline(always)]
    pub fn tprictrl(&mut self) -> TprictrlW<CfgSpec> {
        TprictrlW::new(self, 0)
    }
    #[doc = "Bit 5 - Power Configuration Select"]
    #[inline(always)]
    pub fn pwrsel(&mut self) -> PwrselW<CfgSpec> {
        PwrselW::new(self, 5)
    }
    #[doc = "Bits 6:7 - Voltage Reference Selection"]
    #[inline(always)]
    pub fn refsel(&mut self) -> RefselW<CfgSpec> {
        RefselW::new(self, 6)
    }
    #[doc = "Bit 8 - Trigger Resume Enable"]
    #[inline(always)]
    pub fn tres(&mut self) -> TresW<CfgSpec> {
        TresW::new(self, 8)
    }
    #[doc = "Bit 9 - Trigger Command Resume"]
    #[inline(always)]
    pub fn tcmdres(&mut self) -> TcmdresW<CfgSpec> {
        TcmdresW::new(self, 9)
    }
    #[doc = "Bit 10 - High Priority Trigger Exception Disable"]
    #[inline(always)]
    pub fn hpt_exdi(&mut self) -> HptExdiW<CfgSpec> {
        HptExdiW::new(self, 10)
    }
    #[doc = "Bits 16:23 - Power Up Delay"]
    #[inline(always)]
    pub fn pudly(&mut self) -> PudlyW<CfgSpec> {
        PudlyW::new(self, 16)
    }
    #[doc = "Bit 28 - ADC Analog Pre-Enable"]
    #[inline(always)]
    pub fn pwren(&mut self) -> PwrenW<CfgSpec> {
        PwrenW::new(self, 28)
    }
}
#[doc = "Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`cfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CfgSpec;
impl crate::RegisterSpec for CfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cfg::R`](R) reader structure"]
impl crate::Readable for CfgSpec {}
#[doc = "`write(|w| ..)` method takes [`cfg::W`](W) writer structure"]
impl crate::Writable for CfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CFG to value 0x0080_0000"]
impl crate::Resettable for CfgSpec {
    const RESET_VALUE: u32 = 0x0080_0000;
}

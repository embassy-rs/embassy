#[doc = "Register `CTRLSTAT` reader"]
pub type R = crate::R<CtrlstatSpec>;
#[doc = "Register `CTRLSTAT` writer"]
pub type W = crate::W<CtrlstatSpec>;
#[doc = "Field `REF_SCALE` reader - Reference Scale"]
pub type RefScaleR = crate::FieldReader;
#[doc = "Pulse Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PulseMode {
    #[doc = "0: Frequency Measurement mode"]
    Freq = 0,
    #[doc = "1: Pulse Width Measurement mode"]
    Pulse = 1,
}
impl From<PulseMode> for bool {
    #[inline(always)]
    fn from(variant: PulseMode) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PULSE_MODE` reader - Pulse Mode"]
pub type PulseModeR = crate::BitReader<PulseMode>;
impl PulseModeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> PulseMode {
        match self.bits {
            false => PulseMode::Freq,
            true => PulseMode::Pulse,
        }
    }
    #[doc = "Frequency Measurement mode"]
    #[inline(always)]
    pub fn is_freq(&self) -> bool {
        *self == PulseMode::Freq
    }
    #[doc = "Pulse Width Measurement mode"]
    #[inline(always)]
    pub fn is_pulse(&self) -> bool {
        *self == PulseMode::Pulse
    }
}
#[doc = "Pulse Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PulsePol {
    #[doc = "0: High period"]
    High = 0,
    #[doc = "1: Low period"]
    Low = 1,
}
impl From<PulsePol> for bool {
    #[inline(always)]
    fn from(variant: PulsePol) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PULSE_POL` reader - Pulse Polarity"]
pub type PulsePolR = crate::BitReader<PulsePol>;
impl PulsePolR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> PulsePol {
        match self.bits {
            false => PulsePol::High,
            true => PulsePol::Low,
        }
    }
    #[doc = "High period"]
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        *self == PulsePol::High
    }
    #[doc = "Low period"]
    #[inline(always)]
    pub fn is_low(&self) -> bool {
        *self == PulsePol::Low
    }
}
#[doc = "Less Than Minimum Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LtMinIntEn {
    #[doc = "0: Disabled"]
    Disabled = 0,
    #[doc = "1: Enabled"]
    Enabled = 1,
}
impl From<LtMinIntEn> for bool {
    #[inline(always)]
    fn from(variant: LtMinIntEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LT_MIN_INT_EN` reader - Less Than Minimum Interrupt Enable"]
pub type LtMinIntEnR = crate::BitReader<LtMinIntEn>;
impl LtMinIntEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> LtMinIntEn {
        match self.bits {
            false => LtMinIntEn::Disabled,
            true => LtMinIntEn::Enabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == LtMinIntEn::Disabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == LtMinIntEn::Enabled
    }
}
#[doc = "Greater Than Maximum Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GtMaxIntEn {
    #[doc = "0: Disabled"]
    Disabled = 0,
    #[doc = "1: Enabled"]
    Enabled = 1,
}
impl From<GtMaxIntEn> for bool {
    #[inline(always)]
    fn from(variant: GtMaxIntEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GT_MAX_INT_EN` reader - Greater Than Maximum Interrupt Enable"]
pub type GtMaxIntEnR = crate::BitReader<GtMaxIntEn>;
impl GtMaxIntEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> GtMaxIntEn {
        match self.bits {
            false => GtMaxIntEn::Disabled,
            true => GtMaxIntEn::Enabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == GtMaxIntEn::Disabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == GtMaxIntEn::Enabled
    }
}
#[doc = "Result Ready Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultReadyIntEn {
    #[doc = "0: Disabled"]
    Disabled = 0,
    #[doc = "1: Enabled"]
    Enabled = 1,
}
impl From<ResultReadyIntEn> for bool {
    #[inline(always)]
    fn from(variant: ResultReadyIntEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RESULT_READY_INT_EN` reader - Result Ready Interrupt Enable"]
pub type ResultReadyIntEnR = crate::BitReader<ResultReadyIntEn>;
impl ResultReadyIntEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> ResultReadyIntEn {
        match self.bits {
            false => ResultReadyIntEn::Disabled,
            true => ResultReadyIntEn::Enabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == ResultReadyIntEn::Disabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == ResultReadyIntEn::Enabled
    }
}
#[doc = "Less Than Minimum Results Status\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LtMinStat {
    #[doc = "0: Greater than MIN\\[MIN_VALUE\\]"]
    InRange = 0,
    #[doc = "1: Less than MIN\\[MIN_VALUE\\]"]
    LtMin = 1,
}
impl From<LtMinStat> for bool {
    #[inline(always)]
    fn from(variant: LtMinStat) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LT_MIN_STAT` reader - Less Than Minimum Results Status"]
pub type LtMinStatR = crate::BitReader<LtMinStat>;
impl LtMinStatR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> LtMinStat {
        match self.bits {
            false => LtMinStat::InRange,
            true => LtMinStat::LtMin,
        }
    }
    #[doc = "Greater than MIN\\[MIN_VALUE\\]"]
    #[inline(always)]
    pub fn is_in_range(&self) -> bool {
        *self == LtMinStat::InRange
    }
    #[doc = "Less than MIN\\[MIN_VALUE\\]"]
    #[inline(always)]
    pub fn is_lt_min(&self) -> bool {
        *self == LtMinStat::LtMin
    }
}
#[doc = "Field `LT_MIN_STAT` writer - Less Than Minimum Results Status"]
pub type LtMinStatW<'a, REG> = crate::BitWriter1C<'a, REG, LtMinStat>;
impl<'a, REG> LtMinStatW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Greater than MIN\\[MIN_VALUE\\]"]
    #[inline(always)]
    pub fn in_range(self) -> &'a mut crate::W<REG> {
        self.variant(LtMinStat::InRange)
    }
    #[doc = "Less than MIN\\[MIN_VALUE\\]"]
    #[inline(always)]
    pub fn lt_min(self) -> &'a mut crate::W<REG> {
        self.variant(LtMinStat::LtMin)
    }
}
#[doc = "Greater Than Maximum Result Status\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GtMaxStat {
    #[doc = "0: Less than MAX\\[MAX_VALUE\\]"]
    InRange = 0,
    #[doc = "1: Greater than MAX\\[MAX_VALUE\\]"]
    GtMax = 1,
}
impl From<GtMaxStat> for bool {
    #[inline(always)]
    fn from(variant: GtMaxStat) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GT_MAX_STAT` reader - Greater Than Maximum Result Status"]
pub type GtMaxStatR = crate::BitReader<GtMaxStat>;
impl GtMaxStatR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> GtMaxStat {
        match self.bits {
            false => GtMaxStat::InRange,
            true => GtMaxStat::GtMax,
        }
    }
    #[doc = "Less than MAX\\[MAX_VALUE\\]"]
    #[inline(always)]
    pub fn is_in_range(&self) -> bool {
        *self == GtMaxStat::InRange
    }
    #[doc = "Greater than MAX\\[MAX_VALUE\\]"]
    #[inline(always)]
    pub fn is_gt_max(&self) -> bool {
        *self == GtMaxStat::GtMax
    }
}
#[doc = "Field `GT_MAX_STAT` writer - Greater Than Maximum Result Status"]
pub type GtMaxStatW<'a, REG> = crate::BitWriter1C<'a, REG, GtMaxStat>;
impl<'a, REG> GtMaxStatW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Less than MAX\\[MAX_VALUE\\]"]
    #[inline(always)]
    pub fn in_range(self) -> &'a mut crate::W<REG> {
        self.variant(GtMaxStat::InRange)
    }
    #[doc = "Greater than MAX\\[MAX_VALUE\\]"]
    #[inline(always)]
    pub fn gt_max(self) -> &'a mut crate::W<REG> {
        self.variant(GtMaxStat::GtMax)
    }
}
#[doc = "Result Ready Status\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultReadyStat {
    #[doc = "0: Not complete"]
    NotComplete = 0,
    #[doc = "1: Complete"]
    Complete = 1,
}
impl From<ResultReadyStat> for bool {
    #[inline(always)]
    fn from(variant: ResultReadyStat) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RESULT_READY_STAT` reader - Result Ready Status"]
pub type ResultReadyStatR = crate::BitReader<ResultReadyStat>;
impl ResultReadyStatR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> ResultReadyStat {
        match self.bits {
            false => ResultReadyStat::NotComplete,
            true => ResultReadyStat::Complete,
        }
    }
    #[doc = "Not complete"]
    #[inline(always)]
    pub fn is_not_complete(&self) -> bool {
        *self == ResultReadyStat::NotComplete
    }
    #[doc = "Complete"]
    #[inline(always)]
    pub fn is_complete(&self) -> bool {
        *self == ResultReadyStat::Complete
    }
}
#[doc = "Field `RESULT_READY_STAT` writer - Result Ready Status"]
pub type ResultReadyStatW<'a, REG> = crate::BitWriter1C<'a, REG, ResultReadyStat>;
impl<'a, REG> ResultReadyStatW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not complete"]
    #[inline(always)]
    pub fn not_complete(self) -> &'a mut crate::W<REG> {
        self.variant(ResultReadyStat::NotComplete)
    }
    #[doc = "Complete"]
    #[inline(always)]
    pub fn complete(self) -> &'a mut crate::W<REG> {
        self.variant(ResultReadyStat::Complete)
    }
}
#[doc = "Continuous Mode Enable Status\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContinuousModeEn {
    #[doc = "0: Disabled"]
    Disabled = 0,
    #[doc = "1: Enabled"]
    Enabled = 1,
}
impl From<ContinuousModeEn> for bool {
    #[inline(always)]
    fn from(variant: ContinuousModeEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CONTINUOUS_MODE_EN` reader - Continuous Mode Enable Status"]
pub type ContinuousModeEnR = crate::BitReader<ContinuousModeEn>;
impl ContinuousModeEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> ContinuousModeEn {
        match self.bits {
            false => ContinuousModeEn::Disabled,
            true => ContinuousModeEn::Enabled,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == ContinuousModeEn::Disabled
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == ContinuousModeEn::Enabled
    }
}
#[doc = "Measurement in Progress Status\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeasureInProgress {
    #[doc = "0: Not in progress"]
    Idle = 0,
    #[doc = "1: In progress"]
    Ongoing = 1,
}
impl From<MeasureInProgress> for bool {
    #[inline(always)]
    fn from(variant: MeasureInProgress) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MEASURE_IN_PROGRESS` reader - Measurement in Progress Status"]
pub type MeasureInProgressR = crate::BitReader<MeasureInProgress>;
impl MeasureInProgressR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> MeasureInProgress {
        match self.bits {
            false => MeasureInProgress::Idle,
            true => MeasureInProgress::Ongoing,
        }
    }
    #[doc = "Not in progress"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == MeasureInProgress::Idle
    }
    #[doc = "In progress"]
    #[inline(always)]
    pub fn is_ongoing(&self) -> bool {
        *self == MeasureInProgress::Ongoing
    }
}
impl R {
    #[doc = "Bits 0:4 - Reference Scale"]
    #[inline(always)]
    pub fn ref_scale(&self) -> RefScaleR {
        RefScaleR::new((self.bits & 0x1f) as u8)
    }
    #[doc = "Bit 8 - Pulse Mode"]
    #[inline(always)]
    pub fn pulse_mode(&self) -> PulseModeR {
        PulseModeR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Pulse Polarity"]
    #[inline(always)]
    pub fn pulse_pol(&self) -> PulsePolR {
        PulsePolR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 12 - Less Than Minimum Interrupt Enable"]
    #[inline(always)]
    pub fn lt_min_int_en(&self) -> LtMinIntEnR {
        LtMinIntEnR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Greater Than Maximum Interrupt Enable"]
    #[inline(always)]
    pub fn gt_max_int_en(&self) -> GtMaxIntEnR {
        GtMaxIntEnR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Result Ready Interrupt Enable"]
    #[inline(always)]
    pub fn result_ready_int_en(&self) -> ResultReadyIntEnR {
        ResultReadyIntEnR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 24 - Less Than Minimum Results Status"]
    #[inline(always)]
    pub fn lt_min_stat(&self) -> LtMinStatR {
        LtMinStatR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Greater Than Maximum Result Status"]
    #[inline(always)]
    pub fn gt_max_stat(&self) -> GtMaxStatR {
        GtMaxStatR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Result Ready Status"]
    #[inline(always)]
    pub fn result_ready_stat(&self) -> ResultReadyStatR {
        ResultReadyStatR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 30 - Continuous Mode Enable Status"]
    #[inline(always)]
    pub fn continuous_mode_en(&self) -> ContinuousModeEnR {
        ContinuousModeEnR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Measurement in Progress Status"]
    #[inline(always)]
    pub fn measure_in_progress(&self) -> MeasureInProgressR {
        MeasureInProgressR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 24 - Less Than Minimum Results Status"]
    #[inline(always)]
    pub fn lt_min_stat(&mut self) -> LtMinStatW<CtrlstatSpec> {
        LtMinStatW::new(self, 24)
    }
    #[doc = "Bit 25 - Greater Than Maximum Result Status"]
    #[inline(always)]
    pub fn gt_max_stat(&mut self) -> GtMaxStatW<CtrlstatSpec> {
        GtMaxStatW::new(self, 25)
    }
    #[doc = "Bit 26 - Result Ready Status"]
    #[inline(always)]
    pub fn result_ready_stat(&mut self) -> ResultReadyStatW<CtrlstatSpec> {
        ResultReadyStatW::new(self, 26)
    }
}
#[doc = "Control Status\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrlstat::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrlstat::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CtrlstatSpec;
impl crate::RegisterSpec for CtrlstatSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ctrlstat::R`](R) reader structure"]
impl crate::Readable for CtrlstatSpec {}
#[doc = "`write(|w| ..)` method takes [`ctrlstat::W`](W) writer structure"]
impl crate::Writable for CtrlstatSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0700_0000;
}
#[doc = "`reset()` method sets CTRLSTAT to value 0"]
impl crate::Resettable for CtrlstatSpec {}

#[doc = "Register `CTRL_R` reader"]
pub type R = crate::R<ReadModeCtrlRSpec>;
#[doc = "Field `RESULT` reader - Indicates the measurement result-either the target clock counter value (for Frequency Measurement mode) or pulse width measurement (for Pulse Width Measurement mode)"]
pub type ResultR = crate::FieldReader<u32>;
#[doc = "Measurement In Progress\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeasureInProgress {
    #[doc = "0: Complete"]
    CycleDone = 0,
    #[doc = "1: In progress"]
    InProgress = 1,
}
impl From<MeasureInProgress> for bool {
    #[inline(always)]
    fn from(variant: MeasureInProgress) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MEASURE_IN_PROGRESS` reader - Measurement In Progress"]
pub type MeasureInProgressR = crate::BitReader<MeasureInProgress>;
impl MeasureInProgressR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> MeasureInProgress {
        match self.bits {
            false => MeasureInProgress::CycleDone,
            true => MeasureInProgress::InProgress,
        }
    }
    #[doc = "Complete"]
    #[inline(always)]
    pub fn is_cycle_done(&self) -> bool {
        *self == MeasureInProgress::CycleDone
    }
    #[doc = "In progress"]
    #[inline(always)]
    pub fn is_in_progress(&self) -> bool {
        *self == MeasureInProgress::InProgress
    }
}
impl R {
    #[doc = "Bits 0:30 - Indicates the measurement result-either the target clock counter value (for Frequency Measurement mode) or pulse width measurement (for Pulse Width Measurement mode)"]
    #[inline(always)]
    pub fn result(&self) -> ResultR {
        ResultR::new(self.bits & 0x7fff_ffff)
    }
    #[doc = "Bit 31 - Measurement In Progress"]
    #[inline(always)]
    pub fn measure_in_progress(&self) -> MeasureInProgressR {
        MeasureInProgressR::new(((self.bits >> 31) & 1) != 0)
    }
}
#[doc = "Control (in Read mode)\n\nYou can [`read`](crate::Reg::read) this register and get [`read_mode_ctrl_r::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ReadModeCtrlRSpec;
impl crate::RegisterSpec for ReadModeCtrlRSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`read_mode_ctrl_r::R`](R) reader structure"]
impl crate::Readable for ReadModeCtrlRSpec {}
#[doc = "`reset()` method sets CTRL_R to value 0"]
impl crate::Resettable for ReadModeCtrlRSpec {}

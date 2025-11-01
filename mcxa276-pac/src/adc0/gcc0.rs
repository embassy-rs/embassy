#[doc = "Register `GCC0` reader"]
pub type R = crate::R<Gcc0Spec>;
#[doc = "Field `GAIN_CAL` reader - Gain Calibration Value"]
pub type GainCalR = crate::FieldReader<u16>;
#[doc = "Gain Calibration Value Valid\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdy {
    #[doc = "0: The GAIN_CAL value is invalid. Run the hardware calibration routine for this value to be set."]
    GainCalNotValid = 0,
    #[doc = "1: The GAIN_CAL value is valid. GAIN_CAL should be used by software to derive GCRa\\[GCALR\\]."]
    HardwareCalRoutineCompleted = 1,
}
impl From<Rdy> for bool {
    #[inline(always)]
    fn from(variant: Rdy) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RDY` reader - Gain Calibration Value Valid"]
pub type RdyR = crate::BitReader<Rdy>;
impl RdyR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rdy {
        match self.bits {
            false => Rdy::GainCalNotValid,
            true => Rdy::HardwareCalRoutineCompleted,
        }
    }
    #[doc = "The GAIN_CAL value is invalid. Run the hardware calibration routine for this value to be set."]
    #[inline(always)]
    pub fn is_gain_cal_not_valid(&self) -> bool {
        *self == Rdy::GainCalNotValid
    }
    #[doc = "The GAIN_CAL value is valid. GAIN_CAL should be used by software to derive GCRa\\[GCALR\\]."]
    #[inline(always)]
    pub fn is_hardware_cal_routine_completed(&self) -> bool {
        *self == Rdy::HardwareCalRoutineCompleted
    }
}
impl R {
    #[doc = "Bits 0:15 - Gain Calibration Value"]
    #[inline(always)]
    pub fn gain_cal(&self) -> GainCalR {
        GainCalR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bit 24 - Gain Calibration Value Valid"]
    #[inline(always)]
    pub fn rdy(&self) -> RdyR {
        RdyR::new(((self.bits >> 24) & 1) != 0)
    }
}
#[doc = "Gain Calibration Control\n\nYou can [`read`](crate::Reg::read) this register and get [`gcc0::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Gcc0Spec;
impl crate::RegisterSpec for Gcc0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`gcc0::R`](R) reader structure"]
impl crate::Readable for Gcc0Spec {}
#[doc = "`reset()` method sets GCC0 to value 0"]
impl crate::Resettable for Gcc0Spec {}

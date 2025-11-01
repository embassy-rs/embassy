#[doc = "Register `SPLLSSCGSTAT` reader"]
pub type R = crate::R<SpllsscgstatSpec>;
#[doc = "SS_MDIV change acknowledge\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SsMdivAck {
    #[doc = "0: The SS_MDIV, MF, MR, MC ratio change is not accepted by the analog PLL"]
    Disabled = 0,
    #[doc = "1: The SS_MDIV, MF, MR, MC ratio change is accepted by the analog PLL"]
    Enabled = 1,
}
impl From<SsMdivAck> for bool {
    #[inline(always)]
    fn from(variant: SsMdivAck) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SS_MDIV_ACK` reader - SS_MDIV change acknowledge"]
pub type SsMdivAckR = crate::BitReader<SsMdivAck>;
impl SsMdivAckR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SsMdivAck {
        match self.bits {
            false => SsMdivAck::Disabled,
            true => SsMdivAck::Enabled,
        }
    }
    #[doc = "The SS_MDIV, MF, MR, MC ratio change is not accepted by the analog PLL"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == SsMdivAck::Disabled
    }
    #[doc = "The SS_MDIV, MF, MR, MC ratio change is accepted by the analog PLL"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == SsMdivAck::Enabled
    }
}
impl R {
    #[doc = "Bit 0 - SS_MDIV change acknowledge"]
    #[inline(always)]
    pub fn ss_mdiv_ack(&self) -> SsMdivAckR {
        SsMdivAckR::new((self.bits & 1) != 0)
    }
}
#[doc = "SPLL SSCG Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`spllsscgstat::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SpllsscgstatSpec;
impl crate::RegisterSpec for SpllsscgstatSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`spllsscgstat::R`](R) reader structure"]
impl crate::Readable for SpllsscgstatSpec {}
#[doc = "`reset()` method sets SPLLSSCGSTAT to value 0"]
impl crate::Resettable for SpllsscgstatSpec {}

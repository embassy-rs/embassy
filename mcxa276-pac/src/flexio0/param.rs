#[doc = "Register `PARAM` reader"]
pub type R = crate::R<ParamSpec>;
#[doc = "Field `SHIFTER` reader - Shifter Number"]
pub type ShifterR = crate::FieldReader;
#[doc = "Field `TIMER` reader - Timer Number"]
pub type TimerR = crate::FieldReader;
#[doc = "Field `PIN` reader - Pin Number"]
pub type PinR = crate::FieldReader;
#[doc = "Field `TRIGGER` reader - Trigger Number"]
pub type TriggerR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:7 - Shifter Number"]
    #[inline(always)]
    pub fn shifter(&self) -> ShifterR {
        ShifterR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Timer Number"]
    #[inline(always)]
    pub fn timer(&self) -> TimerR {
        TimerR::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Pin Number"]
    #[inline(always)]
    pub fn pin(&self) -> PinR {
        PinR::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Trigger Number"]
    #[inline(always)]
    pub fn trigger(&self) -> TriggerR {
        TriggerR::new(((self.bits >> 24) & 0xff) as u8)
    }
}
#[doc = "Parameter\n\nYou can [`read`](crate::Reg::read) this register and get [`param::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ParamSpec;
impl crate::RegisterSpec for ParamSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`param::R`](R) reader structure"]
impl crate::Readable for ParamSpec {}
#[doc = "`reset()` method sets PARAM to value 0x0420_0404"]
impl crate::Resettable for ParamSpec {
    const RESET_VALUE: u32 = 0x0420_0404;
}

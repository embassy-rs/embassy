#[doc = "Register `OSC2_FRQCNT` reader"]
pub type R = crate::R<Osc2FrqcntOsc2FrqcntSpec>;
#[doc = "Field `OSC2_FRQ_CT` reader - Frequency Count"]
pub type Osc2FrqCtR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:21 - Frequency Count"]
    #[inline(always)]
    pub fn osc2_frq_ct(&self) -> Osc2FrqCtR {
        Osc2FrqCtR::new(self.bits & 0x003f_ffff)
    }
}
#[doc = "Oscillator-2 Frequency Count Register\n\nYou can [`read`](crate::Reg::read) this register and get [`osc2_frqcnt_osc2_frqcnt::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Osc2FrqcntOsc2FrqcntSpec;
impl crate::RegisterSpec for Osc2FrqcntOsc2FrqcntSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`osc2_frqcnt_osc2_frqcnt::R`](R) reader structure"]
impl crate::Readable for Osc2FrqcntOsc2FrqcntSpec {}
#[doc = "`reset()` method sets OSC2_FRQCNT to value 0"]
impl crate::Resettable for Osc2FrqcntOsc2FrqcntSpec {}

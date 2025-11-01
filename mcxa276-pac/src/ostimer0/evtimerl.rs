#[doc = "Register `EVTIMERL` reader"]
pub type R = crate::R<EvtimerlSpec>;
#[doc = "Field `EVTIMER_COUNT_VALUE` reader - EVTimer Count Value"]
pub type EvtimerCountValueR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - EVTimer Count Value"]
    #[inline(always)]
    pub fn evtimer_count_value(&self) -> EvtimerCountValueR {
        EvtimerCountValueR::new(self.bits)
    }
}
#[doc = "EVTIMER Low\n\nYou can [`read`](crate::Reg::read) this register and get [`evtimerl::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct EvtimerlSpec;
impl crate::RegisterSpec for EvtimerlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`evtimerl::R`](R) reader structure"]
impl crate::Readable for EvtimerlSpec {}
#[doc = "`reset()` method sets EVTIMERL to value 0"]
impl crate::Resettable for EvtimerlSpec {}

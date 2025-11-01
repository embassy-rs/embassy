#[doc = "Register `INSTRUCTION_TIMER` reader"]
pub type R = crate::R<InstructionTimerSpec>;
#[doc = "Field `INSTIM` reader - Current value of the Instruction Timer"]
pub type InstimR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - Current value of the Instruction Timer"]
    #[inline(always)]
    pub fn instim(&self) -> InstimR {
        InstimR::new(self.bits)
    }
}
#[doc = "Instruction Timer Register\n\nYou can [`read`](crate::Reg::read) this register and get [`instruction_timer::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct InstructionTimerSpec;
impl crate::RegisterSpec for InstructionTimerSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`instruction_timer::R`](R) reader structure"]
impl crate::Readable for InstructionTimerSpec {}
#[doc = "`reset()` method sets INSTRUCTION_TIMER to value 0xffff_ffff"]
impl crate::Resettable for InstructionTimerSpec {
    const RESET_VALUE: u32 = 0xffff_ffff;
}

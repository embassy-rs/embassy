#[doc = "Register `PKC_INT_STATUS` reader"]
pub type R = crate::R<PkcIntStatusSpec>;
#[doc = "Field `INT_PDONE` reader - End-of-computation status flag"]
pub type IntPdoneR = crate::BitReader;
impl R {
    #[doc = "Bit 0 - End-of-computation status flag"]
    #[inline(always)]
    pub fn int_pdone(&self) -> IntPdoneR {
        IntPdoneR::new((self.bits & 1) != 0)
    }
}
#[doc = "Interrupt status\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_int_status::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcIntStatusSpec;
impl crate::RegisterSpec for PkcIntStatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_int_status::R`](R) reader structure"]
impl crate::Readable for PkcIntStatusSpec {}
#[doc = "`reset()` method sets PKC_INT_STATUS to value 0"]
impl crate::Resettable for PkcIntStatusSpec {}

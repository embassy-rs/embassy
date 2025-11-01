#[doc = "Register `ROP_STATE` reader"]
pub type R = crate::R<RopStateSpec>;
#[doc = "Field `ROP_STATE` reader - ROP state"]
pub type RopStateR = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - ROP state"]
    #[inline(always)]
    pub fn rop_state(&self) -> RopStateR {
        RopStateR::new(self.bits)
    }
}
#[doc = "ROP State Register\n\nYou can [`read`](crate::Reg::read) this register and get [`rop_state::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RopStateSpec;
impl crate::RegisterSpec for RopStateSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rop_state::R`](R) reader structure"]
impl crate::Readable for RopStateSpec {}
#[doc = "`reset()` method sets ROP_STATE to value 0"]
impl crate::Resettable for RopStateSpec {}

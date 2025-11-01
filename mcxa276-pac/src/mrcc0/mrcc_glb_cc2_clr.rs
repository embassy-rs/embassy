#[doc = "Register `MRCC_GLB_CC2_CLR` writer"]
pub type W = crate::W<MrccGlbCc2ClrSpec>;
#[doc = "Field `DATA` writer - Data array value, refer to corresponding position in MRCC_GLB_CCn."]
pub type DataW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - Data array value, refer to corresponding position in MRCC_GLB_CCn."]
    #[inline(always)]
    pub fn data(&mut self) -> DataW<MrccGlbCc2ClrSpec> {
        DataW::new(self, 0)
    }
}
#[doc = "AHB Clock Control Clear 2\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_cc2_clr::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrccGlbCc2ClrSpec;
impl crate::RegisterSpec for MrccGlbCc2ClrSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`mrcc_glb_cc2_clr::W`](W) writer structure"]
impl crate::Writable for MrccGlbCc2ClrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MRCC_GLB_CC2_CLR to value 0"]
impl crate::Resettable for MrccGlbCc2ClrSpec {}

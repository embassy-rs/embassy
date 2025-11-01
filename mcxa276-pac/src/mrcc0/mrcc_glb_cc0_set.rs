#[doc = "Register `MRCC_GLB_CC0_SET` writer"]
pub type W = crate::W<MrccGlbCc0SetSpec>;
#[doc = "Field `DATA` writer - Data array value, refer to corresponding position in MRCC_GLB_CCn."]
pub type DataW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - Data array value, refer to corresponding position in MRCC_GLB_CCn."]
    #[inline(always)]
    pub fn data(&mut self) -> DataW<MrccGlbCc0SetSpec> {
        DataW::new(self, 0)
    }
}
#[doc = "AHB Clock Control Set 0\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_cc0_set::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrccGlbCc0SetSpec;
impl crate::RegisterSpec for MrccGlbCc0SetSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`mrcc_glb_cc0_set::W`](W) writer structure"]
impl crate::Writable for MrccGlbCc0SetSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MRCC_GLB_CC0_SET to value 0x0001_0000"]
impl crate::Resettable for MrccGlbCc0SetSpec {
    const RESET_VALUE: u32 = 0x0001_0000;
}

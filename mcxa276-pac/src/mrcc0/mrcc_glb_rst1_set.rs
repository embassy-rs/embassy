#[doc = "Register `MRCC_GLB_RST1_SET` writer"]
pub type W = crate::W<MrccGlbRst1SetSpec>;
#[doc = "Field `DATA` writer - Data array value, refer to corresponding position in MRCC_GLB_RSTn."]
pub type DataW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - Data array value, refer to corresponding position in MRCC_GLB_RSTn."]
    #[inline(always)]
    pub fn data(&mut self) -> DataW<MrccGlbRst1SetSpec> {
        DataW::new(self, 0)
    }
}
#[doc = "Peripheral Reset Control Set 1\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_rst1_set::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrccGlbRst1SetSpec;
impl crate::RegisterSpec for MrccGlbRst1SetSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`mrcc_glb_rst1_set::W`](W) writer structure"]
impl crate::Writable for MrccGlbRst1SetSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MRCC_GLB_RST1_SET to value 0"]
impl crate::Resettable for MrccGlbRst1SetSpec {}

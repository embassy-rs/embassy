#[doc = "Register `MRCC_GLB_RST2_CLR` writer"]
pub type W = crate::W<MrccGlbRst2ClrSpec>;
#[doc = "Field `DATA` writer - Data array value, refer to corresponding position in MRCC_GLB_RSTn."]
pub type DataW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl W {
    #[doc = "Bits 0:31 - Data array value, refer to corresponding position in MRCC_GLB_RSTn."]
    #[inline(always)]
    pub fn data(&mut self) -> DataW<MrccGlbRst2ClrSpec> {
        DataW::new(self, 0)
    }
}
#[doc = "Peripheral Reset Control Clear 2\n\nYou can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_glb_rst2_clr::W`](W). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrccGlbRst2ClrSpec;
impl crate::RegisterSpec for MrccGlbRst2ClrSpec {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [`mrcc_glb_rst2_clr::W`](W) writer structure"]
impl crate::Writable for MrccGlbRst2ClrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MRCC_GLB_RST2_CLR to value 0"]
impl crate::Resettable for MrccGlbRst2ClrSpec {}

#[doc = "Register `MBC0_MEM0_GLBCFG` reader"]
pub type R = crate::R<Mbc0Mem0GlbcfgSpec>;
#[doc = "Field `NBLKS` reader - Number of blocks in this memory"]
pub type NblksR = crate::FieldReader<u16>;
#[doc = "Field `SIZE_LOG2` reader - Log2 size per block"]
pub type SizeLog2R = crate::FieldReader;
impl R {
    #[doc = "Bits 0:9 - Number of blocks in this memory"]
    #[inline(always)]
    pub fn nblks(&self) -> NblksR {
        NblksR::new((self.bits & 0x03ff) as u16)
    }
    #[doc = "Bits 16:20 - Log2 size per block"]
    #[inline(always)]
    pub fn size_log2(&self) -> SizeLog2R {
        SizeLog2R::new(((self.bits >> 16) & 0x1f) as u8)
    }
}
#[doc = "MBC Global Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_mem0_glbcfg::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mbc0Mem0GlbcfgSpec;
impl crate::RegisterSpec for Mbc0Mem0GlbcfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mbc0_mem0_glbcfg::R`](R) reader structure"]
impl crate::Readable for Mbc0Mem0GlbcfgSpec {}
#[doc = "`reset()` method sets MBC0_MEM0_GLBCFG to value 0x000d_0080"]
impl crate::Resettable for Mbc0Mem0GlbcfgSpec {
    const RESET_VALUE: u32 = 0x000d_0080;
}

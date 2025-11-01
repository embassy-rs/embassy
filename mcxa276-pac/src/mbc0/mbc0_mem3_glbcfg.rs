#[doc = "Register `MBC0_MEM3_GLBCFG` reader"]
pub type R = crate::R<Mbc0Mem3GlbcfgSpec>;
#[doc = "Register `MBC0_MEM3_GLBCFG` writer"]
pub type W = crate::W<Mbc0Mem3GlbcfgSpec>;
#[doc = "Field `NBLKS` reader - Number of blocks in this memory"]
pub type NblksR = crate::FieldReader<u16>;
#[doc = "Field `SIZE_LOG2` reader - Log2 size per block"]
pub type SizeLog2R = crate::FieldReader;
#[doc = "Field `CLRE` reader - Clear Error"]
pub type ClreR = crate::FieldReader;
#[doc = "Field `CLRE` writer - Clear Error"]
pub type ClreW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
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
    #[doc = "Bits 30:31 - Clear Error"]
    #[inline(always)]
    pub fn clre(&self) -> ClreR {
        ClreR::new(((self.bits >> 30) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 30:31 - Clear Error"]
    #[inline(always)]
    pub fn clre(&mut self) -> ClreW<Mbc0Mem3GlbcfgSpec> {
        ClreW::new(self, 30)
    }
}
#[doc = "MBC Global Configuration Register\n\nYou can [`read`](crate::Reg::read) this register and get [`mbc0_mem3_glbcfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mbc0_mem3_glbcfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Mbc0Mem3GlbcfgSpec;
impl crate::RegisterSpec for Mbc0Mem3GlbcfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mbc0_mem3_glbcfg::R`](R) reader structure"]
impl crate::Readable for Mbc0Mem3GlbcfgSpec {}
#[doc = "`write(|w| ..)` method takes [`mbc0_mem3_glbcfg::W`](W) writer structure"]
impl crate::Writable for Mbc0Mem3GlbcfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MBC0_MEM3_GLBCFG to value 0"]
impl crate::Resettable for Mbc0Mem3GlbcfgSpec {}

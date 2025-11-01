#[doc = "Register `SBLIM` reader"]
pub type R = crate::R<SblimSblimSpec>;
#[doc = "Register `SBLIM` writer"]
pub type W = crate::W<SblimSblimSpec>;
#[doc = "Field `SB_LIM` reader - Sparse Bit Limit"]
pub type SbLimR = crate::FieldReader<u16>;
#[doc = "Field `SB_LIM` writer - Sparse Bit Limit"]
pub type SbLimW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
impl R {
    #[doc = "Bits 0:9 - Sparse Bit Limit"]
    #[inline(always)]
    pub fn sb_lim(&self) -> SbLimR {
        SbLimR::new((self.bits & 0x03ff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:9 - Sparse Bit Limit"]
    #[inline(always)]
    pub fn sb_lim(&mut self) -> SbLimW<SblimSblimSpec> {
        SbLimW::new(self, 0)
    }
}
#[doc = "Sparse Bit Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sblim_sblim::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sblim_sblim::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SblimSblimSpec;
impl crate::RegisterSpec for SblimSblimSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sblim_sblim::R`](R) reader structure"]
impl crate::Readable for SblimSblimSpec {}
#[doc = "`write(|w| ..)` method takes [`sblim_sblim::W`](W) writer structure"]
impl crate::Writable for SblimSblimSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SBLIM to value 0x3f"]
impl crate::Resettable for SblimSblimSpec {
    const RESET_VALUE: u32 = 0x3f;
}

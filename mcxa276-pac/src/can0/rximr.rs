#[doc = "Register `RXIMR[%s]` reader"]
pub type R = crate::R<RximrSpec>;
#[doc = "Register `RXIMR[%s]` writer"]
pub type W = crate::W<RximrSpec>;
#[doc = "Field `MI` reader - Individual Mask Bits"]
pub type MiR = crate::FieldReader<u32>;
#[doc = "Field `MI` writer - Individual Mask Bits"]
pub type MiW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Individual Mask Bits"]
    #[inline(always)]
    pub fn mi(&self) -> MiR {
        MiR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Individual Mask Bits"]
    #[inline(always)]
    pub fn mi(&mut self) -> MiW<RximrSpec> {
        MiW::new(self, 0)
    }
}
#[doc = "Receive Individual Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`rximr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rximr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RximrSpec;
impl crate::RegisterSpec for RximrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rximr::R`](R) reader structure"]
impl crate::Readable for RximrSpec {}
#[doc = "`write(|w| ..)` method takes [`rximr::W`](W) writer structure"]
impl crate::Writable for RximrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RXIMR[%s] to value 0"]
impl crate::Resettable for RximrSpec {}

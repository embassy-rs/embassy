#[doc = "Register `sgi_sha_fifo` reader"]
pub type R = crate::R<SgiShaFifoSpec>;
#[doc = "Register `sgi_sha_fifo` writer"]
pub type W = crate::W<SgiShaFifoSpec>;
#[doc = "Field `fifo` reader - SHA FIFO register"]
pub type FifoR = crate::FieldReader<u32>;
#[doc = "Field `fifo` writer - SHA FIFO register"]
pub type FifoW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - SHA FIFO register"]
    #[inline(always)]
    pub fn fifo(&self) -> FifoR {
        FifoR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - SHA FIFO register"]
    #[inline(always)]
    pub fn fifo(&mut self) -> FifoW<SgiShaFifoSpec> {
        FifoW::new(self, 0)
    }
}
#[doc = "SHA FIFO lower-bank low\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_sha_fifo::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_sha_fifo::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiShaFifoSpec;
impl crate::RegisterSpec for SgiShaFifoSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_sha_fifo::R`](R) reader structure"]
impl crate::Readable for SgiShaFifoSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_sha_fifo::W`](W) writer structure"]
impl crate::Writable for SgiShaFifoSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_sha_fifo to value 0"]
impl crate::Resettable for SgiShaFifoSpec {}

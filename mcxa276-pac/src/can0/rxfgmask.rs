#[doc = "Register `RXFGMASK` reader"]
pub type R = crate::R<RxfgmaskSpec>;
#[doc = "Register `RXFGMASK` writer"]
pub type W = crate::W<RxfgmaskSpec>;
#[doc = "Field `FGM` reader - Legacy RX FIFO Global Mask Bits"]
pub type FgmR = crate::FieldReader<u32>;
#[doc = "Field `FGM` writer - Legacy RX FIFO Global Mask Bits"]
pub type FgmW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Legacy RX FIFO Global Mask Bits"]
    #[inline(always)]
    pub fn fgm(&self) -> FgmR {
        FgmR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Legacy RX FIFO Global Mask Bits"]
    #[inline(always)]
    pub fn fgm(&mut self) -> FgmW<RxfgmaskSpec> {
        FgmW::new(self, 0)
    }
}
#[doc = "Legacy RX FIFO Global Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`rxfgmask::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rxfgmask::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RxfgmaskSpec;
impl crate::RegisterSpec for RxfgmaskSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rxfgmask::R`](R) reader structure"]
impl crate::Readable for RxfgmaskSpec {}
#[doc = "`write(|w| ..)` method takes [`rxfgmask::W`](W) writer structure"]
impl crate::Writable for RxfgmaskSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RXFGMASK to value 0"]
impl crate::Resettable for RxfgmaskSpec {}

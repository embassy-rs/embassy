#[doc = "Register `MFCR` reader"]
pub type R = crate::R<MfcrSpec>;
#[doc = "Register `MFCR` writer"]
pub type W = crate::W<MfcrSpec>;
#[doc = "Field `TXWATER` reader - Transmit FIFO Watermark"]
pub type TxwaterR = crate::FieldReader;
#[doc = "Field `TXWATER` writer - Transmit FIFO Watermark"]
pub type TxwaterW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `RXWATER` reader - Receive FIFO Watermark"]
pub type RxwaterR = crate::FieldReader;
#[doc = "Field `RXWATER` writer - Receive FIFO Watermark"]
pub type RxwaterW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
impl R {
    #[doc = "Bits 0:1 - Transmit FIFO Watermark"]
    #[inline(always)]
    pub fn txwater(&self) -> TxwaterR {
        TxwaterR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 16:17 - Receive FIFO Watermark"]
    #[inline(always)]
    pub fn rxwater(&self) -> RxwaterR {
        RxwaterR::new(((self.bits >> 16) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Transmit FIFO Watermark"]
    #[inline(always)]
    pub fn txwater(&mut self) -> TxwaterW<MfcrSpec> {
        TxwaterW::new(self, 0)
    }
    #[doc = "Bits 16:17 - Receive FIFO Watermark"]
    #[inline(always)]
    pub fn rxwater(&mut self) -> RxwaterW<MfcrSpec> {
        RxwaterW::new(self, 16)
    }
}
#[doc = "Controller FIFO Control\n\nYou can [`read`](crate::Reg::read) this register and get [`mfcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mfcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MfcrSpec;
impl crate::RegisterSpec for MfcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mfcr::R`](R) reader structure"]
impl crate::Readable for MfcrSpec {}
#[doc = "`write(|w| ..)` method takes [`mfcr::W`](W) writer structure"]
impl crate::Writable for MfcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MFCR to value 0"]
impl crate::Resettable for MfcrSpec {}

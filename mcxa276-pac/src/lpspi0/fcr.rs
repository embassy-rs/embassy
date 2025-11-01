#[doc = "Register `FCR` reader"]
pub type R = crate::R<FcrSpec>;
#[doc = "Register `FCR` writer"]
pub type W = crate::W<FcrSpec>;
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
    pub fn txwater(&mut self) -> TxwaterW<FcrSpec> {
        TxwaterW::new(self, 0)
    }
    #[doc = "Bits 16:17 - Receive FIFO Watermark"]
    #[inline(always)]
    pub fn rxwater(&mut self) -> RxwaterW<FcrSpec> {
        RxwaterW::new(self, 16)
    }
}
#[doc = "FIFO Control\n\nYou can [`read`](crate::Reg::read) this register and get [`fcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FcrSpec;
impl crate::RegisterSpec for FcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fcr::R`](R) reader structure"]
impl crate::Readable for FcrSpec {}
#[doc = "`write(|w| ..)` method takes [`fcr::W`](W) writer structure"]
impl crate::Writable for FcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FCR to value 0"]
impl crate::Resettable for FcrSpec {}

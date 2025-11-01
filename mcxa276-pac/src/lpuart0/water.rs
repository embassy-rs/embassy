#[doc = "Register `WATER` reader"]
pub type R = crate::R<WaterSpec>;
#[doc = "Register `WATER` writer"]
pub type W = crate::W<WaterSpec>;
#[doc = "Field `TXWATER` reader - Transmit Watermark"]
pub type TxwaterR = crate::FieldReader;
#[doc = "Field `TXWATER` writer - Transmit Watermark"]
pub type TxwaterW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `TXCOUNT` reader - Transmit Counter"]
pub type TxcountR = crate::FieldReader;
#[doc = "Field `RXWATER` reader - Receive Watermark"]
pub type RxwaterR = crate::FieldReader;
#[doc = "Field `RXWATER` writer - Receive Watermark"]
pub type RxwaterW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `RXCOUNT` reader - Receive Counter"]
pub type RxcountR = crate::FieldReader;
impl R {
    #[doc = "Bits 0:1 - Transmit Watermark"]
    #[inline(always)]
    pub fn txwater(&self) -> TxwaterR {
        TxwaterR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 8:10 - Transmit Counter"]
    #[inline(always)]
    pub fn txcount(&self) -> TxcountR {
        TxcountR::new(((self.bits >> 8) & 7) as u8)
    }
    #[doc = "Bits 16:17 - Receive Watermark"]
    #[inline(always)]
    pub fn rxwater(&self) -> RxwaterR {
        RxwaterR::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bits 24:26 - Receive Counter"]
    #[inline(always)]
    pub fn rxcount(&self) -> RxcountR {
        RxcountR::new(((self.bits >> 24) & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Transmit Watermark"]
    #[inline(always)]
    pub fn txwater(&mut self) -> TxwaterW<WaterSpec> {
        TxwaterW::new(self, 0)
    }
    #[doc = "Bits 16:17 - Receive Watermark"]
    #[inline(always)]
    pub fn rxwater(&mut self) -> RxwaterW<WaterSpec> {
        RxwaterW::new(self, 16)
    }
}
#[doc = "Watermark\n\nYou can [`read`](crate::Reg::read) this register and get [`water::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`water::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct WaterSpec;
impl crate::RegisterSpec for WaterSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`water::R`](R) reader structure"]
impl crate::Readable for WaterSpec {}
#[doc = "`write(|w| ..)` method takes [`water::W`](W) writer structure"]
impl crate::Writable for WaterSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets WATER to value 0"]
impl crate::Resettable for WaterSpec {}

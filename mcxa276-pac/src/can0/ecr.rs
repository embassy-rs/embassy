#[doc = "Register `ECR` reader"]
pub type R = crate::R<EcrSpec>;
#[doc = "Register `ECR` writer"]
pub type W = crate::W<EcrSpec>;
#[doc = "Field `TXERRCNT` reader - Transmit Error Counter"]
pub type TxerrcntR = crate::FieldReader;
#[doc = "Field `TXERRCNT` writer - Transmit Error Counter"]
pub type TxerrcntW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `RXERRCNT` reader - Receive Error Counter"]
pub type RxerrcntR = crate::FieldReader;
#[doc = "Field `RXERRCNT` writer - Receive Error Counter"]
pub type RxerrcntW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `TXERRCNT_FAST` reader - Transmit Error Counter for Fast Bits"]
pub type TxerrcntFastR = crate::FieldReader;
#[doc = "Field `TXERRCNT_FAST` writer - Transmit Error Counter for Fast Bits"]
pub type TxerrcntFastW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `RXERRCNT_FAST` reader - Receive Error Counter for Fast Bits"]
pub type RxerrcntFastR = crate::FieldReader;
#[doc = "Field `RXERRCNT_FAST` writer - Receive Error Counter for Fast Bits"]
pub type RxerrcntFastW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:7 - Transmit Error Counter"]
    #[inline(always)]
    pub fn txerrcnt(&self) -> TxerrcntR {
        TxerrcntR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - Receive Error Counter"]
    #[inline(always)]
    pub fn rxerrcnt(&self) -> RxerrcntR {
        RxerrcntR::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - Transmit Error Counter for Fast Bits"]
    #[inline(always)]
    pub fn txerrcnt_fast(&self) -> TxerrcntFastR {
        TxerrcntFastR::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 24:31 - Receive Error Counter for Fast Bits"]
    #[inline(always)]
    pub fn rxerrcnt_fast(&self) -> RxerrcntFastR {
        RxerrcntFastR::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Transmit Error Counter"]
    #[inline(always)]
    pub fn txerrcnt(&mut self) -> TxerrcntW<EcrSpec> {
        TxerrcntW::new(self, 0)
    }
    #[doc = "Bits 8:15 - Receive Error Counter"]
    #[inline(always)]
    pub fn rxerrcnt(&mut self) -> RxerrcntW<EcrSpec> {
        RxerrcntW::new(self, 8)
    }
    #[doc = "Bits 16:23 - Transmit Error Counter for Fast Bits"]
    #[inline(always)]
    pub fn txerrcnt_fast(&mut self) -> TxerrcntFastW<EcrSpec> {
        TxerrcntFastW::new(self, 16)
    }
    #[doc = "Bits 24:31 - Receive Error Counter for Fast Bits"]
    #[inline(always)]
    pub fn rxerrcnt_fast(&mut self) -> RxerrcntFastW<EcrSpec> {
        RxerrcntFastW::new(self, 24)
    }
}
#[doc = "Error Counter\n\nYou can [`read`](crate::Reg::read) this register and get [`ecr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ecr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct EcrSpec;
impl crate::RegisterSpec for EcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ecr::R`](R) reader structure"]
impl crate::Readable for EcrSpec {}
#[doc = "`write(|w| ..)` method takes [`ecr::W`](W) writer structure"]
impl crate::Writable for EcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ECR to value 0"]
impl crate::Resettable for EcrSpec {}

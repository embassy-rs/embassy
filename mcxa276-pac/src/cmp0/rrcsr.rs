#[doc = "Register `RRCSR` reader"]
pub type R = crate::R<RrcsrSpec>;
#[doc = "Register `RRCSR` writer"]
pub type W = crate::W<RrcsrSpec>;
#[doc = "Field `RR_CH0OUT` reader - Comparison Result for Channel 0"]
pub type RrCh0outR = crate::BitReader;
#[doc = "Field `RR_CH0OUT` writer - Comparison Result for Channel 0"]
pub type RrCh0outW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RR_CH1OUT` reader - Comparison Result for Channel 1"]
pub type RrCh1outR = crate::BitReader;
#[doc = "Field `RR_CH1OUT` writer - Comparison Result for Channel 1"]
pub type RrCh1outW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RR_CH2OUT` reader - Comparison Result for Channel 2"]
pub type RrCh2outR = crate::BitReader;
#[doc = "Field `RR_CH2OUT` writer - Comparison Result for Channel 2"]
pub type RrCh2outW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RR_CH3OUT` reader - Comparison Result for Channel 3"]
pub type RrCh3outR = crate::BitReader;
#[doc = "Field `RR_CH3OUT` writer - Comparison Result for Channel 3"]
pub type RrCh3outW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RR_CH4OUT` reader - Comparison Result for Channel 4"]
pub type RrCh4outR = crate::BitReader;
#[doc = "Field `RR_CH4OUT` writer - Comparison Result for Channel 4"]
pub type RrCh4outW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RR_CH5OUT` reader - Comparison Result for Channel 5"]
pub type RrCh5outR = crate::BitReader;
#[doc = "Field `RR_CH5OUT` writer - Comparison Result for Channel 5"]
pub type RrCh5outW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RR_CH6OUT` reader - Comparison Result for Channel 6"]
pub type RrCh6outR = crate::BitReader;
#[doc = "Field `RR_CH6OUT` writer - Comparison Result for Channel 6"]
pub type RrCh6outW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `RR_CH7OUT` reader - Comparison Result for Channel 7"]
pub type RrCh7outR = crate::BitReader;
#[doc = "Field `RR_CH7OUT` writer - Comparison Result for Channel 7"]
pub type RrCh7outW<'a, REG> = crate::BitWriter<'a, REG>;
impl R {
    #[doc = "Bit 0 - Comparison Result for Channel 0"]
    #[inline(always)]
    pub fn rr_ch0out(&self) -> RrCh0outR {
        RrCh0outR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Comparison Result for Channel 1"]
    #[inline(always)]
    pub fn rr_ch1out(&self) -> RrCh1outR {
        RrCh1outR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Comparison Result for Channel 2"]
    #[inline(always)]
    pub fn rr_ch2out(&self) -> RrCh2outR {
        RrCh2outR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Comparison Result for Channel 3"]
    #[inline(always)]
    pub fn rr_ch3out(&self) -> RrCh3outR {
        RrCh3outR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Comparison Result for Channel 4"]
    #[inline(always)]
    pub fn rr_ch4out(&self) -> RrCh4outR {
        RrCh4outR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Comparison Result for Channel 5"]
    #[inline(always)]
    pub fn rr_ch5out(&self) -> RrCh5outR {
        RrCh5outR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Comparison Result for Channel 6"]
    #[inline(always)]
    pub fn rr_ch6out(&self) -> RrCh6outR {
        RrCh6outR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Comparison Result for Channel 7"]
    #[inline(always)]
    pub fn rr_ch7out(&self) -> RrCh7outR {
        RrCh7outR::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Comparison Result for Channel 0"]
    #[inline(always)]
    pub fn rr_ch0out(&mut self) -> RrCh0outW<RrcsrSpec> {
        RrCh0outW::new(self, 0)
    }
    #[doc = "Bit 1 - Comparison Result for Channel 1"]
    #[inline(always)]
    pub fn rr_ch1out(&mut self) -> RrCh1outW<RrcsrSpec> {
        RrCh1outW::new(self, 1)
    }
    #[doc = "Bit 2 - Comparison Result for Channel 2"]
    #[inline(always)]
    pub fn rr_ch2out(&mut self) -> RrCh2outW<RrcsrSpec> {
        RrCh2outW::new(self, 2)
    }
    #[doc = "Bit 3 - Comparison Result for Channel 3"]
    #[inline(always)]
    pub fn rr_ch3out(&mut self) -> RrCh3outW<RrcsrSpec> {
        RrCh3outW::new(self, 3)
    }
    #[doc = "Bit 4 - Comparison Result for Channel 4"]
    #[inline(always)]
    pub fn rr_ch4out(&mut self) -> RrCh4outW<RrcsrSpec> {
        RrCh4outW::new(self, 4)
    }
    #[doc = "Bit 5 - Comparison Result for Channel 5"]
    #[inline(always)]
    pub fn rr_ch5out(&mut self) -> RrCh5outW<RrcsrSpec> {
        RrCh5outW::new(self, 5)
    }
    #[doc = "Bit 6 - Comparison Result for Channel 6"]
    #[inline(always)]
    pub fn rr_ch6out(&mut self) -> RrCh6outW<RrcsrSpec> {
        RrCh6outW::new(self, 6)
    }
    #[doc = "Bit 7 - Comparison Result for Channel 7"]
    #[inline(always)]
    pub fn rr_ch7out(&mut self) -> RrCh7outW<RrcsrSpec> {
        RrCh7outW::new(self, 7)
    }
}
#[doc = "Round Robin Control and Status\n\nYou can [`read`](crate::Reg::read) this register and get [`rrcsr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`rrcsr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct RrcsrSpec;
impl crate::RegisterSpec for RrcsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`rrcsr::R`](R) reader structure"]
impl crate::Readable for RrcsrSpec {}
#[doc = "`write(|w| ..)` method takes [`rrcsr::W`](W) writer structure"]
impl crate::Writable for RrcsrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets RRCSR to value 0"]
impl crate::Resettable for RrcsrSpec {}

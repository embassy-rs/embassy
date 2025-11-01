#[doc = "Register `FRQMAX` reader"]
pub type R = crate::R<FrqmaxFrqmaxSpec>;
#[doc = "Register `FRQMAX` writer"]
pub type W = crate::W<FrqmaxFrqmaxSpec>;
#[doc = "Field `FRQ_MAX` reader - Frequency Counter Maximum Limit"]
pub type FrqMaxR = crate::FieldReader<u32>;
#[doc = "Field `FRQ_MAX` writer - Frequency Counter Maximum Limit"]
pub type FrqMaxW<'a, REG> = crate::FieldWriter<'a, REG, 22, u32>;
impl R {
    #[doc = "Bits 0:21 - Frequency Counter Maximum Limit"]
    #[inline(always)]
    pub fn frq_max(&self) -> FrqMaxR {
        FrqMaxR::new(self.bits & 0x003f_ffff)
    }
}
impl W {
    #[doc = "Bits 0:21 - Frequency Counter Maximum Limit"]
    #[inline(always)]
    pub fn frq_max(&mut self) -> FrqMaxW<FrqmaxFrqmaxSpec> {
        FrqMaxW::new(self, 0)
    }
}
#[doc = "Frequency Count Maximum Limit Register\n\nYou can [`read`](crate::Reg::read) this register and get [`frqmax_frqmax::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`frqmax_frqmax::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FrqmaxFrqmaxSpec;
impl crate::RegisterSpec for FrqmaxFrqmaxSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`frqmax_frqmax::R`](R) reader structure"]
impl crate::Readable for FrqmaxFrqmaxSpec {}
#[doc = "`write(|w| ..)` method takes [`frqmax_frqmax::W`](W) writer structure"]
impl crate::Writable for FrqmaxFrqmaxSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FRQMAX to value 0x6400"]
impl crate::Resettable for FrqmaxFrqmaxSpec {
    const RESET_VALUE: u32 = 0x6400;
}

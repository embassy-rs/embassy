#[doc = "Register `TRIGFIL[%s]` reader"]
pub type R = crate::R<TrigfilSpec>;
#[doc = "Register `TRIGFIL[%s]` writer"]
pub type W = crate::W<TrigfilSpec>;
#[doc = "Field `FILT_PER` reader - Input Filter Sample Period"]
pub type FiltPerR = crate::FieldReader;
#[doc = "Field `FILT_PER` writer - Input Filter Sample Period"]
pub type FiltPerW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `FILT_CNT` reader - Input Filter Sample Count"]
pub type FiltCntR = crate::FieldReader;
#[doc = "Field `FILT_CNT` writer - Input Filter Sample Count"]
pub type FiltCntW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
impl R {
    #[doc = "Bits 0:7 - Input Filter Sample Period"]
    #[inline(always)]
    pub fn filt_per(&self) -> FiltPerR {
        FiltPerR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:10 - Input Filter Sample Count"]
    #[inline(always)]
    pub fn filt_cnt(&self) -> FiltCntR {
        FiltCntR::new(((self.bits >> 8) & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Input Filter Sample Period"]
    #[inline(always)]
    pub fn filt_per(&mut self) -> FiltPerW<TrigfilSpec> {
        FiltPerW::new(self, 0)
    }
    #[doc = "Bits 8:10 - Input Filter Sample Count"]
    #[inline(always)]
    pub fn filt_cnt(&mut self) -> FiltCntW<TrigfilSpec> {
        FiltCntW::new(self, 8)
    }
}
#[doc = "TRIGFIL control\n\nYou can [`read`](crate::Reg::read) this register and get [`trigfil::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`trigfil::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TrigfilSpec;
impl crate::RegisterSpec for TrigfilSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`trigfil::R`](R) reader structure"]
impl crate::Readable for TrigfilSpec {}
#[doc = "`write(|w| ..)` method takes [`trigfil::W`](W) writer structure"]
impl crate::Writable for TrigfilSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TRIGFIL[%s] to value 0"]
impl crate::Resettable for TrigfilSpec {}

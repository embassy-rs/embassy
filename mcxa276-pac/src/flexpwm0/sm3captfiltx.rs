#[doc = "Register `SM3CAPTFILTX` reader"]
pub type R = crate::R<Sm3captfiltxSpec>;
#[doc = "Register `SM3CAPTFILTX` writer"]
pub type W = crate::W<Sm3captfiltxSpec>;
#[doc = "Field `CAPTX_FILT_PER` reader - Input Capture Filter Period"]
pub type CaptxFiltPerR = crate::FieldReader;
#[doc = "Field `CAPTX_FILT_PER` writer - Input Capture Filter Period"]
pub type CaptxFiltPerW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `CAPTX_FILT_CNT` reader - Input Capture Filter Count"]
pub type CaptxFiltCntR = crate::FieldReader;
#[doc = "Field `CAPTX_FILT_CNT` writer - Input Capture Filter Count"]
pub type CaptxFiltCntW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
impl R {
    #[doc = "Bits 0:7 - Input Capture Filter Period"]
    #[inline(always)]
    pub fn captx_filt_per(&self) -> CaptxFiltPerR {
        CaptxFiltPerR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:10 - Input Capture Filter Count"]
    #[inline(always)]
    pub fn captx_filt_cnt(&self) -> CaptxFiltCntR {
        CaptxFiltCntR::new(((self.bits >> 8) & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Input Capture Filter Period"]
    #[inline(always)]
    pub fn captx_filt_per(&mut self) -> CaptxFiltPerW<Sm3captfiltxSpec> {
        CaptxFiltPerW::new(self, 0)
    }
    #[doc = "Bits 8:10 - Input Capture Filter Count"]
    #[inline(always)]
    pub fn captx_filt_cnt(&mut self) -> CaptxFiltCntW<Sm3captfiltxSpec> {
        CaptxFiltCntW::new(self, 8)
    }
}
#[doc = "Capture PWM_X Input Filter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm3captfiltx::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm3captfiltx::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm3captfiltxSpec;
impl crate::RegisterSpec for Sm3captfiltxSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm3captfiltx::R`](R) reader structure"]
impl crate::Readable for Sm3captfiltxSpec {}
#[doc = "`write(|w| ..)` method takes [`sm3captfiltx::W`](W) writer structure"]
impl crate::Writable for Sm3captfiltxSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM3CAPTFILTX to value 0"]
impl crate::Resettable for Sm3captfiltxSpec {}

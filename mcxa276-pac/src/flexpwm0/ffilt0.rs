#[doc = "Register `FFILT0` reader"]
pub type R = crate::R<Ffilt0Spec>;
#[doc = "Register `FFILT0` writer"]
pub type W = crate::W<Ffilt0Spec>;
#[doc = "Field `FILT_PER` reader - Fault Filter Period"]
pub type FiltPerR = crate::FieldReader;
#[doc = "Field `FILT_PER` writer - Fault Filter Period"]
pub type FiltPerW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `FILT_CNT` reader - Fault Filter Count"]
pub type FiltCntR = crate::FieldReader;
#[doc = "Field `FILT_CNT` writer - Fault Filter Count"]
pub type FiltCntW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Fault Glitch Stretch Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gstr {
    #[doc = "0: Fault input glitch stretching is disabled."]
    Disabled = 0,
    #[doc = "1: Input fault signals are stretched to at least 2 IPBus clock cycles."]
    Enabled = 1,
}
impl From<Gstr> for bool {
    #[inline(always)]
    fn from(variant: Gstr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GSTR` reader - Fault Glitch Stretch Enable"]
pub type GstrR = crate::BitReader<Gstr>;
impl GstrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gstr {
        match self.bits {
            false => Gstr::Disabled,
            true => Gstr::Enabled,
        }
    }
    #[doc = "Fault input glitch stretching is disabled."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Gstr::Disabled
    }
    #[doc = "Input fault signals are stretched to at least 2 IPBus clock cycles."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Gstr::Enabled
    }
}
#[doc = "Field `GSTR` writer - Fault Glitch Stretch Enable"]
pub type GstrW<'a, REG> = crate::BitWriter<'a, REG, Gstr>;
impl<'a, REG> GstrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Fault input glitch stretching is disabled."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gstr::Disabled)
    }
    #[doc = "Input fault signals are stretched to at least 2 IPBus clock cycles."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Gstr::Enabled)
    }
}
impl R {
    #[doc = "Bits 0:7 - Fault Filter Period"]
    #[inline(always)]
    pub fn filt_per(&self) -> FiltPerR {
        FiltPerR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:10 - Fault Filter Count"]
    #[inline(always)]
    pub fn filt_cnt(&self) -> FiltCntR {
        FiltCntR::new(((self.bits >> 8) & 7) as u8)
    }
    #[doc = "Bit 15 - Fault Glitch Stretch Enable"]
    #[inline(always)]
    pub fn gstr(&self) -> GstrR {
        GstrR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:7 - Fault Filter Period"]
    #[inline(always)]
    pub fn filt_per(&mut self) -> FiltPerW<Ffilt0Spec> {
        FiltPerW::new(self, 0)
    }
    #[doc = "Bits 8:10 - Fault Filter Count"]
    #[inline(always)]
    pub fn filt_cnt(&mut self) -> FiltCntW<Ffilt0Spec> {
        FiltCntW::new(self, 8)
    }
    #[doc = "Bit 15 - Fault Glitch Stretch Enable"]
    #[inline(always)]
    pub fn gstr(&mut self) -> GstrW<Ffilt0Spec> {
        GstrW::new(self, 15)
    }
}
#[doc = "Fault Filter Register\n\nYou can [`read`](crate::Reg::read) this register and get [`ffilt0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ffilt0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ffilt0Spec;
impl crate::RegisterSpec for Ffilt0Spec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`ffilt0::R`](R) reader structure"]
impl crate::Readable for Ffilt0Spec {}
#[doc = "`write(|w| ..)` method takes [`ffilt0::W`](W) writer structure"]
impl crate::Writable for Ffilt0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FFILT0 to value 0"]
impl crate::Resettable for Ffilt0Spec {}

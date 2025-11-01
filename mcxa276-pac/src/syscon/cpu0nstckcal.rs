#[doc = "Register `CPU0NSTCKCAL` reader"]
pub type R = crate::R<Cpu0nstckcalSpec>;
#[doc = "Register `CPU0NSTCKCAL` writer"]
pub type W = crate::W<Cpu0nstckcalSpec>;
#[doc = "Field `TENMS` reader - Reload value for 10 ms (100 Hz) timing, subject to system clock skew errors. If the value reads as zero, the calibration value is not known."]
pub type TenmsR = crate::FieldReader<u32>;
#[doc = "Field `TENMS` writer - Reload value for 10 ms (100 Hz) timing, subject to system clock skew errors. If the value reads as zero, the calibration value is not known."]
pub type TenmsW<'a, REG> = crate::FieldWriter<'a, REG, 24, u32>;
#[doc = "Indicates whether the TENMS value is exact.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skew {
    #[doc = "0: TENMS value is exact"]
    Exact = 0,
    #[doc = "1: TENMS value is not exact or not given"]
    Inexact = 1,
}
impl From<Skew> for bool {
    #[inline(always)]
    fn from(variant: Skew) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SKEW` reader - Indicates whether the TENMS value is exact."]
pub type SkewR = crate::BitReader<Skew>;
impl SkewR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Skew {
        match self.bits {
            false => Skew::Exact,
            true => Skew::Inexact,
        }
    }
    #[doc = "TENMS value is exact"]
    #[inline(always)]
    pub fn is_exact(&self) -> bool {
        *self == Skew::Exact
    }
    #[doc = "TENMS value is not exact or not given"]
    #[inline(always)]
    pub fn is_inexact(&self) -> bool {
        *self == Skew::Inexact
    }
}
#[doc = "Field `SKEW` writer - Indicates whether the TENMS value is exact."]
pub type SkewW<'a, REG> = crate::BitWriter<'a, REG, Skew>;
impl<'a, REG> SkewW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "TENMS value is exact"]
    #[inline(always)]
    pub fn exact(self) -> &'a mut crate::W<REG> {
        self.variant(Skew::Exact)
    }
    #[doc = "TENMS value is not exact or not given"]
    #[inline(always)]
    pub fn inexact(self) -> &'a mut crate::W<REG> {
        self.variant(Skew::Inexact)
    }
}
#[doc = "Indicates whether the device provides a reference clock to the processor.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Noref {
    #[doc = "0: Reference clock is provided"]
    YesRef = 0,
    #[doc = "1: No reference clock is provided"]
    NoRef = 1,
}
impl From<Noref> for bool {
    #[inline(always)]
    fn from(variant: Noref) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NOREF` reader - Indicates whether the device provides a reference clock to the processor."]
pub type NorefR = crate::BitReader<Noref>;
impl NorefR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Noref {
        match self.bits {
            false => Noref::YesRef,
            true => Noref::NoRef,
        }
    }
    #[doc = "Reference clock is provided"]
    #[inline(always)]
    pub fn is_yes_ref(&self) -> bool {
        *self == Noref::YesRef
    }
    #[doc = "No reference clock is provided"]
    #[inline(always)]
    pub fn is_no_ref(&self) -> bool {
        *self == Noref::NoRef
    }
}
#[doc = "Field `NOREF` writer - Indicates whether the device provides a reference clock to the processor."]
pub type NorefW<'a, REG> = crate::BitWriter<'a, REG, Noref>;
impl<'a, REG> NorefW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reference clock is provided"]
    #[inline(always)]
    pub fn yes_ref(self) -> &'a mut crate::W<REG> {
        self.variant(Noref::YesRef)
    }
    #[doc = "No reference clock is provided"]
    #[inline(always)]
    pub fn no_ref(self) -> &'a mut crate::W<REG> {
        self.variant(Noref::NoRef)
    }
}
impl R {
    #[doc = "Bits 0:23 - Reload value for 10 ms (100 Hz) timing, subject to system clock skew errors. If the value reads as zero, the calibration value is not known."]
    #[inline(always)]
    pub fn tenms(&self) -> TenmsR {
        TenmsR::new(self.bits & 0x00ff_ffff)
    }
    #[doc = "Bit 24 - Indicates whether the TENMS value is exact."]
    #[inline(always)]
    pub fn skew(&self) -> SkewR {
        SkewR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Indicates whether the device provides a reference clock to the processor."]
    #[inline(always)]
    pub fn noref(&self) -> NorefR {
        NorefR::new(((self.bits >> 25) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:23 - Reload value for 10 ms (100 Hz) timing, subject to system clock skew errors. If the value reads as zero, the calibration value is not known."]
    #[inline(always)]
    pub fn tenms(&mut self) -> TenmsW<Cpu0nstckcalSpec> {
        TenmsW::new(self, 0)
    }
    #[doc = "Bit 24 - Indicates whether the TENMS value is exact."]
    #[inline(always)]
    pub fn skew(&mut self) -> SkewW<Cpu0nstckcalSpec> {
        SkewW::new(self, 24)
    }
    #[doc = "Bit 25 - Indicates whether the device provides a reference clock to the processor."]
    #[inline(always)]
    pub fn noref(&mut self) -> NorefW<Cpu0nstckcalSpec> {
        NorefW::new(self, 25)
    }
}
#[doc = "Non-Secure CPU0 System Tick Calibration\n\nYou can [`read`](crate::Reg::read) this register and get [`cpu0nstckcal::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cpu0nstckcal::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Cpu0nstckcalSpec;
impl crate::RegisterSpec for Cpu0nstckcalSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cpu0nstckcal::R`](R) reader structure"]
impl crate::Readable for Cpu0nstckcalSpec {}
#[doc = "`write(|w| ..)` method takes [`cpu0nstckcal::W`](W) writer structure"]
impl crate::Writable for Cpu0nstckcalSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CPU0NSTCKCAL to value 0"]
impl crate::Resettable for Cpu0nstckcalSpec {}

#[doc = "Register `CTRL` reader"]
pub type R = crate::R<CtrlSpec>;
#[doc = "Register `CTRL` writer"]
pub type W = crate::W<CtrlSpec>;
#[doc = "Field `DELAYVAL` reader - Tick Interval"]
pub type DelayvalR = crate::FieldReader<u32>;
#[doc = "Field `DELAYVAL` writer - Tick Interval"]
pub type DelayvalW<'a, REG> = crate::FieldWriter<'a, REG, 31, u32>;
#[doc = "Repeat Delay\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Repeat {
    #[doc = "0: One-time delay"]
    Delayonce = 0,
    #[doc = "1: Delay repeats continuously"]
    Delayrepeats = 1,
}
impl From<Repeat> for bool {
    #[inline(always)]
    fn from(variant: Repeat) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `REPEAT` reader - Repeat Delay"]
pub type RepeatR = crate::BitReader<Repeat>;
impl RepeatR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Repeat {
        match self.bits {
            false => Repeat::Delayonce,
            true => Repeat::Delayrepeats,
        }
    }
    #[doc = "One-time delay"]
    #[inline(always)]
    pub fn is_delayonce(&self) -> bool {
        *self == Repeat::Delayonce
    }
    #[doc = "Delay repeats continuously"]
    #[inline(always)]
    pub fn is_delayrepeats(&self) -> bool {
        *self == Repeat::Delayrepeats
    }
}
#[doc = "Field `REPEAT` writer - Repeat Delay"]
pub type RepeatW<'a, REG> = crate::BitWriter<'a, REG, Repeat>;
impl<'a, REG> RepeatW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "One-time delay"]
    #[inline(always)]
    pub fn delayonce(self) -> &'a mut crate::W<REG> {
        self.variant(Repeat::Delayonce)
    }
    #[doc = "Delay repeats continuously"]
    #[inline(always)]
    pub fn delayrepeats(self) -> &'a mut crate::W<REG> {
        self.variant(Repeat::Delayrepeats)
    }
}
impl R {
    #[doc = "Bits 0:30 - Tick Interval"]
    #[inline(always)]
    pub fn delayval(&self) -> DelayvalR {
        DelayvalR::new(self.bits & 0x7fff_ffff)
    }
    #[doc = "Bit 31 - Repeat Delay"]
    #[inline(always)]
    pub fn repeat(&self) -> RepeatR {
        RepeatR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:30 - Tick Interval"]
    #[inline(always)]
    pub fn delayval(&mut self) -> DelayvalW<CtrlSpec> {
        DelayvalW::new(self, 0)
    }
    #[doc = "Bit 31 - Repeat Delay"]
    #[inline(always)]
    pub fn repeat(&mut self) -> RepeatW<CtrlSpec> {
        RepeatW::new(self, 31)
    }
}
#[doc = "Control\n\nYou can [`read`](crate::Reg::read) this register and get [`ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CtrlSpec;
impl crate::RegisterSpec for CtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ctrl::R`](R) reader structure"]
impl crate::Readable for CtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`ctrl::W`](W) writer structure"]
impl crate::Writable for CtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CTRL to value 0"]
impl crate::Resettable for CtrlSpec {}

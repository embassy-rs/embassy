#[doc = "Register `SLOWCLKDIV` reader"]
pub type R = crate::R<SlowclkdivSpec>;
#[doc = "Register `SLOWCLKDIV` writer"]
pub type W = crate::W<SlowclkdivSpec>;
#[doc = "Field `DIV` reader - Clock divider value"]
pub type DivR = crate::FieldReader;
#[doc = "Resets the divider counter\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reset {
    #[doc = "0: Divider is not reset"]
    Released = 0,
    #[doc = "1: Divider is reset"]
    Asserted = 1,
}
impl From<Reset> for bool {
    #[inline(always)]
    fn from(variant: Reset) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RESET` reader - Resets the divider counter"]
pub type ResetR = crate::BitReader<Reset>;
impl ResetR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Reset {
        match self.bits {
            false => Reset::Released,
            true => Reset::Asserted,
        }
    }
    #[doc = "Divider is not reset"]
    #[inline(always)]
    pub fn is_released(&self) -> bool {
        *self == Reset::Released
    }
    #[doc = "Divider is reset"]
    #[inline(always)]
    pub fn is_asserted(&self) -> bool {
        *self == Reset::Asserted
    }
}
#[doc = "Field `RESET` writer - Resets the divider counter"]
pub type ResetW<'a, REG> = crate::BitWriter<'a, REG, Reset>;
impl<'a, REG> ResetW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Divider is not reset"]
    #[inline(always)]
    pub fn released(self) -> &'a mut crate::W<REG> {
        self.variant(Reset::Released)
    }
    #[doc = "Divider is reset"]
    #[inline(always)]
    pub fn asserted(self) -> &'a mut crate::W<REG> {
        self.variant(Reset::Asserted)
    }
}
#[doc = "Halts the divider counter\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Halt {
    #[doc = "0: Divider clock is running"]
    Run = 0,
    #[doc = "1: Divider clock is stopped"]
    Halt = 1,
}
impl From<Halt> for bool {
    #[inline(always)]
    fn from(variant: Halt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HALT` reader - Halts the divider counter"]
pub type HaltR = crate::BitReader<Halt>;
impl HaltR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Halt {
        match self.bits {
            false => Halt::Run,
            true => Halt::Halt,
        }
    }
    #[doc = "Divider clock is running"]
    #[inline(always)]
    pub fn is_run(&self) -> bool {
        *self == Halt::Run
    }
    #[doc = "Divider clock is stopped"]
    #[inline(always)]
    pub fn is_halt(&self) -> bool {
        *self == Halt::Halt
    }
}
#[doc = "Field `HALT` writer - Halts the divider counter"]
pub type HaltW<'a, REG> = crate::BitWriter<'a, REG, Halt>;
impl<'a, REG> HaltW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Divider clock is running"]
    #[inline(always)]
    pub fn run(self) -> &'a mut crate::W<REG> {
        self.variant(Halt::Run)
    }
    #[doc = "Divider clock is stopped"]
    #[inline(always)]
    pub fn halt(self) -> &'a mut crate::W<REG> {
        self.variant(Halt::Halt)
    }
}
#[doc = "Divider status flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unstab {
    #[doc = "0: Divider clock is stable"]
    Stable = 0,
    #[doc = "1: Clock frequency is not stable"]
    Ongoing = 1,
}
impl From<Unstab> for bool {
    #[inline(always)]
    fn from(variant: Unstab) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `UNSTAB` reader - Divider status flag"]
pub type UnstabR = crate::BitReader<Unstab>;
impl UnstabR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Unstab {
        match self.bits {
            false => Unstab::Stable,
            true => Unstab::Ongoing,
        }
    }
    #[doc = "Divider clock is stable"]
    #[inline(always)]
    pub fn is_stable(&self) -> bool {
        *self == Unstab::Stable
    }
    #[doc = "Clock frequency is not stable"]
    #[inline(always)]
    pub fn is_ongoing(&self) -> bool {
        *self == Unstab::Ongoing
    }
}
impl R {
    #[doc = "Bits 0:7 - Clock divider value"]
    #[inline(always)]
    pub fn div(&self) -> DivR {
        DivR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bit 29 - Resets the divider counter"]
    #[inline(always)]
    pub fn reset(&self) -> ResetR {
        ResetR::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Halts the divider counter"]
    #[inline(always)]
    pub fn halt(&self) -> HaltR {
        HaltR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Divider status flag"]
    #[inline(always)]
    pub fn unstab(&self) -> UnstabR {
        UnstabR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 29 - Resets the divider counter"]
    #[inline(always)]
    pub fn reset(&mut self) -> ResetW<SlowclkdivSpec> {
        ResetW::new(self, 29)
    }
    #[doc = "Bit 30 - Halts the divider counter"]
    #[inline(always)]
    pub fn halt(&mut self) -> HaltW<SlowclkdivSpec> {
        HaltW::new(self, 30)
    }
}
#[doc = "SLOW_CLK Clock Divider\n\nYou can [`read`](crate::Reg::read) this register and get [`slowclkdiv::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`slowclkdiv::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SlowclkdivSpec;
impl crate::RegisterSpec for SlowclkdivSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`slowclkdiv::R`](R) reader structure"]
impl crate::Readable for SlowclkdivSpec {}
#[doc = "`write(|w| ..)` method takes [`slowclkdiv::W`](W) writer structure"]
impl crate::Writable for SlowclkdivSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SLOWCLKDIV to value 0x05"]
impl crate::Resettable for SlowclkdivSpec {
    const RESET_VALUE: u32 = 0x05;
}

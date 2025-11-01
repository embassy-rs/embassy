#[doc = "Register `MRCC_DAC0_CLKDIV` reader"]
pub type R = crate::R<MrccDac0ClkdivSpec>;
#[doc = "Register `MRCC_DAC0_CLKDIV` writer"]
pub type W = crate::W<MrccDac0ClkdivSpec>;
#[doc = "Field `DIV` reader - Functional Clock Divider"]
pub type DivR = crate::FieldReader;
#[doc = "Field `DIV` writer - Functional Clock Divider"]
pub type DivW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Reset divider counter\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reset {
    #[doc = "0: Divider isn't reset"]
    On = 0,
    #[doc = "1: Divider is reset"]
    Off = 1,
}
impl From<Reset> for bool {
    #[inline(always)]
    fn from(variant: Reset) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RESET` writer - Reset divider counter"]
pub type ResetW<'a, REG> = crate::BitWriter<'a, REG, Reset>;
impl<'a, REG> ResetW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Divider isn't reset"]
    #[inline(always)]
    pub fn on(self) -> &'a mut crate::W<REG> {
        self.variant(Reset::On)
    }
    #[doc = "Divider is reset"]
    #[inline(always)]
    pub fn off(self) -> &'a mut crate::W<REG> {
        self.variant(Reset::Off)
    }
}
#[doc = "Halt divider counter\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Halt {
    #[doc = "0: Divider clock is running"]
    On = 0,
    #[doc = "1: Divider clock is stopped"]
    Off = 1,
}
impl From<Halt> for bool {
    #[inline(always)]
    fn from(variant: Halt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HALT` reader - Halt divider counter"]
pub type HaltR = crate::BitReader<Halt>;
impl HaltR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Halt {
        match self.bits {
            false => Halt::On,
            true => Halt::Off,
        }
    }
    #[doc = "Divider clock is running"]
    #[inline(always)]
    pub fn is_on(&self) -> bool {
        *self == Halt::On
    }
    #[doc = "Divider clock is stopped"]
    #[inline(always)]
    pub fn is_off(&self) -> bool {
        *self == Halt::Off
    }
}
#[doc = "Field `HALT` writer - Halt divider counter"]
pub type HaltW<'a, REG> = crate::BitWriter<'a, REG, Halt>;
impl<'a, REG> HaltW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Divider clock is running"]
    #[inline(always)]
    pub fn on(self) -> &'a mut crate::W<REG> {
        self.variant(Halt::On)
    }
    #[doc = "Divider clock is stopped"]
    #[inline(always)]
    pub fn off(self) -> &'a mut crate::W<REG> {
        self.variant(Halt::Off)
    }
}
#[doc = "Divider status flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unstab {
    #[doc = "0: Divider clock is stable"]
    On = 0,
    #[doc = "1: Clock frequency isn't stable"]
    Off = 1,
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
            false => Unstab::On,
            true => Unstab::Off,
        }
    }
    #[doc = "Divider clock is stable"]
    #[inline(always)]
    pub fn is_on(&self) -> bool {
        *self == Unstab::On
    }
    #[doc = "Clock frequency isn't stable"]
    #[inline(always)]
    pub fn is_off(&self) -> bool {
        *self == Unstab::Off
    }
}
impl R {
    #[doc = "Bits 0:3 - Functional Clock Divider"]
    #[inline(always)]
    pub fn div(&self) -> DivR {
        DivR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bit 30 - Halt divider counter"]
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
    #[doc = "Bits 0:3 - Functional Clock Divider"]
    #[inline(always)]
    pub fn div(&mut self) -> DivW<MrccDac0ClkdivSpec> {
        DivW::new(self, 0)
    }
    #[doc = "Bit 29 - Reset divider counter"]
    #[inline(always)]
    pub fn reset(&mut self) -> ResetW<MrccDac0ClkdivSpec> {
        ResetW::new(self, 29)
    }
    #[doc = "Bit 30 - Halt divider counter"]
    #[inline(always)]
    pub fn halt(&mut self) -> HaltW<MrccDac0ClkdivSpec> {
        HaltW::new(self, 30)
    }
}
#[doc = "DAC0 clock divider control\n\nYou can [`read`](crate::Reg::read) this register and get [`mrcc_dac0_clkdiv::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mrcc_dac0_clkdiv::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MrccDac0ClkdivSpec;
impl crate::RegisterSpec for MrccDac0ClkdivSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mrcc_dac0_clkdiv::R`](R) reader structure"]
impl crate::Readable for MrccDac0ClkdivSpec {}
#[doc = "`write(|w| ..)` method takes [`mrcc_dac0_clkdiv::W`](W) writer structure"]
impl crate::Writable for MrccDac0ClkdivSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MRCC_DAC0_CLKDIV to value 0x4000_0000"]
impl crate::Resettable for MrccDac0ClkdivSpec {
    const RESET_VALUE: u32 = 0x4000_0000;
}

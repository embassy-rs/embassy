#[doc = "Register `PWM1SUBCTL` reader"]
pub type R = crate::R<Pwm1subctlSpec>;
#[doc = "Register `PWM1SUBCTL` writer"]
pub type W = crate::W<Pwm1subctlSpec>;
#[doc = "Enables PWM1 SUB Clock0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clk0En {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Clk0En> for bool {
    #[inline(always)]
    fn from(variant: Clk0En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CLK0_EN` reader - Enables PWM1 SUB Clock0"]
pub type Clk0EnR = crate::BitReader<Clk0En>;
impl Clk0EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Clk0En {
        match self.bits {
            false => Clk0En::Disable,
            true => Clk0En::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Clk0En::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Clk0En::Enable
    }
}
#[doc = "Field `CLK0_EN` writer - Enables PWM1 SUB Clock0"]
pub type Clk0EnW<'a, REG> = crate::BitWriter<'a, REG, Clk0En>;
impl<'a, REG> Clk0EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Clk0En::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Clk0En::Enable)
    }
}
#[doc = "Enables PWM1 SUB Clock1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clk1En {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Clk1En> for bool {
    #[inline(always)]
    fn from(variant: Clk1En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CLK1_EN` reader - Enables PWM1 SUB Clock1"]
pub type Clk1EnR = crate::BitReader<Clk1En>;
impl Clk1EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Clk1En {
        match self.bits {
            false => Clk1En::Disable,
            true => Clk1En::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Clk1En::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Clk1En::Enable
    }
}
#[doc = "Field `CLK1_EN` writer - Enables PWM1 SUB Clock1"]
pub type Clk1EnW<'a, REG> = crate::BitWriter<'a, REG, Clk1En>;
impl<'a, REG> Clk1EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Clk1En::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Clk1En::Enable)
    }
}
#[doc = "Enables PWM1 SUB Clock2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clk2En {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Clk2En> for bool {
    #[inline(always)]
    fn from(variant: Clk2En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CLK2_EN` reader - Enables PWM1 SUB Clock2"]
pub type Clk2EnR = crate::BitReader<Clk2En>;
impl Clk2EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Clk2En {
        match self.bits {
            false => Clk2En::Disable,
            true => Clk2En::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Clk2En::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Clk2En::Enable
    }
}
#[doc = "Field `CLK2_EN` writer - Enables PWM1 SUB Clock2"]
pub type Clk2EnW<'a, REG> = crate::BitWriter<'a, REG, Clk2En>;
impl<'a, REG> Clk2EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Clk2En::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Clk2En::Enable)
    }
}
#[doc = "Enables PWM1 SUB Clock3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clk3En {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Clk3En> for bool {
    #[inline(always)]
    fn from(variant: Clk3En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CLK3_EN` reader - Enables PWM1 SUB Clock3"]
pub type Clk3EnR = crate::BitReader<Clk3En>;
impl Clk3EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Clk3En {
        match self.bits {
            false => Clk3En::Disable,
            true => Clk3En::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Clk3En::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Clk3En::Enable
    }
}
#[doc = "Field `CLK3_EN` writer - Enables PWM1 SUB Clock3"]
pub type Clk3EnW<'a, REG> = crate::BitWriter<'a, REG, Clk3En>;
impl<'a, REG> Clk3EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Clk3En::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Clk3En::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - Enables PWM1 SUB Clock0"]
    #[inline(always)]
    pub fn clk0_en(&self) -> Clk0EnR {
        Clk0EnR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Enables PWM1 SUB Clock1"]
    #[inline(always)]
    pub fn clk1_en(&self) -> Clk1EnR {
        Clk1EnR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Enables PWM1 SUB Clock2"]
    #[inline(always)]
    pub fn clk2_en(&self) -> Clk2EnR {
        Clk2EnR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Enables PWM1 SUB Clock3"]
    #[inline(always)]
    pub fn clk3_en(&self) -> Clk3EnR {
        Clk3EnR::new(((self.bits >> 3) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enables PWM1 SUB Clock0"]
    #[inline(always)]
    pub fn clk0_en(&mut self) -> Clk0EnW<Pwm1subctlSpec> {
        Clk0EnW::new(self, 0)
    }
    #[doc = "Bit 1 - Enables PWM1 SUB Clock1"]
    #[inline(always)]
    pub fn clk1_en(&mut self) -> Clk1EnW<Pwm1subctlSpec> {
        Clk1EnW::new(self, 1)
    }
    #[doc = "Bit 2 - Enables PWM1 SUB Clock2"]
    #[inline(always)]
    pub fn clk2_en(&mut self) -> Clk2EnW<Pwm1subctlSpec> {
        Clk2EnW::new(self, 2)
    }
    #[doc = "Bit 3 - Enables PWM1 SUB Clock3"]
    #[inline(always)]
    pub fn clk3_en(&mut self) -> Clk3EnW<Pwm1subctlSpec> {
        Clk3EnW::new(self, 3)
    }
}
#[doc = "PWM1 Submodule Control\n\nYou can [`read`](crate::Reg::read) this register and get [`pwm1subctl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pwm1subctl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Pwm1subctlSpec;
impl crate::RegisterSpec for Pwm1subctlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pwm1subctl::R`](R) reader structure"]
impl crate::Readable for Pwm1subctlSpec {}
#[doc = "`write(|w| ..)` method takes [`pwm1subctl::W`](W) writer structure"]
impl crate::Writable for Pwm1subctlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PWM1SUBCTL to value 0"]
impl crate::Resettable for Pwm1subctlSpec {}

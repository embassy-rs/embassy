#[doc = "Register `PWMC` reader"]
pub type R = crate::R<PwmcSpec>;
#[doc = "Register `PWMC` writer"]
pub type W = crate::W<PwmcSpec>;
#[doc = "PWM Mode Enable for Channel 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pwmen0 {
    #[doc = "0: Disable"]
    Match = 0,
    #[doc = "1: Enable"]
    Pwm = 1,
}
impl From<Pwmen0> for bool {
    #[inline(always)]
    fn from(variant: Pwmen0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PWMEN0` reader - PWM Mode Enable for Channel 0"]
pub type Pwmen0R = crate::BitReader<Pwmen0>;
impl Pwmen0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pwmen0 {
        match self.bits {
            false => Pwmen0::Match,
            true => Pwmen0::Pwm,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_match(&self) -> bool {
        *self == Pwmen0::Match
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_pwm(&self) -> bool {
        *self == Pwmen0::Pwm
    }
}
#[doc = "Field `PWMEN0` writer - PWM Mode Enable for Channel 0"]
pub type Pwmen0W<'a, REG> = crate::BitWriter<'a, REG, Pwmen0>;
impl<'a, REG> Pwmen0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn match_(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmen0::Match)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmen0::Pwm)
    }
}
#[doc = "PWM Mode Enable for Channel 1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pwmen1 {
    #[doc = "0: Disable"]
    Match = 0,
    #[doc = "1: Enable"]
    Pwm = 1,
}
impl From<Pwmen1> for bool {
    #[inline(always)]
    fn from(variant: Pwmen1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PWMEN1` reader - PWM Mode Enable for Channel 1"]
pub type Pwmen1R = crate::BitReader<Pwmen1>;
impl Pwmen1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pwmen1 {
        match self.bits {
            false => Pwmen1::Match,
            true => Pwmen1::Pwm,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_match(&self) -> bool {
        *self == Pwmen1::Match
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_pwm(&self) -> bool {
        *self == Pwmen1::Pwm
    }
}
#[doc = "Field `PWMEN1` writer - PWM Mode Enable for Channel 1"]
pub type Pwmen1W<'a, REG> = crate::BitWriter<'a, REG, Pwmen1>;
impl<'a, REG> Pwmen1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn match_(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmen1::Match)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmen1::Pwm)
    }
}
#[doc = "PWM Mode Enable for Channel 2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pwmen2 {
    #[doc = "0: Disable"]
    Match = 0,
    #[doc = "1: Enable"]
    Pwm = 1,
}
impl From<Pwmen2> for bool {
    #[inline(always)]
    fn from(variant: Pwmen2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PWMEN2` reader - PWM Mode Enable for Channel 2"]
pub type Pwmen2R = crate::BitReader<Pwmen2>;
impl Pwmen2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pwmen2 {
        match self.bits {
            false => Pwmen2::Match,
            true => Pwmen2::Pwm,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_match(&self) -> bool {
        *self == Pwmen2::Match
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_pwm(&self) -> bool {
        *self == Pwmen2::Pwm
    }
}
#[doc = "Field `PWMEN2` writer - PWM Mode Enable for Channel 2"]
pub type Pwmen2W<'a, REG> = crate::BitWriter<'a, REG, Pwmen2>;
impl<'a, REG> Pwmen2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn match_(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmen2::Match)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmen2::Pwm)
    }
}
#[doc = "PWM Mode Enable for Channel 3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pwmen3 {
    #[doc = "0: Disable"]
    Match = 0,
    #[doc = "1: Enable"]
    Pwm = 1,
}
impl From<Pwmen3> for bool {
    #[inline(always)]
    fn from(variant: Pwmen3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PWMEN3` reader - PWM Mode Enable for Channel 3"]
pub type Pwmen3R = crate::BitReader<Pwmen3>;
impl Pwmen3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pwmen3 {
        match self.bits {
            false => Pwmen3::Match,
            true => Pwmen3::Pwm,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_match(&self) -> bool {
        *self == Pwmen3::Match
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_pwm(&self) -> bool {
        *self == Pwmen3::Pwm
    }
}
#[doc = "Field `PWMEN3` writer - PWM Mode Enable for Channel 3"]
pub type Pwmen3W<'a, REG> = crate::BitWriter<'a, REG, Pwmen3>;
impl<'a, REG> Pwmen3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn match_(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmen3::Match)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn pwm(self) -> &'a mut crate::W<REG> {
        self.variant(Pwmen3::Pwm)
    }
}
impl R {
    #[doc = "Bit 0 - PWM Mode Enable for Channel 0"]
    #[inline(always)]
    pub fn pwmen0(&self) -> Pwmen0R {
        Pwmen0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - PWM Mode Enable for Channel 1"]
    #[inline(always)]
    pub fn pwmen1(&self) -> Pwmen1R {
        Pwmen1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - PWM Mode Enable for Channel 2"]
    #[inline(always)]
    pub fn pwmen2(&self) -> Pwmen2R {
        Pwmen2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - PWM Mode Enable for Channel 3"]
    #[inline(always)]
    pub fn pwmen3(&self) -> Pwmen3R {
        Pwmen3R::new(((self.bits >> 3) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - PWM Mode Enable for Channel 0"]
    #[inline(always)]
    pub fn pwmen0(&mut self) -> Pwmen0W<PwmcSpec> {
        Pwmen0W::new(self, 0)
    }
    #[doc = "Bit 1 - PWM Mode Enable for Channel 1"]
    #[inline(always)]
    pub fn pwmen1(&mut self) -> Pwmen1W<PwmcSpec> {
        Pwmen1W::new(self, 1)
    }
    #[doc = "Bit 2 - PWM Mode Enable for Channel 2"]
    #[inline(always)]
    pub fn pwmen2(&mut self) -> Pwmen2W<PwmcSpec> {
        Pwmen2W::new(self, 2)
    }
    #[doc = "Bit 3 - PWM Mode Enable for Channel 3"]
    #[inline(always)]
    pub fn pwmen3(&mut self) -> Pwmen3W<PwmcSpec> {
        Pwmen3W::new(self, 3)
    }
}
#[doc = "PWM Control\n\nYou can [`read`](crate::Reg::read) this register and get [`pwmc::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pwmc::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PwmcSpec;
impl crate::RegisterSpec for PwmcSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pwmc::R`](R) reader structure"]
impl crate::Readable for PwmcSpec {}
#[doc = "`write(|w| ..)` method takes [`pwmc::W`](W) writer structure"]
impl crate::Writable for PwmcSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PWMC to value 0"]
impl crate::Resettable for PwmcSpec {}

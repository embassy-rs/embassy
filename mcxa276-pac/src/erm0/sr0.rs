#[doc = "Register `SR0` reader"]
pub type R = crate::R<Sr0Spec>;
#[doc = "Register `SR0` writer"]
pub type W = crate::W<Sr0Spec>;
#[doc = "NCE1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nce1 {
    #[doc = "0: No non-correctable error event on Memory 1 detected."]
    NoError = 0,
    #[doc = "1: Non-correctable error event on Memory 1 detected."]
    Error = 1,
}
impl From<Nce1> for bool {
    #[inline(always)]
    fn from(variant: Nce1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NCE1` reader - NCE1"]
pub type Nce1R = crate::BitReader<Nce1>;
impl Nce1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nce1 {
        match self.bits {
            false => Nce1::NoError,
            true => Nce1::Error,
        }
    }
    #[doc = "No non-correctable error event on Memory 1 detected."]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Nce1::NoError
    }
    #[doc = "Non-correctable error event on Memory 1 detected."]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Nce1::Error
    }
}
#[doc = "Field `NCE1` writer - NCE1"]
pub type Nce1W<'a, REG> = crate::BitWriter1C<'a, REG, Nce1>;
impl<'a, REG> Nce1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No non-correctable error event on Memory 1 detected."]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Nce1::NoError)
    }
    #[doc = "Non-correctable error event on Memory 1 detected."]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Nce1::Error)
    }
}
#[doc = "SBC1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sbc1 {
    #[doc = "0: No single-bit correction event on Memory 1 detected."]
    NoEvent = 0,
    #[doc = "1: Single-bit correction event on Memory 1 detected."]
    Event = 1,
}
impl From<Sbc1> for bool {
    #[inline(always)]
    fn from(variant: Sbc1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SBC1` reader - SBC1"]
pub type Sbc1R = crate::BitReader<Sbc1>;
impl Sbc1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sbc1 {
        match self.bits {
            false => Sbc1::NoEvent,
            true => Sbc1::Event,
        }
    }
    #[doc = "No single-bit correction event on Memory 1 detected."]
    #[inline(always)]
    pub fn is_no_event(&self) -> bool {
        *self == Sbc1::NoEvent
    }
    #[doc = "Single-bit correction event on Memory 1 detected."]
    #[inline(always)]
    pub fn is_event(&self) -> bool {
        *self == Sbc1::Event
    }
}
#[doc = "Field `SBC1` writer - SBC1"]
pub type Sbc1W<'a, REG> = crate::BitWriter1C<'a, REG, Sbc1>;
impl<'a, REG> Sbc1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No single-bit correction event on Memory 1 detected."]
    #[inline(always)]
    pub fn no_event(self) -> &'a mut crate::W<REG> {
        self.variant(Sbc1::NoEvent)
    }
    #[doc = "Single-bit correction event on Memory 1 detected."]
    #[inline(always)]
    pub fn event(self) -> &'a mut crate::W<REG> {
        self.variant(Sbc1::Event)
    }
}
#[doc = "NCE0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nce0 {
    #[doc = "0: No non-correctable error event on Memory 0 detected."]
    NoError = 0,
    #[doc = "1: Non-correctable error event on Memory 0 detected."]
    Error = 1,
}
impl From<Nce0> for bool {
    #[inline(always)]
    fn from(variant: Nce0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NCE0` reader - NCE0"]
pub type Nce0R = crate::BitReader<Nce0>;
impl Nce0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nce0 {
        match self.bits {
            false => Nce0::NoError,
            true => Nce0::Error,
        }
    }
    #[doc = "No non-correctable error event on Memory 0 detected."]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Nce0::NoError
    }
    #[doc = "Non-correctable error event on Memory 0 detected."]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Nce0::Error
    }
}
#[doc = "Field `NCE0` writer - NCE0"]
pub type Nce0W<'a, REG> = crate::BitWriter1C<'a, REG, Nce0>;
impl<'a, REG> Nce0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No non-correctable error event on Memory 0 detected."]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Nce0::NoError)
    }
    #[doc = "Non-correctable error event on Memory 0 detected."]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Nce0::Error)
    }
}
#[doc = "SBC0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sbc0 {
    #[doc = "0: No single-bit correction event on Memory 0 detected."]
    NoEvent = 0,
    #[doc = "1: Single-bit correction event on Memory 0 detected."]
    Event = 1,
}
impl From<Sbc0> for bool {
    #[inline(always)]
    fn from(variant: Sbc0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SBC0` reader - SBC0"]
pub type Sbc0R = crate::BitReader<Sbc0>;
impl Sbc0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sbc0 {
        match self.bits {
            false => Sbc0::NoEvent,
            true => Sbc0::Event,
        }
    }
    #[doc = "No single-bit correction event on Memory 0 detected."]
    #[inline(always)]
    pub fn is_no_event(&self) -> bool {
        *self == Sbc0::NoEvent
    }
    #[doc = "Single-bit correction event on Memory 0 detected."]
    #[inline(always)]
    pub fn is_event(&self) -> bool {
        *self == Sbc0::Event
    }
}
#[doc = "Field `SBC0` writer - SBC0"]
pub type Sbc0W<'a, REG> = crate::BitWriter1C<'a, REG, Sbc0>;
impl<'a, REG> Sbc0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No single-bit correction event on Memory 0 detected."]
    #[inline(always)]
    pub fn no_event(self) -> &'a mut crate::W<REG> {
        self.variant(Sbc0::NoEvent)
    }
    #[doc = "Single-bit correction event on Memory 0 detected."]
    #[inline(always)]
    pub fn event(self) -> &'a mut crate::W<REG> {
        self.variant(Sbc0::Event)
    }
}
impl R {
    #[doc = "Bit 26 - NCE1"]
    #[inline(always)]
    pub fn nce1(&self) -> Nce1R {
        Nce1R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - SBC1"]
    #[inline(always)]
    pub fn sbc1(&self) -> Sbc1R {
        Sbc1R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 30 - NCE0"]
    #[inline(always)]
    pub fn nce0(&self) -> Nce0R {
        Nce0R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - SBC0"]
    #[inline(always)]
    pub fn sbc0(&self) -> Sbc0R {
        Sbc0R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 26 - NCE1"]
    #[inline(always)]
    pub fn nce1(&mut self) -> Nce1W<Sr0Spec> {
        Nce1W::new(self, 26)
    }
    #[doc = "Bit 27 - SBC1"]
    #[inline(always)]
    pub fn sbc1(&mut self) -> Sbc1W<Sr0Spec> {
        Sbc1W::new(self, 27)
    }
    #[doc = "Bit 30 - NCE0"]
    #[inline(always)]
    pub fn nce0(&mut self) -> Nce0W<Sr0Spec> {
        Nce0W::new(self, 30)
    }
    #[doc = "Bit 31 - SBC0"]
    #[inline(always)]
    pub fn sbc0(&mut self) -> Sbc0W<Sr0Spec> {
        Sbc0W::new(self, 31)
    }
}
#[doc = "ERM Status Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`sr0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sr0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sr0Spec;
impl crate::RegisterSpec for Sr0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sr0::R`](R) reader structure"]
impl crate::Readable for Sr0Spec {}
#[doc = "`write(|w| ..)` method takes [`sr0::W`](W) writer structure"]
impl crate::Writable for Sr0Spec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0xcc00_0000;
}
#[doc = "`reset()` method sets SR0 to value 0"]
impl crate::Resettable for Sr0Spec {}

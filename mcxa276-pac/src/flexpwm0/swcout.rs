#[doc = "Register `SWCOUT` reader"]
pub type R = crate::R<SwcoutSpec>;
#[doc = "Register `SWCOUT` writer"]
pub type W = crate::W<SwcoutSpec>;
#[doc = "Submodule 0 Software Controlled Output 45\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sm0out45 {
    #[doc = "0: A logic 0 is supplied to the deadtime generator of submodule 0 instead of PWM45."]
    Logic0 = 0,
    #[doc = "1: A logic 1 is supplied to the deadtime generator of submodule 0 instead of PWM45."]
    Logic1 = 1,
}
impl From<Sm0out45> for bool {
    #[inline(always)]
    fn from(variant: Sm0out45) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SM0OUT45` reader - Submodule 0 Software Controlled Output 45"]
pub type Sm0out45R = crate::BitReader<Sm0out45>;
impl Sm0out45R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sm0out45 {
        match self.bits {
            false => Sm0out45::Logic0,
            true => Sm0out45::Logic1,
        }
    }
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 0 instead of PWM45."]
    #[inline(always)]
    pub fn is_logic_0(&self) -> bool {
        *self == Sm0out45::Logic0
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 0 instead of PWM45."]
    #[inline(always)]
    pub fn is_logic_1(&self) -> bool {
        *self == Sm0out45::Logic1
    }
}
#[doc = "Field `SM0OUT45` writer - Submodule 0 Software Controlled Output 45"]
pub type Sm0out45W<'a, REG> = crate::BitWriter<'a, REG, Sm0out45>;
impl<'a, REG> Sm0out45W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 0 instead of PWM45."]
    #[inline(always)]
    pub fn logic_0(self) -> &'a mut crate::W<REG> {
        self.variant(Sm0out45::Logic0)
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 0 instead of PWM45."]
    #[inline(always)]
    pub fn logic_1(self) -> &'a mut crate::W<REG> {
        self.variant(Sm0out45::Logic1)
    }
}
#[doc = "Submodule 0 Software Controlled Output 23\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sm0out23 {
    #[doc = "0: A logic 0 is supplied to the deadtime generator of submodule 0 instead of PWM23."]
    Logic0 = 0,
    #[doc = "1: A logic 1 is supplied to the deadtime generator of submodule 0 instead of PWM23."]
    Logic1 = 1,
}
impl From<Sm0out23> for bool {
    #[inline(always)]
    fn from(variant: Sm0out23) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SM0OUT23` reader - Submodule 0 Software Controlled Output 23"]
pub type Sm0out23R = crate::BitReader<Sm0out23>;
impl Sm0out23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sm0out23 {
        match self.bits {
            false => Sm0out23::Logic0,
            true => Sm0out23::Logic1,
        }
    }
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 0 instead of PWM23."]
    #[inline(always)]
    pub fn is_logic_0(&self) -> bool {
        *self == Sm0out23::Logic0
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 0 instead of PWM23."]
    #[inline(always)]
    pub fn is_logic_1(&self) -> bool {
        *self == Sm0out23::Logic1
    }
}
#[doc = "Field `SM0OUT23` writer - Submodule 0 Software Controlled Output 23"]
pub type Sm0out23W<'a, REG> = crate::BitWriter<'a, REG, Sm0out23>;
impl<'a, REG> Sm0out23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 0 instead of PWM23."]
    #[inline(always)]
    pub fn logic_0(self) -> &'a mut crate::W<REG> {
        self.variant(Sm0out23::Logic0)
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 0 instead of PWM23."]
    #[inline(always)]
    pub fn logic_1(self) -> &'a mut crate::W<REG> {
        self.variant(Sm0out23::Logic1)
    }
}
#[doc = "Submodule 1 Software Controlled Output 45\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sm1out45 {
    #[doc = "0: A logic 0 is supplied to the deadtime generator of submodule 1 instead of PWM45."]
    Logic0 = 0,
    #[doc = "1: A logic 1 is supplied to the deadtime generator of submodule 1 instead of PWM45."]
    Logic1 = 1,
}
impl From<Sm1out45> for bool {
    #[inline(always)]
    fn from(variant: Sm1out45) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SM1OUT45` reader - Submodule 1 Software Controlled Output 45"]
pub type Sm1out45R = crate::BitReader<Sm1out45>;
impl Sm1out45R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sm1out45 {
        match self.bits {
            false => Sm1out45::Logic0,
            true => Sm1out45::Logic1,
        }
    }
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 1 instead of PWM45."]
    #[inline(always)]
    pub fn is_logic_0(&self) -> bool {
        *self == Sm1out45::Logic0
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 1 instead of PWM45."]
    #[inline(always)]
    pub fn is_logic_1(&self) -> bool {
        *self == Sm1out45::Logic1
    }
}
#[doc = "Field `SM1OUT45` writer - Submodule 1 Software Controlled Output 45"]
pub type Sm1out45W<'a, REG> = crate::BitWriter<'a, REG, Sm1out45>;
impl<'a, REG> Sm1out45W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 1 instead of PWM45."]
    #[inline(always)]
    pub fn logic_0(self) -> &'a mut crate::W<REG> {
        self.variant(Sm1out45::Logic0)
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 1 instead of PWM45."]
    #[inline(always)]
    pub fn logic_1(self) -> &'a mut crate::W<REG> {
        self.variant(Sm1out45::Logic1)
    }
}
#[doc = "Submodule 1 Software Controlled Output 23\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sm1out23 {
    #[doc = "0: A logic 0 is supplied to the deadtime generator of submodule 1 instead of PWM23."]
    Logic0 = 0,
    #[doc = "1: A logic 1 is supplied to the deadtime generator of submodule 1 instead of PWM23."]
    Logic1 = 1,
}
impl From<Sm1out23> for bool {
    #[inline(always)]
    fn from(variant: Sm1out23) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SM1OUT23` reader - Submodule 1 Software Controlled Output 23"]
pub type Sm1out23R = crate::BitReader<Sm1out23>;
impl Sm1out23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sm1out23 {
        match self.bits {
            false => Sm1out23::Logic0,
            true => Sm1out23::Logic1,
        }
    }
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 1 instead of PWM23."]
    #[inline(always)]
    pub fn is_logic_0(&self) -> bool {
        *self == Sm1out23::Logic0
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 1 instead of PWM23."]
    #[inline(always)]
    pub fn is_logic_1(&self) -> bool {
        *self == Sm1out23::Logic1
    }
}
#[doc = "Field `SM1OUT23` writer - Submodule 1 Software Controlled Output 23"]
pub type Sm1out23W<'a, REG> = crate::BitWriter<'a, REG, Sm1out23>;
impl<'a, REG> Sm1out23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 1 instead of PWM23."]
    #[inline(always)]
    pub fn logic_0(self) -> &'a mut crate::W<REG> {
        self.variant(Sm1out23::Logic0)
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 1 instead of PWM23."]
    #[inline(always)]
    pub fn logic_1(self) -> &'a mut crate::W<REG> {
        self.variant(Sm1out23::Logic1)
    }
}
#[doc = "Submodule 2 Software Controlled Output 45\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sm2out45 {
    #[doc = "0: A logic 0 is supplied to the deadtime generator of submodule 2 instead of PWM45."]
    Logic0 = 0,
    #[doc = "1: A logic 1 is supplied to the deadtime generator of submodule 2 instead of PWM45."]
    Logic1 = 1,
}
impl From<Sm2out45> for bool {
    #[inline(always)]
    fn from(variant: Sm2out45) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SM2OUT45` reader - Submodule 2 Software Controlled Output 45"]
pub type Sm2out45R = crate::BitReader<Sm2out45>;
impl Sm2out45R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sm2out45 {
        match self.bits {
            false => Sm2out45::Logic0,
            true => Sm2out45::Logic1,
        }
    }
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 2 instead of PWM45."]
    #[inline(always)]
    pub fn is_logic_0(&self) -> bool {
        *self == Sm2out45::Logic0
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 2 instead of PWM45."]
    #[inline(always)]
    pub fn is_logic_1(&self) -> bool {
        *self == Sm2out45::Logic1
    }
}
#[doc = "Field `SM2OUT45` writer - Submodule 2 Software Controlled Output 45"]
pub type Sm2out45W<'a, REG> = crate::BitWriter<'a, REG, Sm2out45>;
impl<'a, REG> Sm2out45W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 2 instead of PWM45."]
    #[inline(always)]
    pub fn logic_0(self) -> &'a mut crate::W<REG> {
        self.variant(Sm2out45::Logic0)
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 2 instead of PWM45."]
    #[inline(always)]
    pub fn logic_1(self) -> &'a mut crate::W<REG> {
        self.variant(Sm2out45::Logic1)
    }
}
#[doc = "Submodule 2 Software Controlled Output 23\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sm2out23 {
    #[doc = "0: A logic 0 is supplied to the deadtime generator of submodule 2 instead of PWM23."]
    Logic0 = 0,
    #[doc = "1: A logic 1 is supplied to the deadtime generator of submodule 2 instead of PWM23."]
    Logic1 = 1,
}
impl From<Sm2out23> for bool {
    #[inline(always)]
    fn from(variant: Sm2out23) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SM2OUT23` reader - Submodule 2 Software Controlled Output 23"]
pub type Sm2out23R = crate::BitReader<Sm2out23>;
impl Sm2out23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sm2out23 {
        match self.bits {
            false => Sm2out23::Logic0,
            true => Sm2out23::Logic1,
        }
    }
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 2 instead of PWM23."]
    #[inline(always)]
    pub fn is_logic_0(&self) -> bool {
        *self == Sm2out23::Logic0
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 2 instead of PWM23."]
    #[inline(always)]
    pub fn is_logic_1(&self) -> bool {
        *self == Sm2out23::Logic1
    }
}
#[doc = "Field `SM2OUT23` writer - Submodule 2 Software Controlled Output 23"]
pub type Sm2out23W<'a, REG> = crate::BitWriter<'a, REG, Sm2out23>;
impl<'a, REG> Sm2out23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 2 instead of PWM23."]
    #[inline(always)]
    pub fn logic_0(self) -> &'a mut crate::W<REG> {
        self.variant(Sm2out23::Logic0)
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 2 instead of PWM23."]
    #[inline(always)]
    pub fn logic_1(self) -> &'a mut crate::W<REG> {
        self.variant(Sm2out23::Logic1)
    }
}
#[doc = "Submodule 3 Software Controlled Output 45\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sm3out45 {
    #[doc = "0: A logic 0 is supplied to the deadtime generator of submodule 3 instead of PWM45."]
    Logic0 = 0,
    #[doc = "1: A logic 1 is supplied to the deadtime generator of submodule 3 instead of PWM45."]
    Logic1 = 1,
}
impl From<Sm3out45> for bool {
    #[inline(always)]
    fn from(variant: Sm3out45) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SM3OUT45` reader - Submodule 3 Software Controlled Output 45"]
pub type Sm3out45R = crate::BitReader<Sm3out45>;
impl Sm3out45R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sm3out45 {
        match self.bits {
            false => Sm3out45::Logic0,
            true => Sm3out45::Logic1,
        }
    }
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 3 instead of PWM45."]
    #[inline(always)]
    pub fn is_logic_0(&self) -> bool {
        *self == Sm3out45::Logic0
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 3 instead of PWM45."]
    #[inline(always)]
    pub fn is_logic_1(&self) -> bool {
        *self == Sm3out45::Logic1
    }
}
#[doc = "Field `SM3OUT45` writer - Submodule 3 Software Controlled Output 45"]
pub type Sm3out45W<'a, REG> = crate::BitWriter<'a, REG, Sm3out45>;
impl<'a, REG> Sm3out45W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 3 instead of PWM45."]
    #[inline(always)]
    pub fn logic_0(self) -> &'a mut crate::W<REG> {
        self.variant(Sm3out45::Logic0)
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 3 instead of PWM45."]
    #[inline(always)]
    pub fn logic_1(self) -> &'a mut crate::W<REG> {
        self.variant(Sm3out45::Logic1)
    }
}
#[doc = "Submodule 3 Software Controlled Output 23\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sm3out23 {
    #[doc = "0: A logic 0 is supplied to the deadtime generator of submodule 3 instead of PWM23."]
    Logic0 = 0,
    #[doc = "1: A logic 1 is supplied to the deadtime generator of submodule 3 instead of PWM23."]
    Logic1 = 1,
}
impl From<Sm3out23> for bool {
    #[inline(always)]
    fn from(variant: Sm3out23) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SM3OUT23` reader - Submodule 3 Software Controlled Output 23"]
pub type Sm3out23R = crate::BitReader<Sm3out23>;
impl Sm3out23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sm3out23 {
        match self.bits {
            false => Sm3out23::Logic0,
            true => Sm3out23::Logic1,
        }
    }
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 3 instead of PWM23."]
    #[inline(always)]
    pub fn is_logic_0(&self) -> bool {
        *self == Sm3out23::Logic0
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 3 instead of PWM23."]
    #[inline(always)]
    pub fn is_logic_1(&self) -> bool {
        *self == Sm3out23::Logic1
    }
}
#[doc = "Field `SM3OUT23` writer - Submodule 3 Software Controlled Output 23"]
pub type Sm3out23W<'a, REG> = crate::BitWriter<'a, REG, Sm3out23>;
impl<'a, REG> Sm3out23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "A logic 0 is supplied to the deadtime generator of submodule 3 instead of PWM23."]
    #[inline(always)]
    pub fn logic_0(self) -> &'a mut crate::W<REG> {
        self.variant(Sm3out23::Logic0)
    }
    #[doc = "A logic 1 is supplied to the deadtime generator of submodule 3 instead of PWM23."]
    #[inline(always)]
    pub fn logic_1(self) -> &'a mut crate::W<REG> {
        self.variant(Sm3out23::Logic1)
    }
}
impl R {
    #[doc = "Bit 0 - Submodule 0 Software Controlled Output 45"]
    #[inline(always)]
    pub fn sm0out45(&self) -> Sm0out45R {
        Sm0out45R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Submodule 0 Software Controlled Output 23"]
    #[inline(always)]
    pub fn sm0out23(&self) -> Sm0out23R {
        Sm0out23R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Submodule 1 Software Controlled Output 45"]
    #[inline(always)]
    pub fn sm1out45(&self) -> Sm1out45R {
        Sm1out45R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Submodule 1 Software Controlled Output 23"]
    #[inline(always)]
    pub fn sm1out23(&self) -> Sm1out23R {
        Sm1out23R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Submodule 2 Software Controlled Output 45"]
    #[inline(always)]
    pub fn sm2out45(&self) -> Sm2out45R {
        Sm2out45R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Submodule 2 Software Controlled Output 23"]
    #[inline(always)]
    pub fn sm2out23(&self) -> Sm2out23R {
        Sm2out23R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Submodule 3 Software Controlled Output 45"]
    #[inline(always)]
    pub fn sm3out45(&self) -> Sm3out45R {
        Sm3out45R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Submodule 3 Software Controlled Output 23"]
    #[inline(always)]
    pub fn sm3out23(&self) -> Sm3out23R {
        Sm3out23R::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Submodule 0 Software Controlled Output 45"]
    #[inline(always)]
    pub fn sm0out45(&mut self) -> Sm0out45W<SwcoutSpec> {
        Sm0out45W::new(self, 0)
    }
    #[doc = "Bit 1 - Submodule 0 Software Controlled Output 23"]
    #[inline(always)]
    pub fn sm0out23(&mut self) -> Sm0out23W<SwcoutSpec> {
        Sm0out23W::new(self, 1)
    }
    #[doc = "Bit 2 - Submodule 1 Software Controlled Output 45"]
    #[inline(always)]
    pub fn sm1out45(&mut self) -> Sm1out45W<SwcoutSpec> {
        Sm1out45W::new(self, 2)
    }
    #[doc = "Bit 3 - Submodule 1 Software Controlled Output 23"]
    #[inline(always)]
    pub fn sm1out23(&mut self) -> Sm1out23W<SwcoutSpec> {
        Sm1out23W::new(self, 3)
    }
    #[doc = "Bit 4 - Submodule 2 Software Controlled Output 45"]
    #[inline(always)]
    pub fn sm2out45(&mut self) -> Sm2out45W<SwcoutSpec> {
        Sm2out45W::new(self, 4)
    }
    #[doc = "Bit 5 - Submodule 2 Software Controlled Output 23"]
    #[inline(always)]
    pub fn sm2out23(&mut self) -> Sm2out23W<SwcoutSpec> {
        Sm2out23W::new(self, 5)
    }
    #[doc = "Bit 6 - Submodule 3 Software Controlled Output 45"]
    #[inline(always)]
    pub fn sm3out45(&mut self) -> Sm3out45W<SwcoutSpec> {
        Sm3out45W::new(self, 6)
    }
    #[doc = "Bit 7 - Submodule 3 Software Controlled Output 23"]
    #[inline(always)]
    pub fn sm3out23(&mut self) -> Sm3out23W<SwcoutSpec> {
        Sm3out23W::new(self, 7)
    }
}
#[doc = "Software Controlled Output Register\n\nYou can [`read`](crate::Reg::read) this register and get [`swcout::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`swcout::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SwcoutSpec;
impl crate::RegisterSpec for SwcoutSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`swcout::R`](R) reader structure"]
impl crate::Readable for SwcoutSpec {}
#[doc = "`write(|w| ..)` method takes [`swcout::W`](W) writer structure"]
impl crate::Writable for SwcoutSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SWCOUT to value 0"]
impl crate::Resettable for SwcoutSpec {}

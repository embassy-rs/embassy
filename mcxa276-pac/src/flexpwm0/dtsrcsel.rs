#[doc = "Register `DTSRCSEL` reader"]
pub type R = crate::R<DtsrcselSpec>;
#[doc = "Register `DTSRCSEL` writer"]
pub type W = crate::W<DtsrcselSpec>;
#[doc = "Submodule 0 PWM45 Control Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Sm0sel45 {
    #[doc = "0: Generated SM0PWM45 signal used by the deadtime logic."]
    Sm0pwm45 = 0,
    #[doc = "1: Inverted generated SM0PWM45 signal used by the deadtime logic."]
    InvertedSm0pwm45 = 1,
    #[doc = "2: SWCOUT\\[SM0OUT45\\] used by the deadtime logic."]
    Sm0out45 = 2,
}
impl From<Sm0sel45> for u8 {
    #[inline(always)]
    fn from(variant: Sm0sel45) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Sm0sel45 {
    type Ux = u8;
}
impl crate::IsEnum for Sm0sel45 {}
#[doc = "Field `SM0SEL45` reader - Submodule 0 PWM45 Control Select"]
pub type Sm0sel45R = crate::FieldReader<Sm0sel45>;
impl Sm0sel45R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Sm0sel45> {
        match self.bits {
            0 => Some(Sm0sel45::Sm0pwm45),
            1 => Some(Sm0sel45::InvertedSm0pwm45),
            2 => Some(Sm0sel45::Sm0out45),
            _ => None,
        }
    }
    #[doc = "Generated SM0PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm0pwm45(&self) -> bool {
        *self == Sm0sel45::Sm0pwm45
    }
    #[doc = "Inverted generated SM0PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_inverted_sm0pwm45(&self) -> bool {
        *self == Sm0sel45::InvertedSm0pwm45
    }
    #[doc = "SWCOUT\\[SM0OUT45\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm0out45(&self) -> bool {
        *self == Sm0sel45::Sm0out45
    }
}
#[doc = "Field `SM0SEL45` writer - Submodule 0 PWM45 Control Select"]
pub type Sm0sel45W<'a, REG> = crate::FieldWriter<'a, REG, 2, Sm0sel45>;
impl<'a, REG> Sm0sel45W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Generated SM0PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn sm0pwm45(self) -> &'a mut crate::W<REG> {
        self.variant(Sm0sel45::Sm0pwm45)
    }
    #[doc = "Inverted generated SM0PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn inverted_sm0pwm45(self) -> &'a mut crate::W<REG> {
        self.variant(Sm0sel45::InvertedSm0pwm45)
    }
    #[doc = "SWCOUT\\[SM0OUT45\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn sm0out45(self) -> &'a mut crate::W<REG> {
        self.variant(Sm0sel45::Sm0out45)
    }
}
#[doc = "Submodule 0 PWM23 Control Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Sm0sel23 {
    #[doc = "0: Generated SM0PWM23 signal used by the deadtime logic."]
    Sm0pwm23 = 0,
    #[doc = "1: Inverted generated SM0PWM23 signal used by the deadtime logic."]
    InvertedSm0pwm23 = 1,
    #[doc = "2: SWCOUT\\[SM0OUT23\\] used by the deadtime logic."]
    Sm0out23 = 2,
    #[doc = "3: PWM0_EXTA signal used by the deadtime logic."]
    Pwm0Exta = 3,
}
impl From<Sm0sel23> for u8 {
    #[inline(always)]
    fn from(variant: Sm0sel23) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Sm0sel23 {
    type Ux = u8;
}
impl crate::IsEnum for Sm0sel23 {}
#[doc = "Field `SM0SEL23` reader - Submodule 0 PWM23 Control Select"]
pub type Sm0sel23R = crate::FieldReader<Sm0sel23>;
impl Sm0sel23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sm0sel23 {
        match self.bits {
            0 => Sm0sel23::Sm0pwm23,
            1 => Sm0sel23::InvertedSm0pwm23,
            2 => Sm0sel23::Sm0out23,
            3 => Sm0sel23::Pwm0Exta,
            _ => unreachable!(),
        }
    }
    #[doc = "Generated SM0PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm0pwm23(&self) -> bool {
        *self == Sm0sel23::Sm0pwm23
    }
    #[doc = "Inverted generated SM0PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_inverted_sm0pwm23(&self) -> bool {
        *self == Sm0sel23::InvertedSm0pwm23
    }
    #[doc = "SWCOUT\\[SM0OUT23\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm0out23(&self) -> bool {
        *self == Sm0sel23::Sm0out23
    }
    #[doc = "PWM0_EXTA signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_pwm0_exta(&self) -> bool {
        *self == Sm0sel23::Pwm0Exta
    }
}
#[doc = "Field `SM0SEL23` writer - Submodule 0 PWM23 Control Select"]
pub type Sm0sel23W<'a, REG> = crate::FieldWriter<'a, REG, 2, Sm0sel23, crate::Safe>;
impl<'a, REG> Sm0sel23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Generated SM0PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn sm0pwm23(self) -> &'a mut crate::W<REG> {
        self.variant(Sm0sel23::Sm0pwm23)
    }
    #[doc = "Inverted generated SM0PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn inverted_sm0pwm23(self) -> &'a mut crate::W<REG> {
        self.variant(Sm0sel23::InvertedSm0pwm23)
    }
    #[doc = "SWCOUT\\[SM0OUT23\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn sm0out23(self) -> &'a mut crate::W<REG> {
        self.variant(Sm0sel23::Sm0out23)
    }
    #[doc = "PWM0_EXTA signal used by the deadtime logic."]
    #[inline(always)]
    pub fn pwm0_exta(self) -> &'a mut crate::W<REG> {
        self.variant(Sm0sel23::Pwm0Exta)
    }
}
#[doc = "Submodule 1 PWM45 Control Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Sm1sel45 {
    #[doc = "0: Generated SM1PWM45 signal used by the deadtime logic."]
    Sm1pwm45 = 0,
    #[doc = "1: Inverted generated SM1PWM45 signal used by the deadtime logic."]
    InvertedSm1pwm45 = 1,
    #[doc = "2: SWCOUT\\[SM1OUT45\\] used by the deadtime logic."]
    Sm1out45 = 2,
}
impl From<Sm1sel45> for u8 {
    #[inline(always)]
    fn from(variant: Sm1sel45) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Sm1sel45 {
    type Ux = u8;
}
impl crate::IsEnum for Sm1sel45 {}
#[doc = "Field `SM1SEL45` reader - Submodule 1 PWM45 Control Select"]
pub type Sm1sel45R = crate::FieldReader<Sm1sel45>;
impl Sm1sel45R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Sm1sel45> {
        match self.bits {
            0 => Some(Sm1sel45::Sm1pwm45),
            1 => Some(Sm1sel45::InvertedSm1pwm45),
            2 => Some(Sm1sel45::Sm1out45),
            _ => None,
        }
    }
    #[doc = "Generated SM1PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm1pwm45(&self) -> bool {
        *self == Sm1sel45::Sm1pwm45
    }
    #[doc = "Inverted generated SM1PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_inverted_sm1pwm45(&self) -> bool {
        *self == Sm1sel45::InvertedSm1pwm45
    }
    #[doc = "SWCOUT\\[SM1OUT45\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm1out45(&self) -> bool {
        *self == Sm1sel45::Sm1out45
    }
}
#[doc = "Field `SM1SEL45` writer - Submodule 1 PWM45 Control Select"]
pub type Sm1sel45W<'a, REG> = crate::FieldWriter<'a, REG, 2, Sm1sel45>;
impl<'a, REG> Sm1sel45W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Generated SM1PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn sm1pwm45(self) -> &'a mut crate::W<REG> {
        self.variant(Sm1sel45::Sm1pwm45)
    }
    #[doc = "Inverted generated SM1PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn inverted_sm1pwm45(self) -> &'a mut crate::W<REG> {
        self.variant(Sm1sel45::InvertedSm1pwm45)
    }
    #[doc = "SWCOUT\\[SM1OUT45\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn sm1out45(self) -> &'a mut crate::W<REG> {
        self.variant(Sm1sel45::Sm1out45)
    }
}
#[doc = "Submodule 1 PWM23 Control Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Sm1sel23 {
    #[doc = "0: Generated SM1PWM23 signal used by the deadtime logic."]
    Sm1pwm23 = 0,
    #[doc = "1: Inverted generated SM1PWM23 signal used by the deadtime logic."]
    InvertedSm1pwm23 = 1,
    #[doc = "2: SWCOUT\\[SM1OUT23\\] used by the deadtime logic."]
    Sm1out23 = 2,
    #[doc = "3: PWM1_EXTA signal used by the deadtime logic."]
    Pwm1Exta = 3,
}
impl From<Sm1sel23> for u8 {
    #[inline(always)]
    fn from(variant: Sm1sel23) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Sm1sel23 {
    type Ux = u8;
}
impl crate::IsEnum for Sm1sel23 {}
#[doc = "Field `SM1SEL23` reader - Submodule 1 PWM23 Control Select"]
pub type Sm1sel23R = crate::FieldReader<Sm1sel23>;
impl Sm1sel23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sm1sel23 {
        match self.bits {
            0 => Sm1sel23::Sm1pwm23,
            1 => Sm1sel23::InvertedSm1pwm23,
            2 => Sm1sel23::Sm1out23,
            3 => Sm1sel23::Pwm1Exta,
            _ => unreachable!(),
        }
    }
    #[doc = "Generated SM1PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm1pwm23(&self) -> bool {
        *self == Sm1sel23::Sm1pwm23
    }
    #[doc = "Inverted generated SM1PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_inverted_sm1pwm23(&self) -> bool {
        *self == Sm1sel23::InvertedSm1pwm23
    }
    #[doc = "SWCOUT\\[SM1OUT23\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm1out23(&self) -> bool {
        *self == Sm1sel23::Sm1out23
    }
    #[doc = "PWM1_EXTA signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_pwm1_exta(&self) -> bool {
        *self == Sm1sel23::Pwm1Exta
    }
}
#[doc = "Field `SM1SEL23` writer - Submodule 1 PWM23 Control Select"]
pub type Sm1sel23W<'a, REG> = crate::FieldWriter<'a, REG, 2, Sm1sel23, crate::Safe>;
impl<'a, REG> Sm1sel23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Generated SM1PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn sm1pwm23(self) -> &'a mut crate::W<REG> {
        self.variant(Sm1sel23::Sm1pwm23)
    }
    #[doc = "Inverted generated SM1PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn inverted_sm1pwm23(self) -> &'a mut crate::W<REG> {
        self.variant(Sm1sel23::InvertedSm1pwm23)
    }
    #[doc = "SWCOUT\\[SM1OUT23\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn sm1out23(self) -> &'a mut crate::W<REG> {
        self.variant(Sm1sel23::Sm1out23)
    }
    #[doc = "PWM1_EXTA signal used by the deadtime logic."]
    #[inline(always)]
    pub fn pwm1_exta(self) -> &'a mut crate::W<REG> {
        self.variant(Sm1sel23::Pwm1Exta)
    }
}
#[doc = "Submodule 2 PWM45 Control Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Sm2sel45 {
    #[doc = "0: Generated SM2PWM45 signal used by the deadtime logic."]
    Sm2pwm45 = 0,
    #[doc = "1: Inverted generated SM2PWM45 signal used by the deadtime logic."]
    InvertedSm2pwm45 = 1,
    #[doc = "2: SWCOUT\\[SM2OUT45\\] used by the deadtime logic."]
    Sm2out45 = 2,
}
impl From<Sm2sel45> for u8 {
    #[inline(always)]
    fn from(variant: Sm2sel45) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Sm2sel45 {
    type Ux = u8;
}
impl crate::IsEnum for Sm2sel45 {}
#[doc = "Field `SM2SEL45` reader - Submodule 2 PWM45 Control Select"]
pub type Sm2sel45R = crate::FieldReader<Sm2sel45>;
impl Sm2sel45R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Sm2sel45> {
        match self.bits {
            0 => Some(Sm2sel45::Sm2pwm45),
            1 => Some(Sm2sel45::InvertedSm2pwm45),
            2 => Some(Sm2sel45::Sm2out45),
            _ => None,
        }
    }
    #[doc = "Generated SM2PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm2pwm45(&self) -> bool {
        *self == Sm2sel45::Sm2pwm45
    }
    #[doc = "Inverted generated SM2PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_inverted_sm2pwm45(&self) -> bool {
        *self == Sm2sel45::InvertedSm2pwm45
    }
    #[doc = "SWCOUT\\[SM2OUT45\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm2out45(&self) -> bool {
        *self == Sm2sel45::Sm2out45
    }
}
#[doc = "Field `SM2SEL45` writer - Submodule 2 PWM45 Control Select"]
pub type Sm2sel45W<'a, REG> = crate::FieldWriter<'a, REG, 2, Sm2sel45>;
impl<'a, REG> Sm2sel45W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Generated SM2PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn sm2pwm45(self) -> &'a mut crate::W<REG> {
        self.variant(Sm2sel45::Sm2pwm45)
    }
    #[doc = "Inverted generated SM2PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn inverted_sm2pwm45(self) -> &'a mut crate::W<REG> {
        self.variant(Sm2sel45::InvertedSm2pwm45)
    }
    #[doc = "SWCOUT\\[SM2OUT45\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn sm2out45(self) -> &'a mut crate::W<REG> {
        self.variant(Sm2sel45::Sm2out45)
    }
}
#[doc = "Submodule 2 PWM23 Control Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Sm2sel23 {
    #[doc = "0: Generated SM2PWM23 signal used by the deadtime logic."]
    Sm2pwm23 = 0,
    #[doc = "1: Inverted generated SM2PWM23 signal used by the deadtime logic."]
    InvertedSm2pwm23 = 1,
    #[doc = "2: SWCOUT\\[SM2OUT23\\] used by the deadtime logic."]
    Sm2out23 = 2,
    #[doc = "3: PWM2_EXTA signal used by the deadtime logic."]
    Pwm2Exta = 3,
}
impl From<Sm2sel23> for u8 {
    #[inline(always)]
    fn from(variant: Sm2sel23) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Sm2sel23 {
    type Ux = u8;
}
impl crate::IsEnum for Sm2sel23 {}
#[doc = "Field `SM2SEL23` reader - Submodule 2 PWM23 Control Select"]
pub type Sm2sel23R = crate::FieldReader<Sm2sel23>;
impl Sm2sel23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sm2sel23 {
        match self.bits {
            0 => Sm2sel23::Sm2pwm23,
            1 => Sm2sel23::InvertedSm2pwm23,
            2 => Sm2sel23::Sm2out23,
            3 => Sm2sel23::Pwm2Exta,
            _ => unreachable!(),
        }
    }
    #[doc = "Generated SM2PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm2pwm23(&self) -> bool {
        *self == Sm2sel23::Sm2pwm23
    }
    #[doc = "Inverted generated SM2PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_inverted_sm2pwm23(&self) -> bool {
        *self == Sm2sel23::InvertedSm2pwm23
    }
    #[doc = "SWCOUT\\[SM2OUT23\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm2out23(&self) -> bool {
        *self == Sm2sel23::Sm2out23
    }
    #[doc = "PWM2_EXTA signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_pwm2_exta(&self) -> bool {
        *self == Sm2sel23::Pwm2Exta
    }
}
#[doc = "Field `SM2SEL23` writer - Submodule 2 PWM23 Control Select"]
pub type Sm2sel23W<'a, REG> = crate::FieldWriter<'a, REG, 2, Sm2sel23, crate::Safe>;
impl<'a, REG> Sm2sel23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Generated SM2PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn sm2pwm23(self) -> &'a mut crate::W<REG> {
        self.variant(Sm2sel23::Sm2pwm23)
    }
    #[doc = "Inverted generated SM2PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn inverted_sm2pwm23(self) -> &'a mut crate::W<REG> {
        self.variant(Sm2sel23::InvertedSm2pwm23)
    }
    #[doc = "SWCOUT\\[SM2OUT23\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn sm2out23(self) -> &'a mut crate::W<REG> {
        self.variant(Sm2sel23::Sm2out23)
    }
    #[doc = "PWM2_EXTA signal used by the deadtime logic."]
    #[inline(always)]
    pub fn pwm2_exta(self) -> &'a mut crate::W<REG> {
        self.variant(Sm2sel23::Pwm2Exta)
    }
}
#[doc = "Submodule 3 PWM45 Control Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Sm3sel45 {
    #[doc = "0: Generated SM3PWM45 signal used by the deadtime logic."]
    Sm3pwm45 = 0,
    #[doc = "1: Inverted generated SM3PWM45 signal used by the deadtime logic."]
    InvertedSm3pwm45 = 1,
    #[doc = "2: SWCOUT\\[SM3OUT45\\] used by the deadtime logic."]
    Sm3out45 = 2,
}
impl From<Sm3sel45> for u8 {
    #[inline(always)]
    fn from(variant: Sm3sel45) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Sm3sel45 {
    type Ux = u8;
}
impl crate::IsEnum for Sm3sel45 {}
#[doc = "Field `SM3SEL45` reader - Submodule 3 PWM45 Control Select"]
pub type Sm3sel45R = crate::FieldReader<Sm3sel45>;
impl Sm3sel45R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Sm3sel45> {
        match self.bits {
            0 => Some(Sm3sel45::Sm3pwm45),
            1 => Some(Sm3sel45::InvertedSm3pwm45),
            2 => Some(Sm3sel45::Sm3out45),
            _ => None,
        }
    }
    #[doc = "Generated SM3PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm3pwm45(&self) -> bool {
        *self == Sm3sel45::Sm3pwm45
    }
    #[doc = "Inverted generated SM3PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_inverted_sm3pwm45(&self) -> bool {
        *self == Sm3sel45::InvertedSm3pwm45
    }
    #[doc = "SWCOUT\\[SM3OUT45\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm3out45(&self) -> bool {
        *self == Sm3sel45::Sm3out45
    }
}
#[doc = "Field `SM3SEL45` writer - Submodule 3 PWM45 Control Select"]
pub type Sm3sel45W<'a, REG> = crate::FieldWriter<'a, REG, 2, Sm3sel45>;
impl<'a, REG> Sm3sel45W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Generated SM3PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn sm3pwm45(self) -> &'a mut crate::W<REG> {
        self.variant(Sm3sel45::Sm3pwm45)
    }
    #[doc = "Inverted generated SM3PWM45 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn inverted_sm3pwm45(self) -> &'a mut crate::W<REG> {
        self.variant(Sm3sel45::InvertedSm3pwm45)
    }
    #[doc = "SWCOUT\\[SM3OUT45\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn sm3out45(self) -> &'a mut crate::W<REG> {
        self.variant(Sm3sel45::Sm3out45)
    }
}
#[doc = "Submodule 3 PWM23 Control Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Sm3sel23 {
    #[doc = "0: Generated SM3PWM23 signal used by the deadtime logic."]
    Sm3pwm23 = 0,
    #[doc = "1: Inverted generated SM3PWM23 signal used by the deadtime logic."]
    InvertedSm3pwm23 = 1,
    #[doc = "2: SWCOUT\\[SM3OUT23\\] used by the deadtime logic."]
    Sm3out23 = 2,
    #[doc = "3: PWM3_EXTA signal used by the deadtime logic."]
    Pwm3Exta = 3,
}
impl From<Sm3sel23> for u8 {
    #[inline(always)]
    fn from(variant: Sm3sel23) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Sm3sel23 {
    type Ux = u8;
}
impl crate::IsEnum for Sm3sel23 {}
#[doc = "Field `SM3SEL23` reader - Submodule 3 PWM23 Control Select"]
pub type Sm3sel23R = crate::FieldReader<Sm3sel23>;
impl Sm3sel23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sm3sel23 {
        match self.bits {
            0 => Sm3sel23::Sm3pwm23,
            1 => Sm3sel23::InvertedSm3pwm23,
            2 => Sm3sel23::Sm3out23,
            3 => Sm3sel23::Pwm3Exta,
            _ => unreachable!(),
        }
    }
    #[doc = "Generated SM3PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm3pwm23(&self) -> bool {
        *self == Sm3sel23::Sm3pwm23
    }
    #[doc = "Inverted generated SM3PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_inverted_sm3pwm23(&self) -> bool {
        *self == Sm3sel23::InvertedSm3pwm23
    }
    #[doc = "SWCOUT\\[SM3OUT23\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn is_sm3out23(&self) -> bool {
        *self == Sm3sel23::Sm3out23
    }
    #[doc = "PWM3_EXTA signal used by the deadtime logic."]
    #[inline(always)]
    pub fn is_pwm3_exta(&self) -> bool {
        *self == Sm3sel23::Pwm3Exta
    }
}
#[doc = "Field `SM3SEL23` writer - Submodule 3 PWM23 Control Select"]
pub type Sm3sel23W<'a, REG> = crate::FieldWriter<'a, REG, 2, Sm3sel23, crate::Safe>;
impl<'a, REG> Sm3sel23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Generated SM3PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn sm3pwm23(self) -> &'a mut crate::W<REG> {
        self.variant(Sm3sel23::Sm3pwm23)
    }
    #[doc = "Inverted generated SM3PWM23 signal used by the deadtime logic."]
    #[inline(always)]
    pub fn inverted_sm3pwm23(self) -> &'a mut crate::W<REG> {
        self.variant(Sm3sel23::InvertedSm3pwm23)
    }
    #[doc = "SWCOUT\\[SM3OUT23\\] used by the deadtime logic."]
    #[inline(always)]
    pub fn sm3out23(self) -> &'a mut crate::W<REG> {
        self.variant(Sm3sel23::Sm3out23)
    }
    #[doc = "PWM3_EXTA signal used by the deadtime logic."]
    #[inline(always)]
    pub fn pwm3_exta(self) -> &'a mut crate::W<REG> {
        self.variant(Sm3sel23::Pwm3Exta)
    }
}
impl R {
    #[doc = "Bits 0:1 - Submodule 0 PWM45 Control Select"]
    #[inline(always)]
    pub fn sm0sel45(&self) -> Sm0sel45R {
        Sm0sel45R::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 2:3 - Submodule 0 PWM23 Control Select"]
    #[inline(always)]
    pub fn sm0sel23(&self) -> Sm0sel23R {
        Sm0sel23R::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:5 - Submodule 1 PWM45 Control Select"]
    #[inline(always)]
    pub fn sm1sel45(&self) -> Sm1sel45R {
        Sm1sel45R::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 6:7 - Submodule 1 PWM23 Control Select"]
    #[inline(always)]
    pub fn sm1sel23(&self) -> Sm1sel23R {
        Sm1sel23R::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bits 8:9 - Submodule 2 PWM45 Control Select"]
    #[inline(always)]
    pub fn sm2sel45(&self) -> Sm2sel45R {
        Sm2sel45R::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bits 10:11 - Submodule 2 PWM23 Control Select"]
    #[inline(always)]
    pub fn sm2sel23(&self) -> Sm2sel23R {
        Sm2sel23R::new(((self.bits >> 10) & 3) as u8)
    }
    #[doc = "Bits 12:13 - Submodule 3 PWM45 Control Select"]
    #[inline(always)]
    pub fn sm3sel45(&self) -> Sm3sel45R {
        Sm3sel45R::new(((self.bits >> 12) & 3) as u8)
    }
    #[doc = "Bits 14:15 - Submodule 3 PWM23 Control Select"]
    #[inline(always)]
    pub fn sm3sel23(&self) -> Sm3sel23R {
        Sm3sel23R::new(((self.bits >> 14) & 3) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Submodule 0 PWM45 Control Select"]
    #[inline(always)]
    pub fn sm0sel45(&mut self) -> Sm0sel45W<DtsrcselSpec> {
        Sm0sel45W::new(self, 0)
    }
    #[doc = "Bits 2:3 - Submodule 0 PWM23 Control Select"]
    #[inline(always)]
    pub fn sm0sel23(&mut self) -> Sm0sel23W<DtsrcselSpec> {
        Sm0sel23W::new(self, 2)
    }
    #[doc = "Bits 4:5 - Submodule 1 PWM45 Control Select"]
    #[inline(always)]
    pub fn sm1sel45(&mut self) -> Sm1sel45W<DtsrcselSpec> {
        Sm1sel45W::new(self, 4)
    }
    #[doc = "Bits 6:7 - Submodule 1 PWM23 Control Select"]
    #[inline(always)]
    pub fn sm1sel23(&mut self) -> Sm1sel23W<DtsrcselSpec> {
        Sm1sel23W::new(self, 6)
    }
    #[doc = "Bits 8:9 - Submodule 2 PWM45 Control Select"]
    #[inline(always)]
    pub fn sm2sel45(&mut self) -> Sm2sel45W<DtsrcselSpec> {
        Sm2sel45W::new(self, 8)
    }
    #[doc = "Bits 10:11 - Submodule 2 PWM23 Control Select"]
    #[inline(always)]
    pub fn sm2sel23(&mut self) -> Sm2sel23W<DtsrcselSpec> {
        Sm2sel23W::new(self, 10)
    }
    #[doc = "Bits 12:13 - Submodule 3 PWM45 Control Select"]
    #[inline(always)]
    pub fn sm3sel45(&mut self) -> Sm3sel45W<DtsrcselSpec> {
        Sm3sel45W::new(self, 12)
    }
    #[doc = "Bits 14:15 - Submodule 3 PWM23 Control Select"]
    #[inline(always)]
    pub fn sm3sel23(&mut self) -> Sm3sel23W<DtsrcselSpec> {
        Sm3sel23W::new(self, 14)
    }
}
#[doc = "PWM Source Select Register\n\nYou can [`read`](crate::Reg::read) this register and get [`dtsrcsel::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`dtsrcsel::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DtsrcselSpec;
impl crate::RegisterSpec for DtsrcselSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`dtsrcsel::R`](R) reader structure"]
impl crate::Readable for DtsrcselSpec {}
#[doc = "`write(|w| ..)` method takes [`dtsrcsel::W`](W) writer structure"]
impl crate::Writable for DtsrcselSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DTSRCSEL to value 0"]
impl crate::Resettable for DtsrcselSpec {}

#[doc = "Register `TER` reader"]
pub type R = crate::R<TerSpec>;
#[doc = "Register `TER` writer"]
pub type W = crate::W<TerSpec>;
#[doc = "Tamper Input Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tie0 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tie0> for bool {
    #[inline(always)]
    fn from(variant: Tie0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIE0` reader - Tamper Input Enable"]
pub type Tie0R = crate::BitReader<Tie0>;
impl Tie0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tie0 {
        match self.bits {
            false => Tie0::Disable,
            true => Tie0::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tie0::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tie0::Enable
    }
}
#[doc = "Field `TIE0` writer - Tamper Input Enable"]
pub type Tie0W<'a, REG> = crate::BitWriter<'a, REG, Tie0>;
impl<'a, REG> Tie0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie0::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie0::Enable)
    }
}
#[doc = "Tamper Input Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tie1 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tie1> for bool {
    #[inline(always)]
    fn from(variant: Tie1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIE1` reader - Tamper Input Enable"]
pub type Tie1R = crate::BitReader<Tie1>;
impl Tie1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tie1 {
        match self.bits {
            false => Tie1::Disable,
            true => Tie1::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tie1::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tie1::Enable
    }
}
#[doc = "Field `TIE1` writer - Tamper Input Enable"]
pub type Tie1W<'a, REG> = crate::BitWriter<'a, REG, Tie1>;
impl<'a, REG> Tie1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie1::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie1::Enable)
    }
}
#[doc = "Tamper Input Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tie2 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tie2> for bool {
    #[inline(always)]
    fn from(variant: Tie2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIE2` reader - Tamper Input Enable"]
pub type Tie2R = crate::BitReader<Tie2>;
impl Tie2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tie2 {
        match self.bits {
            false => Tie2::Disable,
            true => Tie2::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tie2::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tie2::Enable
    }
}
#[doc = "Field `TIE2` writer - Tamper Input Enable"]
pub type Tie2W<'a, REG> = crate::BitWriter<'a, REG, Tie2>;
impl<'a, REG> Tie2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie2::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie2::Enable)
    }
}
#[doc = "Tamper Input Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tie3 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tie3> for bool {
    #[inline(always)]
    fn from(variant: Tie3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIE3` reader - Tamper Input Enable"]
pub type Tie3R = crate::BitReader<Tie3>;
impl Tie3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tie3 {
        match self.bits {
            false => Tie3::Disable,
            true => Tie3::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tie3::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tie3::Enable
    }
}
#[doc = "Field `TIE3` writer - Tamper Input Enable"]
pub type Tie3W<'a, REG> = crate::BitWriter<'a, REG, Tie3>;
impl<'a, REG> Tie3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie3::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie3::Enable)
    }
}
#[doc = "Tamper Input Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tie4 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tie4> for bool {
    #[inline(always)]
    fn from(variant: Tie4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIE4` reader - Tamper Input Enable"]
pub type Tie4R = crate::BitReader<Tie4>;
impl Tie4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tie4 {
        match self.bits {
            false => Tie4::Disable,
            true => Tie4::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tie4::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tie4::Enable
    }
}
#[doc = "Field `TIE4` writer - Tamper Input Enable"]
pub type Tie4W<'a, REG> = crate::BitWriter<'a, REG, Tie4>;
impl<'a, REG> Tie4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie4::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie4::Enable)
    }
}
#[doc = "Tamper Input Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tie5 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tie5> for bool {
    #[inline(always)]
    fn from(variant: Tie5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIE5` reader - Tamper Input Enable"]
pub type Tie5R = crate::BitReader<Tie5>;
impl Tie5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tie5 {
        match self.bits {
            false => Tie5::Disable,
            true => Tie5::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tie5::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tie5::Enable
    }
}
#[doc = "Field `TIE5` writer - Tamper Input Enable"]
pub type Tie5W<'a, REG> = crate::BitWriter<'a, REG, Tie5>;
impl<'a, REG> Tie5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie5::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie5::Enable)
    }
}
#[doc = "Tamper Input Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tie6 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tie6> for bool {
    #[inline(always)]
    fn from(variant: Tie6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIE6` reader - Tamper Input Enable"]
pub type Tie6R = crate::BitReader<Tie6>;
impl Tie6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tie6 {
        match self.bits {
            false => Tie6::Disable,
            true => Tie6::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tie6::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tie6::Enable
    }
}
#[doc = "Field `TIE6` writer - Tamper Input Enable"]
pub type Tie6W<'a, REG> = crate::BitWriter<'a, REG, Tie6>;
impl<'a, REG> Tie6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie6::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie6::Enable)
    }
}
#[doc = "Tamper Input Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tie7 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tie7> for bool {
    #[inline(always)]
    fn from(variant: Tie7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIE7` reader - Tamper Input Enable"]
pub type Tie7R = crate::BitReader<Tie7>;
impl Tie7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tie7 {
        match self.bits {
            false => Tie7::Disable,
            true => Tie7::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tie7::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tie7::Enable
    }
}
#[doc = "Field `TIE7` writer - Tamper Input Enable"]
pub type Tie7W<'a, REG> = crate::BitWriter<'a, REG, Tie7>;
impl<'a, REG> Tie7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie7::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie7::Enable)
    }
}
#[doc = "Tamper Input Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tie8 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tie8> for bool {
    #[inline(always)]
    fn from(variant: Tie8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIE8` reader - Tamper Input Enable"]
pub type Tie8R = crate::BitReader<Tie8>;
impl Tie8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tie8 {
        match self.bits {
            false => Tie8::Disable,
            true => Tie8::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tie8::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tie8::Enable
    }
}
#[doc = "Field `TIE8` writer - Tamper Input Enable"]
pub type Tie8W<'a, REG> = crate::BitWriter<'a, REG, Tie8>;
impl<'a, REG> Tie8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie8::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie8::Enable)
    }
}
#[doc = "Tamper Input Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tie9 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tie9> for bool {
    #[inline(always)]
    fn from(variant: Tie9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIE9` reader - Tamper Input Enable"]
pub type Tie9R = crate::BitReader<Tie9>;
impl Tie9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tie9 {
        match self.bits {
            false => Tie9::Disable,
            true => Tie9::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tie9::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tie9::Enable
    }
}
#[doc = "Field `TIE9` writer - Tamper Input Enable"]
pub type Tie9W<'a, REG> = crate::BitWriter<'a, REG, Tie9>;
impl<'a, REG> Tie9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie9::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie9::Enable)
    }
}
#[doc = "Tamper Input Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tie10 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tie10> for bool {
    #[inline(always)]
    fn from(variant: Tie10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIE10` reader - Tamper Input Enable"]
pub type Tie10R = crate::BitReader<Tie10>;
impl Tie10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tie10 {
        match self.bits {
            false => Tie10::Disable,
            true => Tie10::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tie10::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tie10::Enable
    }
}
#[doc = "Field `TIE10` writer - Tamper Input Enable"]
pub type Tie10W<'a, REG> = crate::BitWriter<'a, REG, Tie10>;
impl<'a, REG> Tie10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie10::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tie10::Enable)
    }
}
#[doc = "Tamper Pin Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpe0 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tpe0> for bool {
    #[inline(always)]
    fn from(variant: Tpe0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPE0` reader - Tamper Pin Enable"]
pub type Tpe0R = crate::BitReader<Tpe0>;
impl Tpe0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpe0 {
        match self.bits {
            false => Tpe0::Disable,
            true => Tpe0::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tpe0::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tpe0::Enable
    }
}
#[doc = "Field `TPE0` writer - Tamper Pin Enable"]
pub type Tpe0W<'a, REG> = crate::BitWriter<'a, REG, Tpe0>;
impl<'a, REG> Tpe0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpe0::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpe0::Enable)
    }
}
#[doc = "Tamper Pin Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpe1 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tpe1> for bool {
    #[inline(always)]
    fn from(variant: Tpe1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPE1` reader - Tamper Pin Enable"]
pub type Tpe1R = crate::BitReader<Tpe1>;
impl Tpe1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpe1 {
        match self.bits {
            false => Tpe1::Disable,
            true => Tpe1::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tpe1::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tpe1::Enable
    }
}
#[doc = "Field `TPE1` writer - Tamper Pin Enable"]
pub type Tpe1W<'a, REG> = crate::BitWriter<'a, REG, Tpe1>;
impl<'a, REG> Tpe1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpe1::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpe1::Enable)
    }
}
#[doc = "Tamper Pin Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpe2 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tpe2> for bool {
    #[inline(always)]
    fn from(variant: Tpe2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPE2` reader - Tamper Pin Enable"]
pub type Tpe2R = crate::BitReader<Tpe2>;
impl Tpe2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpe2 {
        match self.bits {
            false => Tpe2::Disable,
            true => Tpe2::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tpe2::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tpe2::Enable
    }
}
#[doc = "Field `TPE2` writer - Tamper Pin Enable"]
pub type Tpe2W<'a, REG> = crate::BitWriter<'a, REG, Tpe2>;
impl<'a, REG> Tpe2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpe2::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpe2::Enable)
    }
}
#[doc = "Tamper Pin Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpe3 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tpe3> for bool {
    #[inline(always)]
    fn from(variant: Tpe3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPE3` reader - Tamper Pin Enable"]
pub type Tpe3R = crate::BitReader<Tpe3>;
impl Tpe3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpe3 {
        match self.bits {
            false => Tpe3::Disable,
            true => Tpe3::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tpe3::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tpe3::Enable
    }
}
#[doc = "Field `TPE3` writer - Tamper Pin Enable"]
pub type Tpe3W<'a, REG> = crate::BitWriter<'a, REG, Tpe3>;
impl<'a, REG> Tpe3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpe3::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpe3::Enable)
    }
}
#[doc = "Tamper Pin Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpe4 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tpe4> for bool {
    #[inline(always)]
    fn from(variant: Tpe4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPE4` reader - Tamper Pin Enable"]
pub type Tpe4R = crate::BitReader<Tpe4>;
impl Tpe4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpe4 {
        match self.bits {
            false => Tpe4::Disable,
            true => Tpe4::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tpe4::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tpe4::Enable
    }
}
#[doc = "Field `TPE4` writer - Tamper Pin Enable"]
pub type Tpe4W<'a, REG> = crate::BitWriter<'a, REG, Tpe4>;
impl<'a, REG> Tpe4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpe4::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpe4::Enable)
    }
}
#[doc = "Tamper Pin Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpe5 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tpe5> for bool {
    #[inline(always)]
    fn from(variant: Tpe5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPE5` reader - Tamper Pin Enable"]
pub type Tpe5R = crate::BitReader<Tpe5>;
impl Tpe5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpe5 {
        match self.bits {
            false => Tpe5::Disable,
            true => Tpe5::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tpe5::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tpe5::Enable
    }
}
#[doc = "Field `TPE5` writer - Tamper Pin Enable"]
pub type Tpe5W<'a, REG> = crate::BitWriter<'a, REG, Tpe5>;
impl<'a, REG> Tpe5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpe5::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpe5::Enable)
    }
}
impl R {
    #[doc = "Bit 2 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie0(&self) -> Tie0R {
        Tie0R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie1(&self) -> Tie1R {
        Tie1R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie2(&self) -> Tie2R {
        Tie2R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie3(&self) -> Tie3R {
        Tie3R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie4(&self) -> Tie4R {
        Tie4R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie5(&self) -> Tie5R {
        Tie5R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie6(&self) -> Tie6R {
        Tie6R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie7(&self) -> Tie7R {
        Tie7R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie8(&self) -> Tie8R {
        Tie8R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie9(&self) -> Tie9R {
        Tie9R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie10(&self) -> Tie10R {
        Tie10R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 16 - Tamper Pin Enable"]
    #[inline(always)]
    pub fn tpe0(&self) -> Tpe0R {
        Tpe0R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Tamper Pin Enable"]
    #[inline(always)]
    pub fn tpe1(&self) -> Tpe1R {
        Tpe1R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Tamper Pin Enable"]
    #[inline(always)]
    pub fn tpe2(&self) -> Tpe2R {
        Tpe2R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Tamper Pin Enable"]
    #[inline(always)]
    pub fn tpe3(&self) -> Tpe3R {
        Tpe3R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Tamper Pin Enable"]
    #[inline(always)]
    pub fn tpe4(&self) -> Tpe4R {
        Tpe4R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Tamper Pin Enable"]
    #[inline(always)]
    pub fn tpe5(&self) -> Tpe5R {
        Tpe5R::new(((self.bits >> 21) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 2 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie0(&mut self) -> Tie0W<TerSpec> {
        Tie0W::new(self, 2)
    }
    #[doc = "Bit 3 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie1(&mut self) -> Tie1W<TerSpec> {
        Tie1W::new(self, 3)
    }
    #[doc = "Bit 4 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie2(&mut self) -> Tie2W<TerSpec> {
        Tie2W::new(self, 4)
    }
    #[doc = "Bit 5 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie3(&mut self) -> Tie3W<TerSpec> {
        Tie3W::new(self, 5)
    }
    #[doc = "Bit 6 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie4(&mut self) -> Tie4W<TerSpec> {
        Tie4W::new(self, 6)
    }
    #[doc = "Bit 7 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie5(&mut self) -> Tie5W<TerSpec> {
        Tie5W::new(self, 7)
    }
    #[doc = "Bit 8 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie6(&mut self) -> Tie6W<TerSpec> {
        Tie6W::new(self, 8)
    }
    #[doc = "Bit 9 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie7(&mut self) -> Tie7W<TerSpec> {
        Tie7W::new(self, 9)
    }
    #[doc = "Bit 10 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie8(&mut self) -> Tie8W<TerSpec> {
        Tie8W::new(self, 10)
    }
    #[doc = "Bit 11 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie9(&mut self) -> Tie9W<TerSpec> {
        Tie9W::new(self, 11)
    }
    #[doc = "Bit 12 - Tamper Input Enable"]
    #[inline(always)]
    pub fn tie10(&mut self) -> Tie10W<TerSpec> {
        Tie10W::new(self, 12)
    }
    #[doc = "Bit 16 - Tamper Pin Enable"]
    #[inline(always)]
    pub fn tpe0(&mut self) -> Tpe0W<TerSpec> {
        Tpe0W::new(self, 16)
    }
    #[doc = "Bit 17 - Tamper Pin Enable"]
    #[inline(always)]
    pub fn tpe1(&mut self) -> Tpe1W<TerSpec> {
        Tpe1W::new(self, 17)
    }
    #[doc = "Bit 18 - Tamper Pin Enable"]
    #[inline(always)]
    pub fn tpe2(&mut self) -> Tpe2W<TerSpec> {
        Tpe2W::new(self, 18)
    }
    #[doc = "Bit 19 - Tamper Pin Enable"]
    #[inline(always)]
    pub fn tpe3(&mut self) -> Tpe3W<TerSpec> {
        Tpe3W::new(self, 19)
    }
    #[doc = "Bit 20 - Tamper Pin Enable"]
    #[inline(always)]
    pub fn tpe4(&mut self) -> Tpe4W<TerSpec> {
        Tpe4W::new(self, 20)
    }
    #[doc = "Bit 21 - Tamper Pin Enable"]
    #[inline(always)]
    pub fn tpe5(&mut self) -> Tpe5W<TerSpec> {
        Tpe5W::new(self, 21)
    }
}
#[doc = "Tamper Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ter::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ter::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TerSpec;
impl crate::RegisterSpec for TerSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ter::R`](R) reader structure"]
impl crate::Readable for TerSpec {}
#[doc = "`write(|w| ..)` method takes [`ter::W`](W) writer structure"]
impl crate::Writable for TerSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TER to value 0"]
impl crate::Resettable for TerSpec {}

#[doc = "Register `IER` reader"]
pub type R = crate::R<IerSpec>;
#[doc = "Register `IER` writer"]
pub type W = crate::W<IerSpec>;
#[doc = "Digital Tamper Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dtie {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Dtie> for bool {
    #[inline(always)]
    fn from(variant: Dtie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DTIE` reader - Digital Tamper Interrupt Enable"]
pub type DtieR = crate::BitReader<Dtie>;
impl DtieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dtie {
        match self.bits {
            false => Dtie::Disable,
            true => Dtie::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Dtie::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Dtie::Enable
    }
}
#[doc = "Field `DTIE` writer - Digital Tamper Interrupt Enable"]
pub type DtieW<'a, REG> = crate::BitWriter<'a, REG, Dtie>;
impl<'a, REG> DtieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Dtie::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Dtie::Enable)
    }
}
#[doc = "Tamper Input n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tiie0 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tiie0> for bool {
    #[inline(always)]
    fn from(variant: Tiie0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIIE0` reader - Tamper Input n Interrupt Enable"]
pub type Tiie0R = crate::BitReader<Tiie0>;
impl Tiie0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tiie0 {
        match self.bits {
            false => Tiie0::Disable,
            true => Tiie0::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tiie0::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tiie0::Enable
    }
}
#[doc = "Field `TIIE0` writer - Tamper Input n Interrupt Enable"]
pub type Tiie0W<'a, REG> = crate::BitWriter<'a, REG, Tiie0>;
impl<'a, REG> Tiie0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie0::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie0::Enable)
    }
}
#[doc = "Tamper Input n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tiie1 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tiie1> for bool {
    #[inline(always)]
    fn from(variant: Tiie1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIIE1` reader - Tamper Input n Interrupt Enable"]
pub type Tiie1R = crate::BitReader<Tiie1>;
impl Tiie1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tiie1 {
        match self.bits {
            false => Tiie1::Disable,
            true => Tiie1::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tiie1::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tiie1::Enable
    }
}
#[doc = "Field `TIIE1` writer - Tamper Input n Interrupt Enable"]
pub type Tiie1W<'a, REG> = crate::BitWriter<'a, REG, Tiie1>;
impl<'a, REG> Tiie1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie1::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie1::Enable)
    }
}
#[doc = "Tamper Input n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tiie2 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tiie2> for bool {
    #[inline(always)]
    fn from(variant: Tiie2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIIE2` reader - Tamper Input n Interrupt Enable"]
pub type Tiie2R = crate::BitReader<Tiie2>;
impl Tiie2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tiie2 {
        match self.bits {
            false => Tiie2::Disable,
            true => Tiie2::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tiie2::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tiie2::Enable
    }
}
#[doc = "Field `TIIE2` writer - Tamper Input n Interrupt Enable"]
pub type Tiie2W<'a, REG> = crate::BitWriter<'a, REG, Tiie2>;
impl<'a, REG> Tiie2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie2::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie2::Enable)
    }
}
#[doc = "Tamper Input n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tiie3 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tiie3> for bool {
    #[inline(always)]
    fn from(variant: Tiie3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIIE3` reader - Tamper Input n Interrupt Enable"]
pub type Tiie3R = crate::BitReader<Tiie3>;
impl Tiie3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tiie3 {
        match self.bits {
            false => Tiie3::Disable,
            true => Tiie3::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tiie3::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tiie3::Enable
    }
}
#[doc = "Field `TIIE3` writer - Tamper Input n Interrupt Enable"]
pub type Tiie3W<'a, REG> = crate::BitWriter<'a, REG, Tiie3>;
impl<'a, REG> Tiie3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie3::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie3::Enable)
    }
}
#[doc = "Tamper Input n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tiie4 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tiie4> for bool {
    #[inline(always)]
    fn from(variant: Tiie4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIIE4` reader - Tamper Input n Interrupt Enable"]
pub type Tiie4R = crate::BitReader<Tiie4>;
impl Tiie4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tiie4 {
        match self.bits {
            false => Tiie4::Disable,
            true => Tiie4::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tiie4::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tiie4::Enable
    }
}
#[doc = "Field `TIIE4` writer - Tamper Input n Interrupt Enable"]
pub type Tiie4W<'a, REG> = crate::BitWriter<'a, REG, Tiie4>;
impl<'a, REG> Tiie4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie4::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie4::Enable)
    }
}
#[doc = "Tamper Input n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tiie5 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tiie5> for bool {
    #[inline(always)]
    fn from(variant: Tiie5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIIE5` reader - Tamper Input n Interrupt Enable"]
pub type Tiie5R = crate::BitReader<Tiie5>;
impl Tiie5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tiie5 {
        match self.bits {
            false => Tiie5::Disable,
            true => Tiie5::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tiie5::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tiie5::Enable
    }
}
#[doc = "Field `TIIE5` writer - Tamper Input n Interrupt Enable"]
pub type Tiie5W<'a, REG> = crate::BitWriter<'a, REG, Tiie5>;
impl<'a, REG> Tiie5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie5::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie5::Enable)
    }
}
#[doc = "Tamper Input n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tiie6 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tiie6> for bool {
    #[inline(always)]
    fn from(variant: Tiie6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIIE6` reader - Tamper Input n Interrupt Enable"]
pub type Tiie6R = crate::BitReader<Tiie6>;
impl Tiie6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tiie6 {
        match self.bits {
            false => Tiie6::Disable,
            true => Tiie6::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tiie6::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tiie6::Enable
    }
}
#[doc = "Field `TIIE6` writer - Tamper Input n Interrupt Enable"]
pub type Tiie6W<'a, REG> = crate::BitWriter<'a, REG, Tiie6>;
impl<'a, REG> Tiie6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie6::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie6::Enable)
    }
}
#[doc = "Tamper Input n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tiie7 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tiie7> for bool {
    #[inline(always)]
    fn from(variant: Tiie7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIIE7` reader - Tamper Input n Interrupt Enable"]
pub type Tiie7R = crate::BitReader<Tiie7>;
impl Tiie7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tiie7 {
        match self.bits {
            false => Tiie7::Disable,
            true => Tiie7::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tiie7::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tiie7::Enable
    }
}
#[doc = "Field `TIIE7` writer - Tamper Input n Interrupt Enable"]
pub type Tiie7W<'a, REG> = crate::BitWriter<'a, REG, Tiie7>;
impl<'a, REG> Tiie7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie7::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie7::Enable)
    }
}
#[doc = "Tamper Input n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tiie8 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tiie8> for bool {
    #[inline(always)]
    fn from(variant: Tiie8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIIE8` reader - Tamper Input n Interrupt Enable"]
pub type Tiie8R = crate::BitReader<Tiie8>;
impl Tiie8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tiie8 {
        match self.bits {
            false => Tiie8::Disable,
            true => Tiie8::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tiie8::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tiie8::Enable
    }
}
#[doc = "Field `TIIE8` writer - Tamper Input n Interrupt Enable"]
pub type Tiie8W<'a, REG> = crate::BitWriter<'a, REG, Tiie8>;
impl<'a, REG> Tiie8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie8::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie8::Enable)
    }
}
#[doc = "Tamper Input n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tiie9 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tiie9> for bool {
    #[inline(always)]
    fn from(variant: Tiie9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIIE9` reader - Tamper Input n Interrupt Enable"]
pub type Tiie9R = crate::BitReader<Tiie9>;
impl Tiie9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tiie9 {
        match self.bits {
            false => Tiie9::Disable,
            true => Tiie9::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tiie9::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tiie9::Enable
    }
}
#[doc = "Field `TIIE9` writer - Tamper Input n Interrupt Enable"]
pub type Tiie9W<'a, REG> = crate::BitWriter<'a, REG, Tiie9>;
impl<'a, REG> Tiie9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie9::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie9::Enable)
    }
}
#[doc = "Tamper Input n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tiie10 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tiie10> for bool {
    #[inline(always)]
    fn from(variant: Tiie10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIIE10` reader - Tamper Input n Interrupt Enable"]
pub type Tiie10R = crate::BitReader<Tiie10>;
impl Tiie10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tiie10 {
        match self.bits {
            false => Tiie10::Disable,
            true => Tiie10::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tiie10::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tiie10::Enable
    }
}
#[doc = "Field `TIIE10` writer - Tamper Input n Interrupt Enable"]
pub type Tiie10W<'a, REG> = crate::BitWriter<'a, REG, Tiie10>;
impl<'a, REG> Tiie10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie10::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie10::Enable)
    }
}
#[doc = "Tamper Pin n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpie0 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tpie0> for bool {
    #[inline(always)]
    fn from(variant: Tpie0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPIE0` reader - Tamper Pin n Interrupt Enable"]
pub type Tpie0R = crate::BitReader<Tpie0>;
impl Tpie0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpie0 {
        match self.bits {
            false => Tpie0::Disable,
            true => Tpie0::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tpie0::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tpie0::Enable
    }
}
#[doc = "Field `TPIE0` writer - Tamper Pin n Interrupt Enable"]
pub type Tpie0W<'a, REG> = crate::BitWriter<'a, REG, Tpie0>;
impl<'a, REG> Tpie0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpie0::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpie0::Enable)
    }
}
#[doc = "Tamper Pin n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpie1 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tpie1> for bool {
    #[inline(always)]
    fn from(variant: Tpie1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPIE1` reader - Tamper Pin n Interrupt Enable"]
pub type Tpie1R = crate::BitReader<Tpie1>;
impl Tpie1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpie1 {
        match self.bits {
            false => Tpie1::Disable,
            true => Tpie1::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tpie1::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tpie1::Enable
    }
}
#[doc = "Field `TPIE1` writer - Tamper Pin n Interrupt Enable"]
pub type Tpie1W<'a, REG> = crate::BitWriter<'a, REG, Tpie1>;
impl<'a, REG> Tpie1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpie1::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpie1::Enable)
    }
}
#[doc = "Tamper Pin n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpie2 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tpie2> for bool {
    #[inline(always)]
    fn from(variant: Tpie2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPIE2` reader - Tamper Pin n Interrupt Enable"]
pub type Tpie2R = crate::BitReader<Tpie2>;
impl Tpie2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpie2 {
        match self.bits {
            false => Tpie2::Disable,
            true => Tpie2::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tpie2::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tpie2::Enable
    }
}
#[doc = "Field `TPIE2` writer - Tamper Pin n Interrupt Enable"]
pub type Tpie2W<'a, REG> = crate::BitWriter<'a, REG, Tpie2>;
impl<'a, REG> Tpie2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpie2::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpie2::Enable)
    }
}
#[doc = "Tamper Pin n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpie3 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tpie3> for bool {
    #[inline(always)]
    fn from(variant: Tpie3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPIE3` reader - Tamper Pin n Interrupt Enable"]
pub type Tpie3R = crate::BitReader<Tpie3>;
impl Tpie3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpie3 {
        match self.bits {
            false => Tpie3::Disable,
            true => Tpie3::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tpie3::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tpie3::Enable
    }
}
#[doc = "Field `TPIE3` writer - Tamper Pin n Interrupt Enable"]
pub type Tpie3W<'a, REG> = crate::BitWriter<'a, REG, Tpie3>;
impl<'a, REG> Tpie3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpie3::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpie3::Enable)
    }
}
#[doc = "Tamper Pin n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpie4 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tpie4> for bool {
    #[inline(always)]
    fn from(variant: Tpie4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPIE4` reader - Tamper Pin n Interrupt Enable"]
pub type Tpie4R = crate::BitReader<Tpie4>;
impl Tpie4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpie4 {
        match self.bits {
            false => Tpie4::Disable,
            true => Tpie4::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tpie4::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tpie4::Enable
    }
}
#[doc = "Field `TPIE4` writer - Tamper Pin n Interrupt Enable"]
pub type Tpie4W<'a, REG> = crate::BitWriter<'a, REG, Tpie4>;
impl<'a, REG> Tpie4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpie4::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpie4::Enable)
    }
}
#[doc = "Tamper Pin n Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpie5 {
    #[doc = "0: Disables"]
    Disable = 0,
    #[doc = "1: Enables"]
    Enable = 1,
}
impl From<Tpie5> for bool {
    #[inline(always)]
    fn from(variant: Tpie5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPIE5` reader - Tamper Pin n Interrupt Enable"]
pub type Tpie5R = crate::BitReader<Tpie5>;
impl Tpie5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpie5 {
        match self.bits {
            false => Tpie5::Disable,
            true => Tpie5::Enable,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tpie5::Disable
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tpie5::Enable
    }
}
#[doc = "Field `TPIE5` writer - Tamper Pin n Interrupt Enable"]
pub type Tpie5W<'a, REG> = crate::BitWriter<'a, REG, Tpie5>;
impl<'a, REG> Tpie5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpie5::Disable)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tpie5::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - Digital Tamper Interrupt Enable"]
    #[inline(always)]
    pub fn dtie(&self) -> DtieR {
        DtieR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 2 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie0(&self) -> Tiie0R {
        Tiie0R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie1(&self) -> Tiie1R {
        Tiie1R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie2(&self) -> Tiie2R {
        Tiie2R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie3(&self) -> Tiie3R {
        Tiie3R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie4(&self) -> Tiie4R {
        Tiie4R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie5(&self) -> Tiie5R {
        Tiie5R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie6(&self) -> Tiie6R {
        Tiie6R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie7(&self) -> Tiie7R {
        Tiie7R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie8(&self) -> Tiie8R {
        Tiie8R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie9(&self) -> Tiie9R {
        Tiie9R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie10(&self) -> Tiie10R {
        Tiie10R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 16 - Tamper Pin n Interrupt Enable"]
    #[inline(always)]
    pub fn tpie0(&self) -> Tpie0R {
        Tpie0R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Tamper Pin n Interrupt Enable"]
    #[inline(always)]
    pub fn tpie1(&self) -> Tpie1R {
        Tpie1R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Tamper Pin n Interrupt Enable"]
    #[inline(always)]
    pub fn tpie2(&self) -> Tpie2R {
        Tpie2R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Tamper Pin n Interrupt Enable"]
    #[inline(always)]
    pub fn tpie3(&self) -> Tpie3R {
        Tpie3R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Tamper Pin n Interrupt Enable"]
    #[inline(always)]
    pub fn tpie4(&self) -> Tpie4R {
        Tpie4R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Tamper Pin n Interrupt Enable"]
    #[inline(always)]
    pub fn tpie5(&self) -> Tpie5R {
        Tpie5R::new(((self.bits >> 21) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Digital Tamper Interrupt Enable"]
    #[inline(always)]
    pub fn dtie(&mut self) -> DtieW<IerSpec> {
        DtieW::new(self, 0)
    }
    #[doc = "Bit 2 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie0(&mut self) -> Tiie0W<IerSpec> {
        Tiie0W::new(self, 2)
    }
    #[doc = "Bit 3 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie1(&mut self) -> Tiie1W<IerSpec> {
        Tiie1W::new(self, 3)
    }
    #[doc = "Bit 4 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie2(&mut self) -> Tiie2W<IerSpec> {
        Tiie2W::new(self, 4)
    }
    #[doc = "Bit 5 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie3(&mut self) -> Tiie3W<IerSpec> {
        Tiie3W::new(self, 5)
    }
    #[doc = "Bit 6 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie4(&mut self) -> Tiie4W<IerSpec> {
        Tiie4W::new(self, 6)
    }
    #[doc = "Bit 7 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie5(&mut self) -> Tiie5W<IerSpec> {
        Tiie5W::new(self, 7)
    }
    #[doc = "Bit 8 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie6(&mut self) -> Tiie6W<IerSpec> {
        Tiie6W::new(self, 8)
    }
    #[doc = "Bit 9 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie7(&mut self) -> Tiie7W<IerSpec> {
        Tiie7W::new(self, 9)
    }
    #[doc = "Bit 10 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie8(&mut self) -> Tiie8W<IerSpec> {
        Tiie8W::new(self, 10)
    }
    #[doc = "Bit 11 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie9(&mut self) -> Tiie9W<IerSpec> {
        Tiie9W::new(self, 11)
    }
    #[doc = "Bit 12 - Tamper Input n Interrupt Enable"]
    #[inline(always)]
    pub fn tiie10(&mut self) -> Tiie10W<IerSpec> {
        Tiie10W::new(self, 12)
    }
    #[doc = "Bit 16 - Tamper Pin n Interrupt Enable"]
    #[inline(always)]
    pub fn tpie0(&mut self) -> Tpie0W<IerSpec> {
        Tpie0W::new(self, 16)
    }
    #[doc = "Bit 17 - Tamper Pin n Interrupt Enable"]
    #[inline(always)]
    pub fn tpie1(&mut self) -> Tpie1W<IerSpec> {
        Tpie1W::new(self, 17)
    }
    #[doc = "Bit 18 - Tamper Pin n Interrupt Enable"]
    #[inline(always)]
    pub fn tpie2(&mut self) -> Tpie2W<IerSpec> {
        Tpie2W::new(self, 18)
    }
    #[doc = "Bit 19 - Tamper Pin n Interrupt Enable"]
    #[inline(always)]
    pub fn tpie3(&mut self) -> Tpie3W<IerSpec> {
        Tpie3W::new(self, 19)
    }
    #[doc = "Bit 20 - Tamper Pin n Interrupt Enable"]
    #[inline(always)]
    pub fn tpie4(&mut self) -> Tpie4W<IerSpec> {
        Tpie4W::new(self, 20)
    }
    #[doc = "Bit 21 - Tamper Pin n Interrupt Enable"]
    #[inline(always)]
    pub fn tpie5(&mut self) -> Tpie5W<IerSpec> {
        Tpie5W::new(self, 21)
    }
}
#[doc = "Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ier::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ier::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IerSpec;
impl crate::RegisterSpec for IerSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ier::R`](R) reader structure"]
impl crate::Readable for IerSpec {}
#[doc = "`write(|w| ..)` method takes [`ier::W`](W) writer structure"]
impl crate::Writable for IerSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets IER to value 0"]
impl crate::Resettable for IerSpec {}

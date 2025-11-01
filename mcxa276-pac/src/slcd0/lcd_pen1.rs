#[doc = "Register `LCD_PEN1` reader"]
pub type R = crate::R<LcdPen1Spec>;
#[doc = "Register `LCD_PEN1` writer"]
pub type W = crate::W<LcdPen1Spec>;
#[doc = "LCD Pin 32 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin32En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin32En> for bool {
    #[inline(always)]
    fn from(variant: Pin32En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_32_EN` reader - LCD Pin 32 Enable"]
pub type Pin32EnR = crate::BitReader<Pin32En>;
impl Pin32EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin32En {
        match self.bits {
            false => Pin32En::Disable,
            true => Pin32En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin32En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin32En::Enable
    }
}
#[doc = "Field `PIN_32_EN` writer - LCD Pin 32 Enable"]
pub type Pin32EnW<'a, REG> = crate::BitWriter<'a, REG, Pin32En>;
impl<'a, REG> Pin32EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin32En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin32En::Enable)
    }
}
#[doc = "LCD Pin 33 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin33En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin33En> for bool {
    #[inline(always)]
    fn from(variant: Pin33En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_33_EN` reader - LCD Pin 33 Enable"]
pub type Pin33EnR = crate::BitReader<Pin33En>;
impl Pin33EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin33En {
        match self.bits {
            false => Pin33En::Disable,
            true => Pin33En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin33En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin33En::Enable
    }
}
#[doc = "Field `PIN_33_EN` writer - LCD Pin 33 Enable"]
pub type Pin33EnW<'a, REG> = crate::BitWriter<'a, REG, Pin33En>;
impl<'a, REG> Pin33EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin33En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin33En::Enable)
    }
}
#[doc = "LCD Pin 34 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin34En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin34En> for bool {
    #[inline(always)]
    fn from(variant: Pin34En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_34_EN` reader - LCD Pin 34 Enable"]
pub type Pin34EnR = crate::BitReader<Pin34En>;
impl Pin34EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin34En {
        match self.bits {
            false => Pin34En::Disable,
            true => Pin34En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin34En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin34En::Enable
    }
}
#[doc = "Field `PIN_34_EN` writer - LCD Pin 34 Enable"]
pub type Pin34EnW<'a, REG> = crate::BitWriter<'a, REG, Pin34En>;
impl<'a, REG> Pin34EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin34En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin34En::Enable)
    }
}
#[doc = "LCD Pin 35 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin35En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin35En> for bool {
    #[inline(always)]
    fn from(variant: Pin35En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_35_EN` reader - LCD Pin 35 Enable"]
pub type Pin35EnR = crate::BitReader<Pin35En>;
impl Pin35EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin35En {
        match self.bits {
            false => Pin35En::Disable,
            true => Pin35En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin35En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin35En::Enable
    }
}
#[doc = "Field `PIN_35_EN` writer - LCD Pin 35 Enable"]
pub type Pin35EnW<'a, REG> = crate::BitWriter<'a, REG, Pin35En>;
impl<'a, REG> Pin35EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin35En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin35En::Enable)
    }
}
#[doc = "LCD Pin 36 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin36En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin36En> for bool {
    #[inline(always)]
    fn from(variant: Pin36En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_36_EN` reader - LCD Pin 36 Enable"]
pub type Pin36EnR = crate::BitReader<Pin36En>;
impl Pin36EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin36En {
        match self.bits {
            false => Pin36En::Disable,
            true => Pin36En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin36En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin36En::Enable
    }
}
#[doc = "Field `PIN_36_EN` writer - LCD Pin 36 Enable"]
pub type Pin36EnW<'a, REG> = crate::BitWriter<'a, REG, Pin36En>;
impl<'a, REG> Pin36EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin36En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin36En::Enable)
    }
}
#[doc = "LCD Pin 37 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin37En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin37En> for bool {
    #[inline(always)]
    fn from(variant: Pin37En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_37_EN` reader - LCD Pin 37 Enable"]
pub type Pin37EnR = crate::BitReader<Pin37En>;
impl Pin37EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin37En {
        match self.bits {
            false => Pin37En::Disable,
            true => Pin37En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin37En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin37En::Enable
    }
}
#[doc = "Field `PIN_37_EN` writer - LCD Pin 37 Enable"]
pub type Pin37EnW<'a, REG> = crate::BitWriter<'a, REG, Pin37En>;
impl<'a, REG> Pin37EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin37En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin37En::Enable)
    }
}
#[doc = "LCD Pin 38 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin38En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin38En> for bool {
    #[inline(always)]
    fn from(variant: Pin38En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_38_EN` reader - LCD Pin 38 Enable"]
pub type Pin38EnR = crate::BitReader<Pin38En>;
impl Pin38EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin38En {
        match self.bits {
            false => Pin38En::Disable,
            true => Pin38En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin38En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin38En::Enable
    }
}
#[doc = "Field `PIN_38_EN` writer - LCD Pin 38 Enable"]
pub type Pin38EnW<'a, REG> = crate::BitWriter<'a, REG, Pin38En>;
impl<'a, REG> Pin38EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin38En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin38En::Enable)
    }
}
#[doc = "LCD Pin 39 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin39En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin39En> for bool {
    #[inline(always)]
    fn from(variant: Pin39En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_39_EN` reader - LCD Pin 39 Enable"]
pub type Pin39EnR = crate::BitReader<Pin39En>;
impl Pin39EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin39En {
        match self.bits {
            false => Pin39En::Disable,
            true => Pin39En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin39En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin39En::Enable
    }
}
#[doc = "Field `PIN_39_EN` writer - LCD Pin 39 Enable"]
pub type Pin39EnW<'a, REG> = crate::BitWriter<'a, REG, Pin39En>;
impl<'a, REG> Pin39EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin39En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin39En::Enable)
    }
}
#[doc = "LCD Pin 40 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin40En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin40En> for bool {
    #[inline(always)]
    fn from(variant: Pin40En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_40_EN` reader - LCD Pin 40 Enable"]
pub type Pin40EnR = crate::BitReader<Pin40En>;
impl Pin40EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin40En {
        match self.bits {
            false => Pin40En::Disable,
            true => Pin40En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin40En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin40En::Enable
    }
}
#[doc = "Field `PIN_40_EN` writer - LCD Pin 40 Enable"]
pub type Pin40EnW<'a, REG> = crate::BitWriter<'a, REG, Pin40En>;
impl<'a, REG> Pin40EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin40En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin40En::Enable)
    }
}
#[doc = "LCD Pin 41 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin41En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin41En> for bool {
    #[inline(always)]
    fn from(variant: Pin41En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_41_EN` reader - LCD Pin 41 Enable"]
pub type Pin41EnR = crate::BitReader<Pin41En>;
impl Pin41EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin41En {
        match self.bits {
            false => Pin41En::Disable,
            true => Pin41En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin41En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin41En::Enable
    }
}
#[doc = "Field `PIN_41_EN` writer - LCD Pin 41 Enable"]
pub type Pin41EnW<'a, REG> = crate::BitWriter<'a, REG, Pin41En>;
impl<'a, REG> Pin41EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin41En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin41En::Enable)
    }
}
#[doc = "LCD Pin 42 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin42En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin42En> for bool {
    #[inline(always)]
    fn from(variant: Pin42En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_42_EN` reader - LCD Pin 42 Enable"]
pub type Pin42EnR = crate::BitReader<Pin42En>;
impl Pin42EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin42En {
        match self.bits {
            false => Pin42En::Disable,
            true => Pin42En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin42En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin42En::Enable
    }
}
#[doc = "Field `PIN_42_EN` writer - LCD Pin 42 Enable"]
pub type Pin42EnW<'a, REG> = crate::BitWriter<'a, REG, Pin42En>;
impl<'a, REG> Pin42EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin42En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin42En::Enable)
    }
}
#[doc = "LCD Pin 43 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin43En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin43En> for bool {
    #[inline(always)]
    fn from(variant: Pin43En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_43_EN` reader - LCD Pin 43 Enable"]
pub type Pin43EnR = crate::BitReader<Pin43En>;
impl Pin43EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin43En {
        match self.bits {
            false => Pin43En::Disable,
            true => Pin43En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin43En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin43En::Enable
    }
}
#[doc = "Field `PIN_43_EN` writer - LCD Pin 43 Enable"]
pub type Pin43EnW<'a, REG> = crate::BitWriter<'a, REG, Pin43En>;
impl<'a, REG> Pin43EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin43En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin43En::Enable)
    }
}
#[doc = "LCD Pin 44 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin44En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin44En> for bool {
    #[inline(always)]
    fn from(variant: Pin44En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_44_EN` reader - LCD Pin 44 Enable"]
pub type Pin44EnR = crate::BitReader<Pin44En>;
impl Pin44EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin44En {
        match self.bits {
            false => Pin44En::Disable,
            true => Pin44En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin44En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin44En::Enable
    }
}
#[doc = "Field `PIN_44_EN` writer - LCD Pin 44 Enable"]
pub type Pin44EnW<'a, REG> = crate::BitWriter<'a, REG, Pin44En>;
impl<'a, REG> Pin44EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin44En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin44En::Enable)
    }
}
#[doc = "LCD Pin 45 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin45En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin45En> for bool {
    #[inline(always)]
    fn from(variant: Pin45En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_45_EN` reader - LCD Pin 45 Enable"]
pub type Pin45EnR = crate::BitReader<Pin45En>;
impl Pin45EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin45En {
        match self.bits {
            false => Pin45En::Disable,
            true => Pin45En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin45En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin45En::Enable
    }
}
#[doc = "Field `PIN_45_EN` writer - LCD Pin 45 Enable"]
pub type Pin45EnW<'a, REG> = crate::BitWriter<'a, REG, Pin45En>;
impl<'a, REG> Pin45EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin45En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin45En::Enable)
    }
}
#[doc = "LCD Pin 46 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin46En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin46En> for bool {
    #[inline(always)]
    fn from(variant: Pin46En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_46_EN` reader - LCD Pin 46 Enable"]
pub type Pin46EnR = crate::BitReader<Pin46En>;
impl Pin46EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin46En {
        match self.bits {
            false => Pin46En::Disable,
            true => Pin46En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin46En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin46En::Enable
    }
}
#[doc = "Field `PIN_46_EN` writer - LCD Pin 46 Enable"]
pub type Pin46EnW<'a, REG> = crate::BitWriter<'a, REG, Pin46En>;
impl<'a, REG> Pin46EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin46En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin46En::Enable)
    }
}
#[doc = "LCD Pin 47 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin47En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin47En> for bool {
    #[inline(always)]
    fn from(variant: Pin47En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_47_EN` reader - LCD Pin 47 Enable"]
pub type Pin47EnR = crate::BitReader<Pin47En>;
impl Pin47EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin47En {
        match self.bits {
            false => Pin47En::Disable,
            true => Pin47En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin47En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin47En::Enable
    }
}
#[doc = "Field `PIN_47_EN` writer - LCD Pin 47 Enable"]
pub type Pin47EnW<'a, REG> = crate::BitWriter<'a, REG, Pin47En>;
impl<'a, REG> Pin47EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin47En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin47En::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - LCD Pin 32 Enable"]
    #[inline(always)]
    pub fn pin_32_en(&self) -> Pin32EnR {
        Pin32EnR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - LCD Pin 33 Enable"]
    #[inline(always)]
    pub fn pin_33_en(&self) -> Pin33EnR {
        Pin33EnR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - LCD Pin 34 Enable"]
    #[inline(always)]
    pub fn pin_34_en(&self) -> Pin34EnR {
        Pin34EnR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - LCD Pin 35 Enable"]
    #[inline(always)]
    pub fn pin_35_en(&self) -> Pin35EnR {
        Pin35EnR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - LCD Pin 36 Enable"]
    #[inline(always)]
    pub fn pin_36_en(&self) -> Pin36EnR {
        Pin36EnR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - LCD Pin 37 Enable"]
    #[inline(always)]
    pub fn pin_37_en(&self) -> Pin37EnR {
        Pin37EnR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - LCD Pin 38 Enable"]
    #[inline(always)]
    pub fn pin_38_en(&self) -> Pin38EnR {
        Pin38EnR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - LCD Pin 39 Enable"]
    #[inline(always)]
    pub fn pin_39_en(&self) -> Pin39EnR {
        Pin39EnR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - LCD Pin 40 Enable"]
    #[inline(always)]
    pub fn pin_40_en(&self) -> Pin40EnR {
        Pin40EnR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - LCD Pin 41 Enable"]
    #[inline(always)]
    pub fn pin_41_en(&self) -> Pin41EnR {
        Pin41EnR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - LCD Pin 42 Enable"]
    #[inline(always)]
    pub fn pin_42_en(&self) -> Pin42EnR {
        Pin42EnR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - LCD Pin 43 Enable"]
    #[inline(always)]
    pub fn pin_43_en(&self) -> Pin43EnR {
        Pin43EnR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - LCD Pin 44 Enable"]
    #[inline(always)]
    pub fn pin_44_en(&self) -> Pin44EnR {
        Pin44EnR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - LCD Pin 45 Enable"]
    #[inline(always)]
    pub fn pin_45_en(&self) -> Pin45EnR {
        Pin45EnR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - LCD Pin 46 Enable"]
    #[inline(always)]
    pub fn pin_46_en(&self) -> Pin46EnR {
        Pin46EnR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - LCD Pin 47 Enable"]
    #[inline(always)]
    pub fn pin_47_en(&self) -> Pin47EnR {
        Pin47EnR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - LCD Pin 32 Enable"]
    #[inline(always)]
    pub fn pin_32_en(&mut self) -> Pin32EnW<LcdPen1Spec> {
        Pin32EnW::new(self, 0)
    }
    #[doc = "Bit 1 - LCD Pin 33 Enable"]
    #[inline(always)]
    pub fn pin_33_en(&mut self) -> Pin33EnW<LcdPen1Spec> {
        Pin33EnW::new(self, 1)
    }
    #[doc = "Bit 2 - LCD Pin 34 Enable"]
    #[inline(always)]
    pub fn pin_34_en(&mut self) -> Pin34EnW<LcdPen1Spec> {
        Pin34EnW::new(self, 2)
    }
    #[doc = "Bit 3 - LCD Pin 35 Enable"]
    #[inline(always)]
    pub fn pin_35_en(&mut self) -> Pin35EnW<LcdPen1Spec> {
        Pin35EnW::new(self, 3)
    }
    #[doc = "Bit 4 - LCD Pin 36 Enable"]
    #[inline(always)]
    pub fn pin_36_en(&mut self) -> Pin36EnW<LcdPen1Spec> {
        Pin36EnW::new(self, 4)
    }
    #[doc = "Bit 5 - LCD Pin 37 Enable"]
    #[inline(always)]
    pub fn pin_37_en(&mut self) -> Pin37EnW<LcdPen1Spec> {
        Pin37EnW::new(self, 5)
    }
    #[doc = "Bit 6 - LCD Pin 38 Enable"]
    #[inline(always)]
    pub fn pin_38_en(&mut self) -> Pin38EnW<LcdPen1Spec> {
        Pin38EnW::new(self, 6)
    }
    #[doc = "Bit 7 - LCD Pin 39 Enable"]
    #[inline(always)]
    pub fn pin_39_en(&mut self) -> Pin39EnW<LcdPen1Spec> {
        Pin39EnW::new(self, 7)
    }
    #[doc = "Bit 8 - LCD Pin 40 Enable"]
    #[inline(always)]
    pub fn pin_40_en(&mut self) -> Pin40EnW<LcdPen1Spec> {
        Pin40EnW::new(self, 8)
    }
    #[doc = "Bit 9 - LCD Pin 41 Enable"]
    #[inline(always)]
    pub fn pin_41_en(&mut self) -> Pin41EnW<LcdPen1Spec> {
        Pin41EnW::new(self, 9)
    }
    #[doc = "Bit 10 - LCD Pin 42 Enable"]
    #[inline(always)]
    pub fn pin_42_en(&mut self) -> Pin42EnW<LcdPen1Spec> {
        Pin42EnW::new(self, 10)
    }
    #[doc = "Bit 11 - LCD Pin 43 Enable"]
    #[inline(always)]
    pub fn pin_43_en(&mut self) -> Pin43EnW<LcdPen1Spec> {
        Pin43EnW::new(self, 11)
    }
    #[doc = "Bit 12 - LCD Pin 44 Enable"]
    #[inline(always)]
    pub fn pin_44_en(&mut self) -> Pin44EnW<LcdPen1Spec> {
        Pin44EnW::new(self, 12)
    }
    #[doc = "Bit 13 - LCD Pin 45 Enable"]
    #[inline(always)]
    pub fn pin_45_en(&mut self) -> Pin45EnW<LcdPen1Spec> {
        Pin45EnW::new(self, 13)
    }
    #[doc = "Bit 14 - LCD Pin 46 Enable"]
    #[inline(always)]
    pub fn pin_46_en(&mut self) -> Pin46EnW<LcdPen1Spec> {
        Pin46EnW::new(self, 14)
    }
    #[doc = "Bit 15 - LCD Pin 47 Enable"]
    #[inline(always)]
    pub fn pin_47_en(&mut self) -> Pin47EnW<LcdPen1Spec> {
        Pin47EnW::new(self, 15)
    }
}
#[doc = "LCD Pin Enable Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_pen1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_pen1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LcdPen1Spec;
impl crate::RegisterSpec for LcdPen1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`lcd_pen1::R`](R) reader structure"]
impl crate::Readable for LcdPen1Spec {}
#[doc = "`write(|w| ..)` method takes [`lcd_pen1::W`](W) writer structure"]
impl crate::Writable for LcdPen1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LCD_PEN1 to value 0"]
impl crate::Resettable for LcdPen1Spec {}

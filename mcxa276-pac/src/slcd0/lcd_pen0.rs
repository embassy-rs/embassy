#[doc = "Register `LCD_PEN0` reader"]
pub type R = crate::R<LcdPen0Spec>;
#[doc = "Register `LCD_PEN0` writer"]
pub type W = crate::W<LcdPen0Spec>;
#[doc = "LCD Pin 0 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin0En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin0En> for bool {
    #[inline(always)]
    fn from(variant: Pin0En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_0_EN` reader - LCD Pin 0 Enable"]
pub type Pin0EnR = crate::BitReader<Pin0En>;
impl Pin0EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin0En {
        match self.bits {
            false => Pin0En::Disable,
            true => Pin0En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin0En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin0En::Enable
    }
}
#[doc = "Field `PIN_0_EN` writer - LCD Pin 0 Enable"]
pub type Pin0EnW<'a, REG> = crate::BitWriter<'a, REG, Pin0En>;
impl<'a, REG> Pin0EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin0En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin0En::Enable)
    }
}
#[doc = "LCD Pin 1 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin1En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin1En> for bool {
    #[inline(always)]
    fn from(variant: Pin1En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_1_EN` reader - LCD Pin 1 Enable"]
pub type Pin1EnR = crate::BitReader<Pin1En>;
impl Pin1EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin1En {
        match self.bits {
            false => Pin1En::Disable,
            true => Pin1En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin1En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin1En::Enable
    }
}
#[doc = "Field `PIN_1_EN` writer - LCD Pin 1 Enable"]
pub type Pin1EnW<'a, REG> = crate::BitWriter<'a, REG, Pin1En>;
impl<'a, REG> Pin1EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin1En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin1En::Enable)
    }
}
#[doc = "LCD Pin 2 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin2En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin2En> for bool {
    #[inline(always)]
    fn from(variant: Pin2En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_2_EN` reader - LCD Pin 2 Enable"]
pub type Pin2EnR = crate::BitReader<Pin2En>;
impl Pin2EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin2En {
        match self.bits {
            false => Pin2En::Disable,
            true => Pin2En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin2En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin2En::Enable
    }
}
#[doc = "Field `PIN_2_EN` writer - LCD Pin 2 Enable"]
pub type Pin2EnW<'a, REG> = crate::BitWriter<'a, REG, Pin2En>;
impl<'a, REG> Pin2EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin2En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin2En::Enable)
    }
}
#[doc = "LCD Pin 3 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin3En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin3En> for bool {
    #[inline(always)]
    fn from(variant: Pin3En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_3_EN` reader - LCD Pin 3 Enable"]
pub type Pin3EnR = crate::BitReader<Pin3En>;
impl Pin3EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin3En {
        match self.bits {
            false => Pin3En::Disable,
            true => Pin3En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin3En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin3En::Enable
    }
}
#[doc = "Field `PIN_3_EN` writer - LCD Pin 3 Enable"]
pub type Pin3EnW<'a, REG> = crate::BitWriter<'a, REG, Pin3En>;
impl<'a, REG> Pin3EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin3En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin3En::Enable)
    }
}
#[doc = "LCD Pin 4 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin4En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin4En> for bool {
    #[inline(always)]
    fn from(variant: Pin4En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_4_EN` reader - LCD Pin 4 Enable"]
pub type Pin4EnR = crate::BitReader<Pin4En>;
impl Pin4EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin4En {
        match self.bits {
            false => Pin4En::Disable,
            true => Pin4En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin4En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin4En::Enable
    }
}
#[doc = "Field `PIN_4_EN` writer - LCD Pin 4 Enable"]
pub type Pin4EnW<'a, REG> = crate::BitWriter<'a, REG, Pin4En>;
impl<'a, REG> Pin4EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin4En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin4En::Enable)
    }
}
#[doc = "LCD Pin 5 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin5En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin5En> for bool {
    #[inline(always)]
    fn from(variant: Pin5En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_5_EN` reader - LCD Pin 5 Enable"]
pub type Pin5EnR = crate::BitReader<Pin5En>;
impl Pin5EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin5En {
        match self.bits {
            false => Pin5En::Disable,
            true => Pin5En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin5En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin5En::Enable
    }
}
#[doc = "Field `PIN_5_EN` writer - LCD Pin 5 Enable"]
pub type Pin5EnW<'a, REG> = crate::BitWriter<'a, REG, Pin5En>;
impl<'a, REG> Pin5EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin5En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin5En::Enable)
    }
}
#[doc = "LCD Pin 6 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin6En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin6En> for bool {
    #[inline(always)]
    fn from(variant: Pin6En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_6_EN` reader - LCD Pin 6 Enable"]
pub type Pin6EnR = crate::BitReader<Pin6En>;
impl Pin6EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin6En {
        match self.bits {
            false => Pin6En::Disable,
            true => Pin6En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin6En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin6En::Enable
    }
}
#[doc = "Field `PIN_6_EN` writer - LCD Pin 6 Enable"]
pub type Pin6EnW<'a, REG> = crate::BitWriter<'a, REG, Pin6En>;
impl<'a, REG> Pin6EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin6En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin6En::Enable)
    }
}
#[doc = "LCD Pin 7 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin7En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin7En> for bool {
    #[inline(always)]
    fn from(variant: Pin7En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_7_EN` reader - LCD Pin 7 Enable"]
pub type Pin7EnR = crate::BitReader<Pin7En>;
impl Pin7EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin7En {
        match self.bits {
            false => Pin7En::Disable,
            true => Pin7En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin7En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin7En::Enable
    }
}
#[doc = "Field `PIN_7_EN` writer - LCD Pin 7 Enable"]
pub type Pin7EnW<'a, REG> = crate::BitWriter<'a, REG, Pin7En>;
impl<'a, REG> Pin7EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin7En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin7En::Enable)
    }
}
#[doc = "LCD Pin 8 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin8En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin8En> for bool {
    #[inline(always)]
    fn from(variant: Pin8En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_8_EN` reader - LCD Pin 8 Enable"]
pub type Pin8EnR = crate::BitReader<Pin8En>;
impl Pin8EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin8En {
        match self.bits {
            false => Pin8En::Disable,
            true => Pin8En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin8En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin8En::Enable
    }
}
#[doc = "Field `PIN_8_EN` writer - LCD Pin 8 Enable"]
pub type Pin8EnW<'a, REG> = crate::BitWriter<'a, REG, Pin8En>;
impl<'a, REG> Pin8EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin8En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin8En::Enable)
    }
}
#[doc = "LCD Pin 9 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin9En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin9En> for bool {
    #[inline(always)]
    fn from(variant: Pin9En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_9_EN` reader - LCD Pin 9 Enable"]
pub type Pin9EnR = crate::BitReader<Pin9En>;
impl Pin9EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin9En {
        match self.bits {
            false => Pin9En::Disable,
            true => Pin9En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin9En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin9En::Enable
    }
}
#[doc = "Field `PIN_9_EN` writer - LCD Pin 9 Enable"]
pub type Pin9EnW<'a, REG> = crate::BitWriter<'a, REG, Pin9En>;
impl<'a, REG> Pin9EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin9En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin9En::Enable)
    }
}
#[doc = "LCD Pin 10 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin10En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin10En> for bool {
    #[inline(always)]
    fn from(variant: Pin10En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_10_EN` reader - LCD Pin 10 Enable"]
pub type Pin10EnR = crate::BitReader<Pin10En>;
impl Pin10EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin10En {
        match self.bits {
            false => Pin10En::Disable,
            true => Pin10En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin10En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin10En::Enable
    }
}
#[doc = "Field `PIN_10_EN` writer - LCD Pin 10 Enable"]
pub type Pin10EnW<'a, REG> = crate::BitWriter<'a, REG, Pin10En>;
impl<'a, REG> Pin10EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin10En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin10En::Enable)
    }
}
#[doc = "LCD Pin 11 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin11En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin11En> for bool {
    #[inline(always)]
    fn from(variant: Pin11En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_11_EN` reader - LCD Pin 11 Enable"]
pub type Pin11EnR = crate::BitReader<Pin11En>;
impl Pin11EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin11En {
        match self.bits {
            false => Pin11En::Disable,
            true => Pin11En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin11En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin11En::Enable
    }
}
#[doc = "Field `PIN_11_EN` writer - LCD Pin 11 Enable"]
pub type Pin11EnW<'a, REG> = crate::BitWriter<'a, REG, Pin11En>;
impl<'a, REG> Pin11EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin11En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin11En::Enable)
    }
}
#[doc = "LCD Pin 12 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin12En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin12En> for bool {
    #[inline(always)]
    fn from(variant: Pin12En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_12_EN` reader - LCD Pin 12 Enable"]
pub type Pin12EnR = crate::BitReader<Pin12En>;
impl Pin12EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin12En {
        match self.bits {
            false => Pin12En::Disable,
            true => Pin12En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin12En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin12En::Enable
    }
}
#[doc = "Field `PIN_12_EN` writer - LCD Pin 12 Enable"]
pub type Pin12EnW<'a, REG> = crate::BitWriter<'a, REG, Pin12En>;
impl<'a, REG> Pin12EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin12En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin12En::Enable)
    }
}
#[doc = "LCD Pin 13 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin13En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin13En> for bool {
    #[inline(always)]
    fn from(variant: Pin13En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_13_EN` reader - LCD Pin 13 Enable"]
pub type Pin13EnR = crate::BitReader<Pin13En>;
impl Pin13EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin13En {
        match self.bits {
            false => Pin13En::Disable,
            true => Pin13En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin13En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin13En::Enable
    }
}
#[doc = "Field `PIN_13_EN` writer - LCD Pin 13 Enable"]
pub type Pin13EnW<'a, REG> = crate::BitWriter<'a, REG, Pin13En>;
impl<'a, REG> Pin13EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin13En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin13En::Enable)
    }
}
#[doc = "LCD Pin 14 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin14En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin14En> for bool {
    #[inline(always)]
    fn from(variant: Pin14En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_14_EN` reader - LCD Pin 14 Enable"]
pub type Pin14EnR = crate::BitReader<Pin14En>;
impl Pin14EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin14En {
        match self.bits {
            false => Pin14En::Disable,
            true => Pin14En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin14En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin14En::Enable
    }
}
#[doc = "Field `PIN_14_EN` writer - LCD Pin 14 Enable"]
pub type Pin14EnW<'a, REG> = crate::BitWriter<'a, REG, Pin14En>;
impl<'a, REG> Pin14EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin14En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin14En::Enable)
    }
}
#[doc = "LCD Pin 15 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin15En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin15En> for bool {
    #[inline(always)]
    fn from(variant: Pin15En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_15_EN` reader - LCD Pin 15 Enable"]
pub type Pin15EnR = crate::BitReader<Pin15En>;
impl Pin15EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin15En {
        match self.bits {
            false => Pin15En::Disable,
            true => Pin15En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin15En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin15En::Enable
    }
}
#[doc = "Field `PIN_15_EN` writer - LCD Pin 15 Enable"]
pub type Pin15EnW<'a, REG> = crate::BitWriter<'a, REG, Pin15En>;
impl<'a, REG> Pin15EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin15En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin15En::Enable)
    }
}
#[doc = "LCD Pin 16 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin16En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin16En> for bool {
    #[inline(always)]
    fn from(variant: Pin16En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_16_EN` reader - LCD Pin 16 Enable"]
pub type Pin16EnR = crate::BitReader<Pin16En>;
impl Pin16EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin16En {
        match self.bits {
            false => Pin16En::Disable,
            true => Pin16En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin16En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin16En::Enable
    }
}
#[doc = "Field `PIN_16_EN` writer - LCD Pin 16 Enable"]
pub type Pin16EnW<'a, REG> = crate::BitWriter<'a, REG, Pin16En>;
impl<'a, REG> Pin16EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin16En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin16En::Enable)
    }
}
#[doc = "LCD Pin 17 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin17En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin17En> for bool {
    #[inline(always)]
    fn from(variant: Pin17En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_17_EN` reader - LCD Pin 17 Enable"]
pub type Pin17EnR = crate::BitReader<Pin17En>;
impl Pin17EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin17En {
        match self.bits {
            false => Pin17En::Disable,
            true => Pin17En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin17En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin17En::Enable
    }
}
#[doc = "Field `PIN_17_EN` writer - LCD Pin 17 Enable"]
pub type Pin17EnW<'a, REG> = crate::BitWriter<'a, REG, Pin17En>;
impl<'a, REG> Pin17EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin17En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin17En::Enable)
    }
}
#[doc = "LCD Pin 18 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin18En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin18En> for bool {
    #[inline(always)]
    fn from(variant: Pin18En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_18_EN` reader - LCD Pin 18 Enable"]
pub type Pin18EnR = crate::BitReader<Pin18En>;
impl Pin18EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin18En {
        match self.bits {
            false => Pin18En::Disable,
            true => Pin18En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin18En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin18En::Enable
    }
}
#[doc = "Field `PIN_18_EN` writer - LCD Pin 18 Enable"]
pub type Pin18EnW<'a, REG> = crate::BitWriter<'a, REG, Pin18En>;
impl<'a, REG> Pin18EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin18En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin18En::Enable)
    }
}
#[doc = "LCD Pin 19 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin19En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin19En> for bool {
    #[inline(always)]
    fn from(variant: Pin19En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_19_EN` reader - LCD Pin 19 Enable"]
pub type Pin19EnR = crate::BitReader<Pin19En>;
impl Pin19EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin19En {
        match self.bits {
            false => Pin19En::Disable,
            true => Pin19En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin19En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin19En::Enable
    }
}
#[doc = "Field `PIN_19_EN` writer - LCD Pin 19 Enable"]
pub type Pin19EnW<'a, REG> = crate::BitWriter<'a, REG, Pin19En>;
impl<'a, REG> Pin19EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin19En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin19En::Enable)
    }
}
#[doc = "LCD Pin 20 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin20En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin20En> for bool {
    #[inline(always)]
    fn from(variant: Pin20En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_20_EN` reader - LCD Pin 20 Enable"]
pub type Pin20EnR = crate::BitReader<Pin20En>;
impl Pin20EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin20En {
        match self.bits {
            false => Pin20En::Disable,
            true => Pin20En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin20En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin20En::Enable
    }
}
#[doc = "Field `PIN_20_EN` writer - LCD Pin 20 Enable"]
pub type Pin20EnW<'a, REG> = crate::BitWriter<'a, REG, Pin20En>;
impl<'a, REG> Pin20EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin20En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin20En::Enable)
    }
}
#[doc = "LCD Pin 21 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin21En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin21En> for bool {
    #[inline(always)]
    fn from(variant: Pin21En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_21_EN` reader - LCD Pin 21 Enable"]
pub type Pin21EnR = crate::BitReader<Pin21En>;
impl Pin21EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin21En {
        match self.bits {
            false => Pin21En::Disable,
            true => Pin21En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin21En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin21En::Enable
    }
}
#[doc = "Field `PIN_21_EN` writer - LCD Pin 21 Enable"]
pub type Pin21EnW<'a, REG> = crate::BitWriter<'a, REG, Pin21En>;
impl<'a, REG> Pin21EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin21En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin21En::Enable)
    }
}
#[doc = "LCD Pin 22 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin22En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin22En> for bool {
    #[inline(always)]
    fn from(variant: Pin22En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_22_EN` reader - LCD Pin 22 Enable"]
pub type Pin22EnR = crate::BitReader<Pin22En>;
impl Pin22EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin22En {
        match self.bits {
            false => Pin22En::Disable,
            true => Pin22En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin22En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin22En::Enable
    }
}
#[doc = "Field `PIN_22_EN` writer - LCD Pin 22 Enable"]
pub type Pin22EnW<'a, REG> = crate::BitWriter<'a, REG, Pin22En>;
impl<'a, REG> Pin22EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin22En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin22En::Enable)
    }
}
#[doc = "LCD Pin 23 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin23En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin23En> for bool {
    #[inline(always)]
    fn from(variant: Pin23En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_23_EN` reader - LCD Pin 23 Enable"]
pub type Pin23EnR = crate::BitReader<Pin23En>;
impl Pin23EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin23En {
        match self.bits {
            false => Pin23En::Disable,
            true => Pin23En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin23En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin23En::Enable
    }
}
#[doc = "Field `PIN_23_EN` writer - LCD Pin 23 Enable"]
pub type Pin23EnW<'a, REG> = crate::BitWriter<'a, REG, Pin23En>;
impl<'a, REG> Pin23EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin23En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin23En::Enable)
    }
}
#[doc = "LCD Pin 24 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin24En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin24En> for bool {
    #[inline(always)]
    fn from(variant: Pin24En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_24_EN` reader - LCD Pin 24 Enable"]
pub type Pin24EnR = crate::BitReader<Pin24En>;
impl Pin24EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin24En {
        match self.bits {
            false => Pin24En::Disable,
            true => Pin24En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin24En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin24En::Enable
    }
}
#[doc = "Field `PIN_24_EN` writer - LCD Pin 24 Enable"]
pub type Pin24EnW<'a, REG> = crate::BitWriter<'a, REG, Pin24En>;
impl<'a, REG> Pin24EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin24En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin24En::Enable)
    }
}
#[doc = "LCD Pin 25 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin25En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin25En> for bool {
    #[inline(always)]
    fn from(variant: Pin25En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_25_EN` reader - LCD Pin 25 Enable"]
pub type Pin25EnR = crate::BitReader<Pin25En>;
impl Pin25EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin25En {
        match self.bits {
            false => Pin25En::Disable,
            true => Pin25En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin25En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin25En::Enable
    }
}
#[doc = "Field `PIN_25_EN` writer - LCD Pin 25 Enable"]
pub type Pin25EnW<'a, REG> = crate::BitWriter<'a, REG, Pin25En>;
impl<'a, REG> Pin25EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin25En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin25En::Enable)
    }
}
#[doc = "LCD Pin 26 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin26En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin26En> for bool {
    #[inline(always)]
    fn from(variant: Pin26En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_26_EN` reader - LCD Pin 26 Enable"]
pub type Pin26EnR = crate::BitReader<Pin26En>;
impl Pin26EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin26En {
        match self.bits {
            false => Pin26En::Disable,
            true => Pin26En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin26En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin26En::Enable
    }
}
#[doc = "Field `PIN_26_EN` writer - LCD Pin 26 Enable"]
pub type Pin26EnW<'a, REG> = crate::BitWriter<'a, REG, Pin26En>;
impl<'a, REG> Pin26EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin26En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin26En::Enable)
    }
}
#[doc = "LCD Pin 27 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin27En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin27En> for bool {
    #[inline(always)]
    fn from(variant: Pin27En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_27_EN` reader - LCD Pin 27 Enable"]
pub type Pin27EnR = crate::BitReader<Pin27En>;
impl Pin27EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin27En {
        match self.bits {
            false => Pin27En::Disable,
            true => Pin27En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin27En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin27En::Enable
    }
}
#[doc = "Field `PIN_27_EN` writer - LCD Pin 27 Enable"]
pub type Pin27EnW<'a, REG> = crate::BitWriter<'a, REG, Pin27En>;
impl<'a, REG> Pin27EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin27En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin27En::Enable)
    }
}
#[doc = "LCD Pin 28 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin28En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin28En> for bool {
    #[inline(always)]
    fn from(variant: Pin28En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_28_EN` reader - LCD Pin 28 Enable"]
pub type Pin28EnR = crate::BitReader<Pin28En>;
impl Pin28EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin28En {
        match self.bits {
            false => Pin28En::Disable,
            true => Pin28En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin28En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin28En::Enable
    }
}
#[doc = "Field `PIN_28_EN` writer - LCD Pin 28 Enable"]
pub type Pin28EnW<'a, REG> = crate::BitWriter<'a, REG, Pin28En>;
impl<'a, REG> Pin28EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin28En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin28En::Enable)
    }
}
#[doc = "LCD Pin 29 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin29En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin29En> for bool {
    #[inline(always)]
    fn from(variant: Pin29En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_29_EN` reader - LCD Pin 29 Enable"]
pub type Pin29EnR = crate::BitReader<Pin29En>;
impl Pin29EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin29En {
        match self.bits {
            false => Pin29En::Disable,
            true => Pin29En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin29En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin29En::Enable
    }
}
#[doc = "Field `PIN_29_EN` writer - LCD Pin 29 Enable"]
pub type Pin29EnW<'a, REG> = crate::BitWriter<'a, REG, Pin29En>;
impl<'a, REG> Pin29EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin29En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin29En::Enable)
    }
}
#[doc = "LCD Pin 30 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin30En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin30En> for bool {
    #[inline(always)]
    fn from(variant: Pin30En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_30_EN` reader - LCD Pin 30 Enable"]
pub type Pin30EnR = crate::BitReader<Pin30En>;
impl Pin30EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin30En {
        match self.bits {
            false => Pin30En::Disable,
            true => Pin30En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin30En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin30En::Enable
    }
}
#[doc = "Field `PIN_30_EN` writer - LCD Pin 30 Enable"]
pub type Pin30EnW<'a, REG> = crate::BitWriter<'a, REG, Pin30En>;
impl<'a, REG> Pin30EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin30En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin30En::Enable)
    }
}
#[doc = "LCD Pin 31 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin31En {
    #[doc = "0: Pin Disable"]
    Disable = 0,
    #[doc = "1: Pin Enable"]
    Enable = 1,
}
impl From<Pin31En> for bool {
    #[inline(always)]
    fn from(variant: Pin31En) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_31_EN` reader - LCD Pin 31 Enable"]
pub type Pin31EnR = crate::BitReader<Pin31En>;
impl Pin31EnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin31En {
        match self.bits {
            false => Pin31En::Disable,
            true => Pin31En::Enable,
        }
    }
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Pin31En::Disable
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Pin31En::Enable
    }
}
#[doc = "Field `PIN_31_EN` writer - LCD Pin 31 Enable"]
pub type Pin31EnW<'a, REG> = crate::BitWriter<'a, REG, Pin31En>;
impl<'a, REG> Pin31EnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin31En::Disable)
    }
    #[doc = "Pin Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Pin31En::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - LCD Pin 0 Enable"]
    #[inline(always)]
    pub fn pin_0_en(&self) -> Pin0EnR {
        Pin0EnR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - LCD Pin 1 Enable"]
    #[inline(always)]
    pub fn pin_1_en(&self) -> Pin1EnR {
        Pin1EnR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - LCD Pin 2 Enable"]
    #[inline(always)]
    pub fn pin_2_en(&self) -> Pin2EnR {
        Pin2EnR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - LCD Pin 3 Enable"]
    #[inline(always)]
    pub fn pin_3_en(&self) -> Pin3EnR {
        Pin3EnR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - LCD Pin 4 Enable"]
    #[inline(always)]
    pub fn pin_4_en(&self) -> Pin4EnR {
        Pin4EnR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - LCD Pin 5 Enable"]
    #[inline(always)]
    pub fn pin_5_en(&self) -> Pin5EnR {
        Pin5EnR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - LCD Pin 6 Enable"]
    #[inline(always)]
    pub fn pin_6_en(&self) -> Pin6EnR {
        Pin6EnR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - LCD Pin 7 Enable"]
    #[inline(always)]
    pub fn pin_7_en(&self) -> Pin7EnR {
        Pin7EnR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - LCD Pin 8 Enable"]
    #[inline(always)]
    pub fn pin_8_en(&self) -> Pin8EnR {
        Pin8EnR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - LCD Pin 9 Enable"]
    #[inline(always)]
    pub fn pin_9_en(&self) -> Pin9EnR {
        Pin9EnR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - LCD Pin 10 Enable"]
    #[inline(always)]
    pub fn pin_10_en(&self) -> Pin10EnR {
        Pin10EnR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - LCD Pin 11 Enable"]
    #[inline(always)]
    pub fn pin_11_en(&self) -> Pin11EnR {
        Pin11EnR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - LCD Pin 12 Enable"]
    #[inline(always)]
    pub fn pin_12_en(&self) -> Pin12EnR {
        Pin12EnR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - LCD Pin 13 Enable"]
    #[inline(always)]
    pub fn pin_13_en(&self) -> Pin13EnR {
        Pin13EnR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - LCD Pin 14 Enable"]
    #[inline(always)]
    pub fn pin_14_en(&self) -> Pin14EnR {
        Pin14EnR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - LCD Pin 15 Enable"]
    #[inline(always)]
    pub fn pin_15_en(&self) -> Pin15EnR {
        Pin15EnR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - LCD Pin 16 Enable"]
    #[inline(always)]
    pub fn pin_16_en(&self) -> Pin16EnR {
        Pin16EnR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - LCD Pin 17 Enable"]
    #[inline(always)]
    pub fn pin_17_en(&self) -> Pin17EnR {
        Pin17EnR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - LCD Pin 18 Enable"]
    #[inline(always)]
    pub fn pin_18_en(&self) -> Pin18EnR {
        Pin18EnR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - LCD Pin 19 Enable"]
    #[inline(always)]
    pub fn pin_19_en(&self) -> Pin19EnR {
        Pin19EnR::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - LCD Pin 20 Enable"]
    #[inline(always)]
    pub fn pin_20_en(&self) -> Pin20EnR {
        Pin20EnR::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - LCD Pin 21 Enable"]
    #[inline(always)]
    pub fn pin_21_en(&self) -> Pin21EnR {
        Pin21EnR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - LCD Pin 22 Enable"]
    #[inline(always)]
    pub fn pin_22_en(&self) -> Pin22EnR {
        Pin22EnR::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - LCD Pin 23 Enable"]
    #[inline(always)]
    pub fn pin_23_en(&self) -> Pin23EnR {
        Pin23EnR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - LCD Pin 24 Enable"]
    #[inline(always)]
    pub fn pin_24_en(&self) -> Pin24EnR {
        Pin24EnR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - LCD Pin 25 Enable"]
    #[inline(always)]
    pub fn pin_25_en(&self) -> Pin25EnR {
        Pin25EnR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - LCD Pin 26 Enable"]
    #[inline(always)]
    pub fn pin_26_en(&self) -> Pin26EnR {
        Pin26EnR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - LCD Pin 27 Enable"]
    #[inline(always)]
    pub fn pin_27_en(&self) -> Pin27EnR {
        Pin27EnR::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - LCD Pin 28 Enable"]
    #[inline(always)]
    pub fn pin_28_en(&self) -> Pin28EnR {
        Pin28EnR::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - LCD Pin 29 Enable"]
    #[inline(always)]
    pub fn pin_29_en(&self) -> Pin29EnR {
        Pin29EnR::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - LCD Pin 30 Enable"]
    #[inline(always)]
    pub fn pin_30_en(&self) -> Pin30EnR {
        Pin30EnR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - LCD Pin 31 Enable"]
    #[inline(always)]
    pub fn pin_31_en(&self) -> Pin31EnR {
        Pin31EnR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - LCD Pin 0 Enable"]
    #[inline(always)]
    pub fn pin_0_en(&mut self) -> Pin0EnW<LcdPen0Spec> {
        Pin0EnW::new(self, 0)
    }
    #[doc = "Bit 1 - LCD Pin 1 Enable"]
    #[inline(always)]
    pub fn pin_1_en(&mut self) -> Pin1EnW<LcdPen0Spec> {
        Pin1EnW::new(self, 1)
    }
    #[doc = "Bit 2 - LCD Pin 2 Enable"]
    #[inline(always)]
    pub fn pin_2_en(&mut self) -> Pin2EnW<LcdPen0Spec> {
        Pin2EnW::new(self, 2)
    }
    #[doc = "Bit 3 - LCD Pin 3 Enable"]
    #[inline(always)]
    pub fn pin_3_en(&mut self) -> Pin3EnW<LcdPen0Spec> {
        Pin3EnW::new(self, 3)
    }
    #[doc = "Bit 4 - LCD Pin 4 Enable"]
    #[inline(always)]
    pub fn pin_4_en(&mut self) -> Pin4EnW<LcdPen0Spec> {
        Pin4EnW::new(self, 4)
    }
    #[doc = "Bit 5 - LCD Pin 5 Enable"]
    #[inline(always)]
    pub fn pin_5_en(&mut self) -> Pin5EnW<LcdPen0Spec> {
        Pin5EnW::new(self, 5)
    }
    #[doc = "Bit 6 - LCD Pin 6 Enable"]
    #[inline(always)]
    pub fn pin_6_en(&mut self) -> Pin6EnW<LcdPen0Spec> {
        Pin6EnW::new(self, 6)
    }
    #[doc = "Bit 7 - LCD Pin 7 Enable"]
    #[inline(always)]
    pub fn pin_7_en(&mut self) -> Pin7EnW<LcdPen0Spec> {
        Pin7EnW::new(self, 7)
    }
    #[doc = "Bit 8 - LCD Pin 8 Enable"]
    #[inline(always)]
    pub fn pin_8_en(&mut self) -> Pin8EnW<LcdPen0Spec> {
        Pin8EnW::new(self, 8)
    }
    #[doc = "Bit 9 - LCD Pin 9 Enable"]
    #[inline(always)]
    pub fn pin_9_en(&mut self) -> Pin9EnW<LcdPen0Spec> {
        Pin9EnW::new(self, 9)
    }
    #[doc = "Bit 10 - LCD Pin 10 Enable"]
    #[inline(always)]
    pub fn pin_10_en(&mut self) -> Pin10EnW<LcdPen0Spec> {
        Pin10EnW::new(self, 10)
    }
    #[doc = "Bit 11 - LCD Pin 11 Enable"]
    #[inline(always)]
    pub fn pin_11_en(&mut self) -> Pin11EnW<LcdPen0Spec> {
        Pin11EnW::new(self, 11)
    }
    #[doc = "Bit 12 - LCD Pin 12 Enable"]
    #[inline(always)]
    pub fn pin_12_en(&mut self) -> Pin12EnW<LcdPen0Spec> {
        Pin12EnW::new(self, 12)
    }
    #[doc = "Bit 13 - LCD Pin 13 Enable"]
    #[inline(always)]
    pub fn pin_13_en(&mut self) -> Pin13EnW<LcdPen0Spec> {
        Pin13EnW::new(self, 13)
    }
    #[doc = "Bit 14 - LCD Pin 14 Enable"]
    #[inline(always)]
    pub fn pin_14_en(&mut self) -> Pin14EnW<LcdPen0Spec> {
        Pin14EnW::new(self, 14)
    }
    #[doc = "Bit 15 - LCD Pin 15 Enable"]
    #[inline(always)]
    pub fn pin_15_en(&mut self) -> Pin15EnW<LcdPen0Spec> {
        Pin15EnW::new(self, 15)
    }
    #[doc = "Bit 16 - LCD Pin 16 Enable"]
    #[inline(always)]
    pub fn pin_16_en(&mut self) -> Pin16EnW<LcdPen0Spec> {
        Pin16EnW::new(self, 16)
    }
    #[doc = "Bit 17 - LCD Pin 17 Enable"]
    #[inline(always)]
    pub fn pin_17_en(&mut self) -> Pin17EnW<LcdPen0Spec> {
        Pin17EnW::new(self, 17)
    }
    #[doc = "Bit 18 - LCD Pin 18 Enable"]
    #[inline(always)]
    pub fn pin_18_en(&mut self) -> Pin18EnW<LcdPen0Spec> {
        Pin18EnW::new(self, 18)
    }
    #[doc = "Bit 19 - LCD Pin 19 Enable"]
    #[inline(always)]
    pub fn pin_19_en(&mut self) -> Pin19EnW<LcdPen0Spec> {
        Pin19EnW::new(self, 19)
    }
    #[doc = "Bit 20 - LCD Pin 20 Enable"]
    #[inline(always)]
    pub fn pin_20_en(&mut self) -> Pin20EnW<LcdPen0Spec> {
        Pin20EnW::new(self, 20)
    }
    #[doc = "Bit 21 - LCD Pin 21 Enable"]
    #[inline(always)]
    pub fn pin_21_en(&mut self) -> Pin21EnW<LcdPen0Spec> {
        Pin21EnW::new(self, 21)
    }
    #[doc = "Bit 22 - LCD Pin 22 Enable"]
    #[inline(always)]
    pub fn pin_22_en(&mut self) -> Pin22EnW<LcdPen0Spec> {
        Pin22EnW::new(self, 22)
    }
    #[doc = "Bit 23 - LCD Pin 23 Enable"]
    #[inline(always)]
    pub fn pin_23_en(&mut self) -> Pin23EnW<LcdPen0Spec> {
        Pin23EnW::new(self, 23)
    }
    #[doc = "Bit 24 - LCD Pin 24 Enable"]
    #[inline(always)]
    pub fn pin_24_en(&mut self) -> Pin24EnW<LcdPen0Spec> {
        Pin24EnW::new(self, 24)
    }
    #[doc = "Bit 25 - LCD Pin 25 Enable"]
    #[inline(always)]
    pub fn pin_25_en(&mut self) -> Pin25EnW<LcdPen0Spec> {
        Pin25EnW::new(self, 25)
    }
    #[doc = "Bit 26 - LCD Pin 26 Enable"]
    #[inline(always)]
    pub fn pin_26_en(&mut self) -> Pin26EnW<LcdPen0Spec> {
        Pin26EnW::new(self, 26)
    }
    #[doc = "Bit 27 - LCD Pin 27 Enable"]
    #[inline(always)]
    pub fn pin_27_en(&mut self) -> Pin27EnW<LcdPen0Spec> {
        Pin27EnW::new(self, 27)
    }
    #[doc = "Bit 28 - LCD Pin 28 Enable"]
    #[inline(always)]
    pub fn pin_28_en(&mut self) -> Pin28EnW<LcdPen0Spec> {
        Pin28EnW::new(self, 28)
    }
    #[doc = "Bit 29 - LCD Pin 29 Enable"]
    #[inline(always)]
    pub fn pin_29_en(&mut self) -> Pin29EnW<LcdPen0Spec> {
        Pin29EnW::new(self, 29)
    }
    #[doc = "Bit 30 - LCD Pin 30 Enable"]
    #[inline(always)]
    pub fn pin_30_en(&mut self) -> Pin30EnW<LcdPen0Spec> {
        Pin30EnW::new(self, 30)
    }
    #[doc = "Bit 31 - LCD Pin 31 Enable"]
    #[inline(always)]
    pub fn pin_31_en(&mut self) -> Pin31EnW<LcdPen0Spec> {
        Pin31EnW::new(self, 31)
    }
}
#[doc = "LCD Pin Enable Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_pen0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_pen0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LcdPen0Spec;
impl crate::RegisterSpec for LcdPen0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`lcd_pen0::R`](R) reader structure"]
impl crate::Readable for LcdPen0Spec {}
#[doc = "`write(|w| ..)` method takes [`lcd_pen0::W`](W) writer structure"]
impl crate::Writable for LcdPen0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LCD_PEN0 to value 0"]
impl crate::Resettable for LcdPen0Spec {}

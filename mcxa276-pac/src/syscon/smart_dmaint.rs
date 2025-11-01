#[doc = "Register `SmartDMAINT` reader"]
pub type R = crate::R<SmartDmaintSpec>;
#[doc = "Register `SmartDMAINT` writer"]
pub type W = crate::W<SmartDmaintSpec>;
#[doc = "SmartDMA hijack NVIC IRQ1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int0 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int0> for bool {
    #[inline(always)]
    fn from(variant: Int0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT0` reader - SmartDMA hijack NVIC IRQ1"]
pub type Int0R = crate::BitReader<Int0>;
impl Int0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int0 {
        match self.bits {
            false => Int0::Disable,
            true => Int0::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int0::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int0::Enable
    }
}
#[doc = "Field `INT0` writer - SmartDMA hijack NVIC IRQ1"]
pub type Int0W<'a, REG> = crate::BitWriter<'a, REG, Int0>;
impl<'a, REG> Int0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int0::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int0::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ17\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int1 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int1> for bool {
    #[inline(always)]
    fn from(variant: Int1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT1` reader - SmartDMA hijack NVIC IRQ17"]
pub type Int1R = crate::BitReader<Int1>;
impl Int1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int1 {
        match self.bits {
            false => Int1::Disable,
            true => Int1::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int1::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int1::Enable
    }
}
#[doc = "Field `INT1` writer - SmartDMA hijack NVIC IRQ17"]
pub type Int1W<'a, REG> = crate::BitWriter<'a, REG, Int1>;
impl<'a, REG> Int1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int1::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int1::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ18\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int2 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int2> for bool {
    #[inline(always)]
    fn from(variant: Int2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT2` reader - SmartDMA hijack NVIC IRQ18"]
pub type Int2R = crate::BitReader<Int2>;
impl Int2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int2 {
        match self.bits {
            false => Int2::Disable,
            true => Int2::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int2::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int2::Enable
    }
}
#[doc = "Field `INT2` writer - SmartDMA hijack NVIC IRQ18"]
pub type Int2W<'a, REG> = crate::BitWriter<'a, REG, Int2>;
impl<'a, REG> Int2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int2::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int2::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ29\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int3 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int3> for bool {
    #[inline(always)]
    fn from(variant: Int3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT3` reader - SmartDMA hijack NVIC IRQ29"]
pub type Int3R = crate::BitReader<Int3>;
impl Int3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int3 {
        match self.bits {
            false => Int3::Disable,
            true => Int3::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int3::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int3::Enable
    }
}
#[doc = "Field `INT3` writer - SmartDMA hijack NVIC IRQ29"]
pub type Int3W<'a, REG> = crate::BitWriter<'a, REG, Int3>;
impl<'a, REG> Int3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int3::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int3::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ30\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int4 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int4> for bool {
    #[inline(always)]
    fn from(variant: Int4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT4` reader - SmartDMA hijack NVIC IRQ30"]
pub type Int4R = crate::BitReader<Int4>;
impl Int4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int4 {
        match self.bits {
            false => Int4::Disable,
            true => Int4::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int4::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int4::Enable
    }
}
#[doc = "Field `INT4` writer - SmartDMA hijack NVIC IRQ30"]
pub type Int4W<'a, REG> = crate::BitWriter<'a, REG, Int4>;
impl<'a, REG> Int4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int4::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int4::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ31\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int5 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int5> for bool {
    #[inline(always)]
    fn from(variant: Int5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT5` reader - SmartDMA hijack NVIC IRQ31"]
pub type Int5R = crate::BitReader<Int5>;
impl Int5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int5 {
        match self.bits {
            false => Int5::Disable,
            true => Int5::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int5::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int5::Enable
    }
}
#[doc = "Field `INT5` writer - SmartDMA hijack NVIC IRQ31"]
pub type Int5W<'a, REG> = crate::BitWriter<'a, REG, Int5>;
impl<'a, REG> Int5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int5::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int5::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ32\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int6 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int6> for bool {
    #[inline(always)]
    fn from(variant: Int6) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT6` reader - SmartDMA hijack NVIC IRQ32"]
pub type Int6R = crate::BitReader<Int6>;
impl Int6R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int6 {
        match self.bits {
            false => Int6::Disable,
            true => Int6::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int6::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int6::Enable
    }
}
#[doc = "Field `INT6` writer - SmartDMA hijack NVIC IRQ32"]
pub type Int6W<'a, REG> = crate::BitWriter<'a, REG, Int6>;
impl<'a, REG> Int6W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int6::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int6::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ33\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int7 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int7> for bool {
    #[inline(always)]
    fn from(variant: Int7) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT7` reader - SmartDMA hijack NVIC IRQ33"]
pub type Int7R = crate::BitReader<Int7>;
impl Int7R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int7 {
        match self.bits {
            false => Int7::Disable,
            true => Int7::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int7::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int7::Enable
    }
}
#[doc = "Field `INT7` writer - SmartDMA hijack NVIC IRQ33"]
pub type Int7W<'a, REG> = crate::BitWriter<'a, REG, Int7>;
impl<'a, REG> Int7W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int7::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int7::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ34\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int8 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int8> for bool {
    #[inline(always)]
    fn from(variant: Int8) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT8` reader - SmartDMA hijack NVIC IRQ34"]
pub type Int8R = crate::BitReader<Int8>;
impl Int8R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int8 {
        match self.bits {
            false => Int8::Disable,
            true => Int8::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int8::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int8::Enable
    }
}
#[doc = "Field `INT8` writer - SmartDMA hijack NVIC IRQ34"]
pub type Int8W<'a, REG> = crate::BitWriter<'a, REG, Int8>;
impl<'a, REG> Int8W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int8::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int8::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ35\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int9 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int9> for bool {
    #[inline(always)]
    fn from(variant: Int9) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT9` reader - SmartDMA hijack NVIC IRQ35"]
pub type Int9R = crate::BitReader<Int9>;
impl Int9R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int9 {
        match self.bits {
            false => Int9::Disable,
            true => Int9::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int9::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int9::Enable
    }
}
#[doc = "Field `INT9` writer - SmartDMA hijack NVIC IRQ35"]
pub type Int9W<'a, REG> = crate::BitWriter<'a, REG, Int9>;
impl<'a, REG> Int9W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int9::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int9::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ36\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int10 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int10> for bool {
    #[inline(always)]
    fn from(variant: Int10) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT10` reader - SmartDMA hijack NVIC IRQ36"]
pub type Int10R = crate::BitReader<Int10>;
impl Int10R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int10 {
        match self.bits {
            false => Int10::Disable,
            true => Int10::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int10::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int10::Enable
    }
}
#[doc = "Field `INT10` writer - SmartDMA hijack NVIC IRQ36"]
pub type Int10W<'a, REG> = crate::BitWriter<'a, REG, Int10>;
impl<'a, REG> Int10W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int10::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int10::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ37\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int11 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int11> for bool {
    #[inline(always)]
    fn from(variant: Int11) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT11` reader - SmartDMA hijack NVIC IRQ37"]
pub type Int11R = crate::BitReader<Int11>;
impl Int11R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int11 {
        match self.bits {
            false => Int11::Disable,
            true => Int11::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int11::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int11::Enable
    }
}
#[doc = "Field `INT11` writer - SmartDMA hijack NVIC IRQ37"]
pub type Int11W<'a, REG> = crate::BitWriter<'a, REG, Int11>;
impl<'a, REG> Int11W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int11::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int11::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ38\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int12 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int12> for bool {
    #[inline(always)]
    fn from(variant: Int12) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT12` reader - SmartDMA hijack NVIC IRQ38"]
pub type Int12R = crate::BitReader<Int12>;
impl Int12R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int12 {
        match self.bits {
            false => Int12::Disable,
            true => Int12::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int12::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int12::Enable
    }
}
#[doc = "Field `INT12` writer - SmartDMA hijack NVIC IRQ38"]
pub type Int12W<'a, REG> = crate::BitWriter<'a, REG, Int12>;
impl<'a, REG> Int12W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int12::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int12::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ39\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int13 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int13> for bool {
    #[inline(always)]
    fn from(variant: Int13) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT13` reader - SmartDMA hijack NVIC IRQ39"]
pub type Int13R = crate::BitReader<Int13>;
impl Int13R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int13 {
        match self.bits {
            false => Int13::Disable,
            true => Int13::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int13::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int13::Enable
    }
}
#[doc = "Field `INT13` writer - SmartDMA hijack NVIC IRQ39"]
pub type Int13W<'a, REG> = crate::BitWriter<'a, REG, Int13>;
impl<'a, REG> Int13W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int13::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int13::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ40\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int14 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int14> for bool {
    #[inline(always)]
    fn from(variant: Int14) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT14` reader - SmartDMA hijack NVIC IRQ40"]
pub type Int14R = crate::BitReader<Int14>;
impl Int14R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int14 {
        match self.bits {
            false => Int14::Disable,
            true => Int14::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int14::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int14::Enable
    }
}
#[doc = "Field `INT14` writer - SmartDMA hijack NVIC IRQ40"]
pub type Int14W<'a, REG> = crate::BitWriter<'a, REG, Int14>;
impl<'a, REG> Int14W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int14::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int14::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ41\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int15 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int15> for bool {
    #[inline(always)]
    fn from(variant: Int15) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT15` reader - SmartDMA hijack NVIC IRQ41"]
pub type Int15R = crate::BitReader<Int15>;
impl Int15R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int15 {
        match self.bits {
            false => Int15::Disable,
            true => Int15::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int15::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int15::Enable
    }
}
#[doc = "Field `INT15` writer - SmartDMA hijack NVIC IRQ41"]
pub type Int15W<'a, REG> = crate::BitWriter<'a, REG, Int15>;
impl<'a, REG> Int15W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int15::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int15::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ42\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int16 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int16> for bool {
    #[inline(always)]
    fn from(variant: Int16) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT16` reader - SmartDMA hijack NVIC IRQ42"]
pub type Int16R = crate::BitReader<Int16>;
impl Int16R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int16 {
        match self.bits {
            false => Int16::Disable,
            true => Int16::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int16::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int16::Enable
    }
}
#[doc = "Field `INT16` writer - SmartDMA hijack NVIC IRQ42"]
pub type Int16W<'a, REG> = crate::BitWriter<'a, REG, Int16>;
impl<'a, REG> Int16W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int16::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int16::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ45\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int17 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int17> for bool {
    #[inline(always)]
    fn from(variant: Int17) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT17` reader - SmartDMA hijack NVIC IRQ45"]
pub type Int17R = crate::BitReader<Int17>;
impl Int17R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int17 {
        match self.bits {
            false => Int17::Disable,
            true => Int17::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int17::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int17::Enable
    }
}
#[doc = "Field `INT17` writer - SmartDMA hijack NVIC IRQ45"]
pub type Int17W<'a, REG> = crate::BitWriter<'a, REG, Int17>;
impl<'a, REG> Int17W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int17::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int17::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ47\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int18 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int18> for bool {
    #[inline(always)]
    fn from(variant: Int18) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT18` reader - SmartDMA hijack NVIC IRQ47"]
pub type Int18R = crate::BitReader<Int18>;
impl Int18R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int18 {
        match self.bits {
            false => Int18::Disable,
            true => Int18::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int18::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int18::Enable
    }
}
#[doc = "Field `INT18` writer - SmartDMA hijack NVIC IRQ47"]
pub type Int18W<'a, REG> = crate::BitWriter<'a, REG, Int18>;
impl<'a, REG> Int18W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int18::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int18::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ50\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int19 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int19> for bool {
    #[inline(always)]
    fn from(variant: Int19) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT19` reader - SmartDMA hijack NVIC IRQ50"]
pub type Int19R = crate::BitReader<Int19>;
impl Int19R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int19 {
        match self.bits {
            false => Int19::Disable,
            true => Int19::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int19::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int19::Enable
    }
}
#[doc = "Field `INT19` writer - SmartDMA hijack NVIC IRQ50"]
pub type Int19W<'a, REG> = crate::BitWriter<'a, REG, Int19>;
impl<'a, REG> Int19W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int19::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int19::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ51\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int20 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int20> for bool {
    #[inline(always)]
    fn from(variant: Int20) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT20` reader - SmartDMA hijack NVIC IRQ51"]
pub type Int20R = crate::BitReader<Int20>;
impl Int20R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int20 {
        match self.bits {
            false => Int20::Disable,
            true => Int20::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int20::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int20::Enable
    }
}
#[doc = "Field `INT20` writer - SmartDMA hijack NVIC IRQ51"]
pub type Int20W<'a, REG> = crate::BitWriter<'a, REG, Int20>;
impl<'a, REG> Int20W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int20::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int20::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ66\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int21 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int21> for bool {
    #[inline(always)]
    fn from(variant: Int21) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT21` reader - SmartDMA hijack NVIC IRQ66"]
pub type Int21R = crate::BitReader<Int21>;
impl Int21R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int21 {
        match self.bits {
            false => Int21::Disable,
            true => Int21::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int21::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int21::Enable
    }
}
#[doc = "Field `INT21` writer - SmartDMA hijack NVIC IRQ66"]
pub type Int21W<'a, REG> = crate::BitWriter<'a, REG, Int21>;
impl<'a, REG> Int21W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int21::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int21::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ67\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int22 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int22> for bool {
    #[inline(always)]
    fn from(variant: Int22) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT22` reader - SmartDMA hijack NVIC IRQ67"]
pub type Int22R = crate::BitReader<Int22>;
impl Int22R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int22 {
        match self.bits {
            false => Int22::Disable,
            true => Int22::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int22::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int22::Enable
    }
}
#[doc = "Field `INT22` writer - SmartDMA hijack NVIC IRQ67"]
pub type Int22W<'a, REG> = crate::BitWriter<'a, REG, Int22>;
impl<'a, REG> Int22W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int22::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int22::Enable)
    }
}
#[doc = "SmartDMA hijack NVIC IRQ77\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int23 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Int23> for bool {
    #[inline(always)]
    fn from(variant: Int23) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT23` reader - SmartDMA hijack NVIC IRQ77"]
pub type Int23R = crate::BitReader<Int23>;
impl Int23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int23 {
        match self.bits {
            false => Int23::Disable,
            true => Int23::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Int23::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Int23::Enable
    }
}
#[doc = "Field `INT23` writer - SmartDMA hijack NVIC IRQ77"]
pub type Int23W<'a, REG> = crate::BitWriter<'a, REG, Int23>;
impl<'a, REG> Int23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Int23::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Int23::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - SmartDMA hijack NVIC IRQ1"]
    #[inline(always)]
    pub fn int0(&self) -> Int0R {
        Int0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - SmartDMA hijack NVIC IRQ17"]
    #[inline(always)]
    pub fn int1(&self) -> Int1R {
        Int1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - SmartDMA hijack NVIC IRQ18"]
    #[inline(always)]
    pub fn int2(&self) -> Int2R {
        Int2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - SmartDMA hijack NVIC IRQ29"]
    #[inline(always)]
    pub fn int3(&self) -> Int3R {
        Int3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - SmartDMA hijack NVIC IRQ30"]
    #[inline(always)]
    pub fn int4(&self) -> Int4R {
        Int4R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - SmartDMA hijack NVIC IRQ31"]
    #[inline(always)]
    pub fn int5(&self) -> Int5R {
        Int5R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - SmartDMA hijack NVIC IRQ32"]
    #[inline(always)]
    pub fn int6(&self) -> Int6R {
        Int6R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - SmartDMA hijack NVIC IRQ33"]
    #[inline(always)]
    pub fn int7(&self) -> Int7R {
        Int7R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - SmartDMA hijack NVIC IRQ34"]
    #[inline(always)]
    pub fn int8(&self) -> Int8R {
        Int8R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - SmartDMA hijack NVIC IRQ35"]
    #[inline(always)]
    pub fn int9(&self) -> Int9R {
        Int9R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - SmartDMA hijack NVIC IRQ36"]
    #[inline(always)]
    pub fn int10(&self) -> Int10R {
        Int10R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - SmartDMA hijack NVIC IRQ37"]
    #[inline(always)]
    pub fn int11(&self) -> Int11R {
        Int11R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - SmartDMA hijack NVIC IRQ38"]
    #[inline(always)]
    pub fn int12(&self) -> Int12R {
        Int12R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - SmartDMA hijack NVIC IRQ39"]
    #[inline(always)]
    pub fn int13(&self) -> Int13R {
        Int13R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - SmartDMA hijack NVIC IRQ40"]
    #[inline(always)]
    pub fn int14(&self) -> Int14R {
        Int14R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - SmartDMA hijack NVIC IRQ41"]
    #[inline(always)]
    pub fn int15(&self) -> Int15R {
        Int15R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - SmartDMA hijack NVIC IRQ42"]
    #[inline(always)]
    pub fn int16(&self) -> Int16R {
        Int16R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - SmartDMA hijack NVIC IRQ45"]
    #[inline(always)]
    pub fn int17(&self) -> Int17R {
        Int17R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - SmartDMA hijack NVIC IRQ47"]
    #[inline(always)]
    pub fn int18(&self) -> Int18R {
        Int18R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - SmartDMA hijack NVIC IRQ50"]
    #[inline(always)]
    pub fn int19(&self) -> Int19R {
        Int19R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - SmartDMA hijack NVIC IRQ51"]
    #[inline(always)]
    pub fn int20(&self) -> Int20R {
        Int20R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - SmartDMA hijack NVIC IRQ66"]
    #[inline(always)]
    pub fn int21(&self) -> Int21R {
        Int21R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - SmartDMA hijack NVIC IRQ67"]
    #[inline(always)]
    pub fn int22(&self) -> Int22R {
        Int22R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - SmartDMA hijack NVIC IRQ77"]
    #[inline(always)]
    pub fn int23(&self) -> Int23R {
        Int23R::new(((self.bits >> 23) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - SmartDMA hijack NVIC IRQ1"]
    #[inline(always)]
    pub fn int0(&mut self) -> Int0W<SmartDmaintSpec> {
        Int0W::new(self, 0)
    }
    #[doc = "Bit 1 - SmartDMA hijack NVIC IRQ17"]
    #[inline(always)]
    pub fn int1(&mut self) -> Int1W<SmartDmaintSpec> {
        Int1W::new(self, 1)
    }
    #[doc = "Bit 2 - SmartDMA hijack NVIC IRQ18"]
    #[inline(always)]
    pub fn int2(&mut self) -> Int2W<SmartDmaintSpec> {
        Int2W::new(self, 2)
    }
    #[doc = "Bit 3 - SmartDMA hijack NVIC IRQ29"]
    #[inline(always)]
    pub fn int3(&mut self) -> Int3W<SmartDmaintSpec> {
        Int3W::new(self, 3)
    }
    #[doc = "Bit 4 - SmartDMA hijack NVIC IRQ30"]
    #[inline(always)]
    pub fn int4(&mut self) -> Int4W<SmartDmaintSpec> {
        Int4W::new(self, 4)
    }
    #[doc = "Bit 5 - SmartDMA hijack NVIC IRQ31"]
    #[inline(always)]
    pub fn int5(&mut self) -> Int5W<SmartDmaintSpec> {
        Int5W::new(self, 5)
    }
    #[doc = "Bit 6 - SmartDMA hijack NVIC IRQ32"]
    #[inline(always)]
    pub fn int6(&mut self) -> Int6W<SmartDmaintSpec> {
        Int6W::new(self, 6)
    }
    #[doc = "Bit 7 - SmartDMA hijack NVIC IRQ33"]
    #[inline(always)]
    pub fn int7(&mut self) -> Int7W<SmartDmaintSpec> {
        Int7W::new(self, 7)
    }
    #[doc = "Bit 8 - SmartDMA hijack NVIC IRQ34"]
    #[inline(always)]
    pub fn int8(&mut self) -> Int8W<SmartDmaintSpec> {
        Int8W::new(self, 8)
    }
    #[doc = "Bit 9 - SmartDMA hijack NVIC IRQ35"]
    #[inline(always)]
    pub fn int9(&mut self) -> Int9W<SmartDmaintSpec> {
        Int9W::new(self, 9)
    }
    #[doc = "Bit 10 - SmartDMA hijack NVIC IRQ36"]
    #[inline(always)]
    pub fn int10(&mut self) -> Int10W<SmartDmaintSpec> {
        Int10W::new(self, 10)
    }
    #[doc = "Bit 11 - SmartDMA hijack NVIC IRQ37"]
    #[inline(always)]
    pub fn int11(&mut self) -> Int11W<SmartDmaintSpec> {
        Int11W::new(self, 11)
    }
    #[doc = "Bit 12 - SmartDMA hijack NVIC IRQ38"]
    #[inline(always)]
    pub fn int12(&mut self) -> Int12W<SmartDmaintSpec> {
        Int12W::new(self, 12)
    }
    #[doc = "Bit 13 - SmartDMA hijack NVIC IRQ39"]
    #[inline(always)]
    pub fn int13(&mut self) -> Int13W<SmartDmaintSpec> {
        Int13W::new(self, 13)
    }
    #[doc = "Bit 14 - SmartDMA hijack NVIC IRQ40"]
    #[inline(always)]
    pub fn int14(&mut self) -> Int14W<SmartDmaintSpec> {
        Int14W::new(self, 14)
    }
    #[doc = "Bit 15 - SmartDMA hijack NVIC IRQ41"]
    #[inline(always)]
    pub fn int15(&mut self) -> Int15W<SmartDmaintSpec> {
        Int15W::new(self, 15)
    }
    #[doc = "Bit 16 - SmartDMA hijack NVIC IRQ42"]
    #[inline(always)]
    pub fn int16(&mut self) -> Int16W<SmartDmaintSpec> {
        Int16W::new(self, 16)
    }
    #[doc = "Bit 17 - SmartDMA hijack NVIC IRQ45"]
    #[inline(always)]
    pub fn int17(&mut self) -> Int17W<SmartDmaintSpec> {
        Int17W::new(self, 17)
    }
    #[doc = "Bit 18 - SmartDMA hijack NVIC IRQ47"]
    #[inline(always)]
    pub fn int18(&mut self) -> Int18W<SmartDmaintSpec> {
        Int18W::new(self, 18)
    }
    #[doc = "Bit 19 - SmartDMA hijack NVIC IRQ50"]
    #[inline(always)]
    pub fn int19(&mut self) -> Int19W<SmartDmaintSpec> {
        Int19W::new(self, 19)
    }
    #[doc = "Bit 20 - SmartDMA hijack NVIC IRQ51"]
    #[inline(always)]
    pub fn int20(&mut self) -> Int20W<SmartDmaintSpec> {
        Int20W::new(self, 20)
    }
    #[doc = "Bit 21 - SmartDMA hijack NVIC IRQ66"]
    #[inline(always)]
    pub fn int21(&mut self) -> Int21W<SmartDmaintSpec> {
        Int21W::new(self, 21)
    }
    #[doc = "Bit 22 - SmartDMA hijack NVIC IRQ67"]
    #[inline(always)]
    pub fn int22(&mut self) -> Int22W<SmartDmaintSpec> {
        Int22W::new(self, 22)
    }
    #[doc = "Bit 23 - SmartDMA hijack NVIC IRQ77"]
    #[inline(always)]
    pub fn int23(&mut self) -> Int23W<SmartDmaintSpec> {
        Int23W::new(self, 23)
    }
}
#[doc = "SmartDMA Interrupt Hijack\n\nYou can [`read`](crate::Reg::read) this register and get [`smart_dmaint::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`smart_dmaint::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SmartDmaintSpec;
impl crate::RegisterSpec for SmartDmaintSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`smart_dmaint::R`](R) reader structure"]
impl crate::Readable for SmartDmaintSpec {}
#[doc = "`write(|w| ..)` method takes [`smart_dmaint::W`](W) writer structure"]
impl crate::Writable for SmartDmaintSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SmartDMAINT to value 0"]
impl crate::Resettable for SmartDmaintSpec {}

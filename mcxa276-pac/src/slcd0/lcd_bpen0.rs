#[doc = "Register `LCD_BPEN0` reader"]
pub type R = crate::R<LcdBpen0Spec>;
#[doc = "Register `LCD_BPEN0` writer"]
pub type W = crate::W<LcdBpen0Spec>;
#[doc = "LCD Pin 0 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin0Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin0Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin0Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_0_BPEN` reader - LCD Pin 0 Back Plane Enable"]
pub type Pin0BpenR = crate::BitReader<Pin0Bpen>;
impl Pin0BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin0Bpen {
        match self.bits {
            false => Pin0Bpen::Fp,
            true => Pin0Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin0Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin0Bpen::Bp
    }
}
#[doc = "Field `PIN_0_BPEN` writer - LCD Pin 0 Back Plane Enable"]
pub type Pin0BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin0Bpen>;
impl<'a, REG> Pin0BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin0Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin0Bpen::Bp)
    }
}
#[doc = "LCD Pin 1 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin1Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin1Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin1Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_1_BPEN` reader - LCD Pin 1 Back Plane Enable"]
pub type Pin1BpenR = crate::BitReader<Pin1Bpen>;
impl Pin1BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin1Bpen {
        match self.bits {
            false => Pin1Bpen::Fp,
            true => Pin1Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin1Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin1Bpen::Bp
    }
}
#[doc = "Field `PIN_1_BPEN` writer - LCD Pin 1 Back Plane Enable"]
pub type Pin1BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin1Bpen>;
impl<'a, REG> Pin1BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin1Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin1Bpen::Bp)
    }
}
#[doc = "LCD Pin 2 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin2Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin2Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin2Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_2_BPEN` reader - LCD Pin 2 Back Plane Enable"]
pub type Pin2BpenR = crate::BitReader<Pin2Bpen>;
impl Pin2BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin2Bpen {
        match self.bits {
            false => Pin2Bpen::Fp,
            true => Pin2Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin2Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin2Bpen::Bp
    }
}
#[doc = "Field `PIN_2_BPEN` writer - LCD Pin 2 Back Plane Enable"]
pub type Pin2BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin2Bpen>;
impl<'a, REG> Pin2BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin2Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin2Bpen::Bp)
    }
}
#[doc = "LCD Pin 3 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin3Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin3Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin3Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_3_BPEN` reader - LCD Pin 3 Back Plane Enable"]
pub type Pin3BpenR = crate::BitReader<Pin3Bpen>;
impl Pin3BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin3Bpen {
        match self.bits {
            false => Pin3Bpen::Fp,
            true => Pin3Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin3Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin3Bpen::Bp
    }
}
#[doc = "Field `PIN_3_BPEN` writer - LCD Pin 3 Back Plane Enable"]
pub type Pin3BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin3Bpen>;
impl<'a, REG> Pin3BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin3Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin3Bpen::Bp)
    }
}
#[doc = "LCD Pin 4 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin4Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin4Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin4Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_4_BPEN` reader - LCD Pin 4 Back Plane Enable"]
pub type Pin4BpenR = crate::BitReader<Pin4Bpen>;
impl Pin4BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin4Bpen {
        match self.bits {
            false => Pin4Bpen::Fp,
            true => Pin4Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin4Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin4Bpen::Bp
    }
}
#[doc = "Field `PIN_4_BPEN` writer - LCD Pin 4 Back Plane Enable"]
pub type Pin4BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin4Bpen>;
impl<'a, REG> Pin4BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin4Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin4Bpen::Bp)
    }
}
#[doc = "LCD Pin 5 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin5Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin5Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin5Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_5_BPEN` reader - LCD Pin 5 Back Plane Enable"]
pub type Pin5BpenR = crate::BitReader<Pin5Bpen>;
impl Pin5BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin5Bpen {
        match self.bits {
            false => Pin5Bpen::Fp,
            true => Pin5Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin5Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin5Bpen::Bp
    }
}
#[doc = "Field `PIN_5_BPEN` writer - LCD Pin 5 Back Plane Enable"]
pub type Pin5BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin5Bpen>;
impl<'a, REG> Pin5BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin5Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin5Bpen::Bp)
    }
}
#[doc = "LCD Pin 6 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin6Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin6Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin6Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_6_BPEN` reader - LCD Pin 6 Back Plane Enable"]
pub type Pin6BpenR = crate::BitReader<Pin6Bpen>;
impl Pin6BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin6Bpen {
        match self.bits {
            false => Pin6Bpen::Fp,
            true => Pin6Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin6Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin6Bpen::Bp
    }
}
#[doc = "Field `PIN_6_BPEN` writer - LCD Pin 6 Back Plane Enable"]
pub type Pin6BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin6Bpen>;
impl<'a, REG> Pin6BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin6Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin6Bpen::Bp)
    }
}
#[doc = "LCD Pin 7 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin7Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin7Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin7Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_7_BPEN` reader - LCD Pin 7 Back Plane Enable"]
pub type Pin7BpenR = crate::BitReader<Pin7Bpen>;
impl Pin7BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin7Bpen {
        match self.bits {
            false => Pin7Bpen::Fp,
            true => Pin7Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin7Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin7Bpen::Bp
    }
}
#[doc = "Field `PIN_7_BPEN` writer - LCD Pin 7 Back Plane Enable"]
pub type Pin7BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin7Bpen>;
impl<'a, REG> Pin7BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin7Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin7Bpen::Bp)
    }
}
#[doc = "LCD Pin 8 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin8Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin8Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin8Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_8_BPEN` reader - LCD Pin 8 Back Plane Enable"]
pub type Pin8BpenR = crate::BitReader<Pin8Bpen>;
impl Pin8BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin8Bpen {
        match self.bits {
            false => Pin8Bpen::Fp,
            true => Pin8Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin8Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin8Bpen::Bp
    }
}
#[doc = "Field `PIN_8_BPEN` writer - LCD Pin 8 Back Plane Enable"]
pub type Pin8BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin8Bpen>;
impl<'a, REG> Pin8BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin8Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin8Bpen::Bp)
    }
}
#[doc = "LCD Pin 9 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin9Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin9Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin9Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_9_BPEN` reader - LCD Pin 9 Back Plane Enable"]
pub type Pin9BpenR = crate::BitReader<Pin9Bpen>;
impl Pin9BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin9Bpen {
        match self.bits {
            false => Pin9Bpen::Fp,
            true => Pin9Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin9Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin9Bpen::Bp
    }
}
#[doc = "Field `PIN_9_BPEN` writer - LCD Pin 9 Back Plane Enable"]
pub type Pin9BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin9Bpen>;
impl<'a, REG> Pin9BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin9Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin9Bpen::Bp)
    }
}
#[doc = "LCD Pin 10 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin10Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin10Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin10Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_10_BPEN` reader - LCD Pin 10 Back Plane Enable"]
pub type Pin10BpenR = crate::BitReader<Pin10Bpen>;
impl Pin10BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin10Bpen {
        match self.bits {
            false => Pin10Bpen::Fp,
            true => Pin10Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin10Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin10Bpen::Bp
    }
}
#[doc = "Field `PIN_10_BPEN` writer - LCD Pin 10 Back Plane Enable"]
pub type Pin10BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin10Bpen>;
impl<'a, REG> Pin10BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin10Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin10Bpen::Bp)
    }
}
#[doc = "LCD Pin 11 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin11Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin11Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin11Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_11_BPEN` reader - LCD Pin 11 Back Plane Enable"]
pub type Pin11BpenR = crate::BitReader<Pin11Bpen>;
impl Pin11BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin11Bpen {
        match self.bits {
            false => Pin11Bpen::Fp,
            true => Pin11Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin11Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin11Bpen::Bp
    }
}
#[doc = "Field `PIN_11_BPEN` writer - LCD Pin 11 Back Plane Enable"]
pub type Pin11BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin11Bpen>;
impl<'a, REG> Pin11BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin11Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin11Bpen::Bp)
    }
}
#[doc = "LCD Pin 12 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin12Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin12Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin12Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_12_BPEN` reader - LCD Pin 12 Back Plane Enable"]
pub type Pin12BpenR = crate::BitReader<Pin12Bpen>;
impl Pin12BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin12Bpen {
        match self.bits {
            false => Pin12Bpen::Fp,
            true => Pin12Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin12Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin12Bpen::Bp
    }
}
#[doc = "Field `PIN_12_BPEN` writer - LCD Pin 12 Back Plane Enable"]
pub type Pin12BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin12Bpen>;
impl<'a, REG> Pin12BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin12Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin12Bpen::Bp)
    }
}
#[doc = "LCD Pin 13 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin13Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin13Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin13Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_13_BPEN` reader - LCD Pin 13 Back Plane Enable"]
pub type Pin13BpenR = crate::BitReader<Pin13Bpen>;
impl Pin13BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin13Bpen {
        match self.bits {
            false => Pin13Bpen::Fp,
            true => Pin13Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin13Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin13Bpen::Bp
    }
}
#[doc = "Field `PIN_13_BPEN` writer - LCD Pin 13 Back Plane Enable"]
pub type Pin13BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin13Bpen>;
impl<'a, REG> Pin13BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin13Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin13Bpen::Bp)
    }
}
#[doc = "LCD Pin 14 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin14Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin14Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin14Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_14_BPEN` reader - LCD Pin 14 Back Plane Enable"]
pub type Pin14BpenR = crate::BitReader<Pin14Bpen>;
impl Pin14BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin14Bpen {
        match self.bits {
            false => Pin14Bpen::Fp,
            true => Pin14Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin14Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin14Bpen::Bp
    }
}
#[doc = "Field `PIN_14_BPEN` writer - LCD Pin 14 Back Plane Enable"]
pub type Pin14BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin14Bpen>;
impl<'a, REG> Pin14BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin14Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin14Bpen::Bp)
    }
}
#[doc = "LCD Pin 15 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin15Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin15Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin15Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_15_BPEN` reader - LCD Pin 15 Back Plane Enable"]
pub type Pin15BpenR = crate::BitReader<Pin15Bpen>;
impl Pin15BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin15Bpen {
        match self.bits {
            false => Pin15Bpen::Fp,
            true => Pin15Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin15Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin15Bpen::Bp
    }
}
#[doc = "Field `PIN_15_BPEN` writer - LCD Pin 15 Back Plane Enable"]
pub type Pin15BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin15Bpen>;
impl<'a, REG> Pin15BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin15Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin15Bpen::Bp)
    }
}
#[doc = "LCD Pin 16 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin16Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin16Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin16Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_16_BPEN` reader - LCD Pin 16 Back Plane Enable"]
pub type Pin16BpenR = crate::BitReader<Pin16Bpen>;
impl Pin16BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin16Bpen {
        match self.bits {
            false => Pin16Bpen::Fp,
            true => Pin16Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin16Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin16Bpen::Bp
    }
}
#[doc = "Field `PIN_16_BPEN` writer - LCD Pin 16 Back Plane Enable"]
pub type Pin16BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin16Bpen>;
impl<'a, REG> Pin16BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin16Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin16Bpen::Bp)
    }
}
#[doc = "LCD Pin 17 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin17Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin17Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin17Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_17_BPEN` reader - LCD Pin 17 Back Plane Enable"]
pub type Pin17BpenR = crate::BitReader<Pin17Bpen>;
impl Pin17BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin17Bpen {
        match self.bits {
            false => Pin17Bpen::Fp,
            true => Pin17Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin17Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin17Bpen::Bp
    }
}
#[doc = "Field `PIN_17_BPEN` writer - LCD Pin 17 Back Plane Enable"]
pub type Pin17BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin17Bpen>;
impl<'a, REG> Pin17BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin17Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin17Bpen::Bp)
    }
}
#[doc = "LCD Pin 18 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin18Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin18Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin18Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_18_BPEN` reader - LCD Pin 18 Back Plane Enable"]
pub type Pin18BpenR = crate::BitReader<Pin18Bpen>;
impl Pin18BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin18Bpen {
        match self.bits {
            false => Pin18Bpen::Fp,
            true => Pin18Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin18Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin18Bpen::Bp
    }
}
#[doc = "Field `PIN_18_BPEN` writer - LCD Pin 18 Back Plane Enable"]
pub type Pin18BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin18Bpen>;
impl<'a, REG> Pin18BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin18Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin18Bpen::Bp)
    }
}
#[doc = "LCD Pin 19 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin19Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin19Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin19Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_19_BPEN` reader - LCD Pin 19 Back Plane Enable"]
pub type Pin19BpenR = crate::BitReader<Pin19Bpen>;
impl Pin19BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin19Bpen {
        match self.bits {
            false => Pin19Bpen::Fp,
            true => Pin19Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin19Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin19Bpen::Bp
    }
}
#[doc = "Field `PIN_19_BPEN` writer - LCD Pin 19 Back Plane Enable"]
pub type Pin19BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin19Bpen>;
impl<'a, REG> Pin19BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin19Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin19Bpen::Bp)
    }
}
#[doc = "LCD Pin 20 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin20Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin20Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin20Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_20_BPEN` reader - LCD Pin 20 Back Plane Enable"]
pub type Pin20BpenR = crate::BitReader<Pin20Bpen>;
impl Pin20BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin20Bpen {
        match self.bits {
            false => Pin20Bpen::Fp,
            true => Pin20Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin20Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin20Bpen::Bp
    }
}
#[doc = "Field `PIN_20_BPEN` writer - LCD Pin 20 Back Plane Enable"]
pub type Pin20BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin20Bpen>;
impl<'a, REG> Pin20BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin20Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin20Bpen::Bp)
    }
}
#[doc = "LCD Pin 21 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin21Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin21Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin21Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_21_BPEN` reader - LCD Pin 21 Back Plane Enable"]
pub type Pin21BpenR = crate::BitReader<Pin21Bpen>;
impl Pin21BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin21Bpen {
        match self.bits {
            false => Pin21Bpen::Fp,
            true => Pin21Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin21Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin21Bpen::Bp
    }
}
#[doc = "Field `PIN_21_BPEN` writer - LCD Pin 21 Back Plane Enable"]
pub type Pin21BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin21Bpen>;
impl<'a, REG> Pin21BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin21Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin21Bpen::Bp)
    }
}
#[doc = "LCD Pin 22 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin22Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin22Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin22Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_22_BPEN` reader - LCD Pin 22 Back Plane Enable"]
pub type Pin22BpenR = crate::BitReader<Pin22Bpen>;
impl Pin22BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin22Bpen {
        match self.bits {
            false => Pin22Bpen::Fp,
            true => Pin22Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin22Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin22Bpen::Bp
    }
}
#[doc = "Field `PIN_22_BPEN` writer - LCD Pin 22 Back Plane Enable"]
pub type Pin22BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin22Bpen>;
impl<'a, REG> Pin22BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin22Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin22Bpen::Bp)
    }
}
#[doc = "LCD Pin 23 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin23Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin23Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin23Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_23_BPEN` reader - LCD Pin 23 Back Plane Enable"]
pub type Pin23BpenR = crate::BitReader<Pin23Bpen>;
impl Pin23BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin23Bpen {
        match self.bits {
            false => Pin23Bpen::Fp,
            true => Pin23Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin23Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin23Bpen::Bp
    }
}
#[doc = "Field `PIN_23_BPEN` writer - LCD Pin 23 Back Plane Enable"]
pub type Pin23BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin23Bpen>;
impl<'a, REG> Pin23BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin23Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin23Bpen::Bp)
    }
}
#[doc = "LCD Pin 24 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin24Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin24Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin24Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_24_BPEN` reader - LCD Pin 24 Back Plane Enable"]
pub type Pin24BpenR = crate::BitReader<Pin24Bpen>;
impl Pin24BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin24Bpen {
        match self.bits {
            false => Pin24Bpen::Fp,
            true => Pin24Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin24Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin24Bpen::Bp
    }
}
#[doc = "Field `PIN_24_BPEN` writer - LCD Pin 24 Back Plane Enable"]
pub type Pin24BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin24Bpen>;
impl<'a, REG> Pin24BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin24Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin24Bpen::Bp)
    }
}
#[doc = "LCD Pin 25 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin25Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin25Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin25Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_25_BPEN` reader - LCD Pin 25 Back Plane Enable"]
pub type Pin25BpenR = crate::BitReader<Pin25Bpen>;
impl Pin25BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin25Bpen {
        match self.bits {
            false => Pin25Bpen::Fp,
            true => Pin25Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin25Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin25Bpen::Bp
    }
}
#[doc = "Field `PIN_25_BPEN` writer - LCD Pin 25 Back Plane Enable"]
pub type Pin25BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin25Bpen>;
impl<'a, REG> Pin25BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin25Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin25Bpen::Bp)
    }
}
#[doc = "LCD Pin 26 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin26Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin26Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin26Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_26_BPEN` reader - LCD Pin 26 Back Plane Enable"]
pub type Pin26BpenR = crate::BitReader<Pin26Bpen>;
impl Pin26BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin26Bpen {
        match self.bits {
            false => Pin26Bpen::Fp,
            true => Pin26Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin26Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin26Bpen::Bp
    }
}
#[doc = "Field `PIN_26_BPEN` writer - LCD Pin 26 Back Plane Enable"]
pub type Pin26BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin26Bpen>;
impl<'a, REG> Pin26BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin26Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin26Bpen::Bp)
    }
}
#[doc = "LCD Pin 27 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin27Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin27Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin27Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_27_BPEN` reader - LCD Pin 27 Back Plane Enable"]
pub type Pin27BpenR = crate::BitReader<Pin27Bpen>;
impl Pin27BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin27Bpen {
        match self.bits {
            false => Pin27Bpen::Fp,
            true => Pin27Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin27Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin27Bpen::Bp
    }
}
#[doc = "Field `PIN_27_BPEN` writer - LCD Pin 27 Back Plane Enable"]
pub type Pin27BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin27Bpen>;
impl<'a, REG> Pin27BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin27Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin27Bpen::Bp)
    }
}
#[doc = "LCD Pin 28 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin28Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin28Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin28Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_28_BPEN` reader - LCD Pin 28 Back Plane Enable"]
pub type Pin28BpenR = crate::BitReader<Pin28Bpen>;
impl Pin28BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin28Bpen {
        match self.bits {
            false => Pin28Bpen::Fp,
            true => Pin28Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin28Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin28Bpen::Bp
    }
}
#[doc = "Field `PIN_28_BPEN` writer - LCD Pin 28 Back Plane Enable"]
pub type Pin28BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin28Bpen>;
impl<'a, REG> Pin28BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin28Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin28Bpen::Bp)
    }
}
#[doc = "LCD Pin 29 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin29Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin29Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin29Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_29_BPEN` reader - LCD Pin 29 Back Plane Enable"]
pub type Pin29BpenR = crate::BitReader<Pin29Bpen>;
impl Pin29BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin29Bpen {
        match self.bits {
            false => Pin29Bpen::Fp,
            true => Pin29Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin29Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin29Bpen::Bp
    }
}
#[doc = "Field `PIN_29_BPEN` writer - LCD Pin 29 Back Plane Enable"]
pub type Pin29BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin29Bpen>;
impl<'a, REG> Pin29BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin29Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin29Bpen::Bp)
    }
}
#[doc = "LCD Pin 30 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin30Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin30Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin30Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_30_BPEN` reader - LCD Pin 30 Back Plane Enable"]
pub type Pin30BpenR = crate::BitReader<Pin30Bpen>;
impl Pin30BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin30Bpen {
        match self.bits {
            false => Pin30Bpen::Fp,
            true => Pin30Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin30Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin30Bpen::Bp
    }
}
#[doc = "Field `PIN_30_BPEN` writer - LCD Pin 30 Back Plane Enable"]
pub type Pin30BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin30Bpen>;
impl<'a, REG> Pin30BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin30Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin30Bpen::Bp)
    }
}
#[doc = "LCD Pin 31 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin31Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin31Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin31Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_31_BPEN` reader - LCD Pin 31 Back Plane Enable"]
pub type Pin31BpenR = crate::BitReader<Pin31Bpen>;
impl Pin31BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin31Bpen {
        match self.bits {
            false => Pin31Bpen::Fp,
            true => Pin31Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin31Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin31Bpen::Bp
    }
}
#[doc = "Field `PIN_31_BPEN` writer - LCD Pin 31 Back Plane Enable"]
pub type Pin31BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin31Bpen>;
impl<'a, REG> Pin31BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin31Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin31Bpen::Bp)
    }
}
impl R {
    #[doc = "Bit 0 - LCD Pin 0 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_0_bpen(&self) -> Pin0BpenR {
        Pin0BpenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - LCD Pin 1 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_1_bpen(&self) -> Pin1BpenR {
        Pin1BpenR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - LCD Pin 2 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_2_bpen(&self) -> Pin2BpenR {
        Pin2BpenR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - LCD Pin 3 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_3_bpen(&self) -> Pin3BpenR {
        Pin3BpenR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - LCD Pin 4 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_4_bpen(&self) -> Pin4BpenR {
        Pin4BpenR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - LCD Pin 5 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_5_bpen(&self) -> Pin5BpenR {
        Pin5BpenR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - LCD Pin 6 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_6_bpen(&self) -> Pin6BpenR {
        Pin6BpenR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - LCD Pin 7 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_7_bpen(&self) -> Pin7BpenR {
        Pin7BpenR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - LCD Pin 8 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_8_bpen(&self) -> Pin8BpenR {
        Pin8BpenR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - LCD Pin 9 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_9_bpen(&self) -> Pin9BpenR {
        Pin9BpenR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - LCD Pin 10 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_10_bpen(&self) -> Pin10BpenR {
        Pin10BpenR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - LCD Pin 11 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_11_bpen(&self) -> Pin11BpenR {
        Pin11BpenR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - LCD Pin 12 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_12_bpen(&self) -> Pin12BpenR {
        Pin12BpenR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - LCD Pin 13 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_13_bpen(&self) -> Pin13BpenR {
        Pin13BpenR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - LCD Pin 14 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_14_bpen(&self) -> Pin14BpenR {
        Pin14BpenR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - LCD Pin 15 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_15_bpen(&self) -> Pin15BpenR {
        Pin15BpenR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - LCD Pin 16 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_16_bpen(&self) -> Pin16BpenR {
        Pin16BpenR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - LCD Pin 17 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_17_bpen(&self) -> Pin17BpenR {
        Pin17BpenR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - LCD Pin 18 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_18_bpen(&self) -> Pin18BpenR {
        Pin18BpenR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - LCD Pin 19 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_19_bpen(&self) -> Pin19BpenR {
        Pin19BpenR::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - LCD Pin 20 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_20_bpen(&self) -> Pin20BpenR {
        Pin20BpenR::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - LCD Pin 21 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_21_bpen(&self) -> Pin21BpenR {
        Pin21BpenR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - LCD Pin 22 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_22_bpen(&self) -> Pin22BpenR {
        Pin22BpenR::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - LCD Pin 23 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_23_bpen(&self) -> Pin23BpenR {
        Pin23BpenR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - LCD Pin 24 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_24_bpen(&self) -> Pin24BpenR {
        Pin24BpenR::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - LCD Pin 25 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_25_bpen(&self) -> Pin25BpenR {
        Pin25BpenR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - LCD Pin 26 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_26_bpen(&self) -> Pin26BpenR {
        Pin26BpenR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - LCD Pin 27 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_27_bpen(&self) -> Pin27BpenR {
        Pin27BpenR::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - LCD Pin 28 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_28_bpen(&self) -> Pin28BpenR {
        Pin28BpenR::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - LCD Pin 29 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_29_bpen(&self) -> Pin29BpenR {
        Pin29BpenR::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - LCD Pin 30 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_30_bpen(&self) -> Pin30BpenR {
        Pin30BpenR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - LCD Pin 31 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_31_bpen(&self) -> Pin31BpenR {
        Pin31BpenR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - LCD Pin 0 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_0_bpen(&mut self) -> Pin0BpenW<LcdBpen0Spec> {
        Pin0BpenW::new(self, 0)
    }
    #[doc = "Bit 1 - LCD Pin 1 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_1_bpen(&mut self) -> Pin1BpenW<LcdBpen0Spec> {
        Pin1BpenW::new(self, 1)
    }
    #[doc = "Bit 2 - LCD Pin 2 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_2_bpen(&mut self) -> Pin2BpenW<LcdBpen0Spec> {
        Pin2BpenW::new(self, 2)
    }
    #[doc = "Bit 3 - LCD Pin 3 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_3_bpen(&mut self) -> Pin3BpenW<LcdBpen0Spec> {
        Pin3BpenW::new(self, 3)
    }
    #[doc = "Bit 4 - LCD Pin 4 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_4_bpen(&mut self) -> Pin4BpenW<LcdBpen0Spec> {
        Pin4BpenW::new(self, 4)
    }
    #[doc = "Bit 5 - LCD Pin 5 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_5_bpen(&mut self) -> Pin5BpenW<LcdBpen0Spec> {
        Pin5BpenW::new(self, 5)
    }
    #[doc = "Bit 6 - LCD Pin 6 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_6_bpen(&mut self) -> Pin6BpenW<LcdBpen0Spec> {
        Pin6BpenW::new(self, 6)
    }
    #[doc = "Bit 7 - LCD Pin 7 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_7_bpen(&mut self) -> Pin7BpenW<LcdBpen0Spec> {
        Pin7BpenW::new(self, 7)
    }
    #[doc = "Bit 8 - LCD Pin 8 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_8_bpen(&mut self) -> Pin8BpenW<LcdBpen0Spec> {
        Pin8BpenW::new(self, 8)
    }
    #[doc = "Bit 9 - LCD Pin 9 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_9_bpen(&mut self) -> Pin9BpenW<LcdBpen0Spec> {
        Pin9BpenW::new(self, 9)
    }
    #[doc = "Bit 10 - LCD Pin 10 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_10_bpen(&mut self) -> Pin10BpenW<LcdBpen0Spec> {
        Pin10BpenW::new(self, 10)
    }
    #[doc = "Bit 11 - LCD Pin 11 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_11_bpen(&mut self) -> Pin11BpenW<LcdBpen0Spec> {
        Pin11BpenW::new(self, 11)
    }
    #[doc = "Bit 12 - LCD Pin 12 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_12_bpen(&mut self) -> Pin12BpenW<LcdBpen0Spec> {
        Pin12BpenW::new(self, 12)
    }
    #[doc = "Bit 13 - LCD Pin 13 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_13_bpen(&mut self) -> Pin13BpenW<LcdBpen0Spec> {
        Pin13BpenW::new(self, 13)
    }
    #[doc = "Bit 14 - LCD Pin 14 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_14_bpen(&mut self) -> Pin14BpenW<LcdBpen0Spec> {
        Pin14BpenW::new(self, 14)
    }
    #[doc = "Bit 15 - LCD Pin 15 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_15_bpen(&mut self) -> Pin15BpenW<LcdBpen0Spec> {
        Pin15BpenW::new(self, 15)
    }
    #[doc = "Bit 16 - LCD Pin 16 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_16_bpen(&mut self) -> Pin16BpenW<LcdBpen0Spec> {
        Pin16BpenW::new(self, 16)
    }
    #[doc = "Bit 17 - LCD Pin 17 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_17_bpen(&mut self) -> Pin17BpenW<LcdBpen0Spec> {
        Pin17BpenW::new(self, 17)
    }
    #[doc = "Bit 18 - LCD Pin 18 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_18_bpen(&mut self) -> Pin18BpenW<LcdBpen0Spec> {
        Pin18BpenW::new(self, 18)
    }
    #[doc = "Bit 19 - LCD Pin 19 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_19_bpen(&mut self) -> Pin19BpenW<LcdBpen0Spec> {
        Pin19BpenW::new(self, 19)
    }
    #[doc = "Bit 20 - LCD Pin 20 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_20_bpen(&mut self) -> Pin20BpenW<LcdBpen0Spec> {
        Pin20BpenW::new(self, 20)
    }
    #[doc = "Bit 21 - LCD Pin 21 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_21_bpen(&mut self) -> Pin21BpenW<LcdBpen0Spec> {
        Pin21BpenW::new(self, 21)
    }
    #[doc = "Bit 22 - LCD Pin 22 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_22_bpen(&mut self) -> Pin22BpenW<LcdBpen0Spec> {
        Pin22BpenW::new(self, 22)
    }
    #[doc = "Bit 23 - LCD Pin 23 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_23_bpen(&mut self) -> Pin23BpenW<LcdBpen0Spec> {
        Pin23BpenW::new(self, 23)
    }
    #[doc = "Bit 24 - LCD Pin 24 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_24_bpen(&mut self) -> Pin24BpenW<LcdBpen0Spec> {
        Pin24BpenW::new(self, 24)
    }
    #[doc = "Bit 25 - LCD Pin 25 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_25_bpen(&mut self) -> Pin25BpenW<LcdBpen0Spec> {
        Pin25BpenW::new(self, 25)
    }
    #[doc = "Bit 26 - LCD Pin 26 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_26_bpen(&mut self) -> Pin26BpenW<LcdBpen0Spec> {
        Pin26BpenW::new(self, 26)
    }
    #[doc = "Bit 27 - LCD Pin 27 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_27_bpen(&mut self) -> Pin27BpenW<LcdBpen0Spec> {
        Pin27BpenW::new(self, 27)
    }
    #[doc = "Bit 28 - LCD Pin 28 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_28_bpen(&mut self) -> Pin28BpenW<LcdBpen0Spec> {
        Pin28BpenW::new(self, 28)
    }
    #[doc = "Bit 29 - LCD Pin 29 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_29_bpen(&mut self) -> Pin29BpenW<LcdBpen0Spec> {
        Pin29BpenW::new(self, 29)
    }
    #[doc = "Bit 30 - LCD Pin 30 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_30_bpen(&mut self) -> Pin30BpenW<LcdBpen0Spec> {
        Pin30BpenW::new(self, 30)
    }
    #[doc = "Bit 31 - LCD Pin 31 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_31_bpen(&mut self) -> Pin31BpenW<LcdBpen0Spec> {
        Pin31BpenW::new(self, 31)
    }
}
#[doc = "LCD Back Plane Enable Register 0\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_bpen0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_bpen0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LcdBpen0Spec;
impl crate::RegisterSpec for LcdBpen0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`lcd_bpen0::R`](R) reader structure"]
impl crate::Readable for LcdBpen0Spec {}
#[doc = "`write(|w| ..)` method takes [`lcd_bpen0::W`](W) writer structure"]
impl crate::Writable for LcdBpen0Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LCD_BPEN0 to value 0"]
impl crate::Resettable for LcdBpen0Spec {}

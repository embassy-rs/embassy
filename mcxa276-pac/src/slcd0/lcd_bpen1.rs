#[doc = "Register `LCD_BPEN1` reader"]
pub type R = crate::R<LcdBpen1Spec>;
#[doc = "Register `LCD_BPEN1` writer"]
pub type W = crate::W<LcdBpen1Spec>;
#[doc = "LCD Pin 32 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin32Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin32Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin32Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_32_BPEN` reader - LCD Pin 32 Back Plane Enable"]
pub type Pin32BpenR = crate::BitReader<Pin32Bpen>;
impl Pin32BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin32Bpen {
        match self.bits {
            false => Pin32Bpen::Fp,
            true => Pin32Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin32Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin32Bpen::Bp
    }
}
#[doc = "Field `PIN_32_BPEN` writer - LCD Pin 32 Back Plane Enable"]
pub type Pin32BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin32Bpen>;
impl<'a, REG> Pin32BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin32Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin32Bpen::Bp)
    }
}
#[doc = "LCD Pin 33 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin33Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin33Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin33Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_33_BPEN` reader - LCD Pin 33 Back Plane Enable"]
pub type Pin33BpenR = crate::BitReader<Pin33Bpen>;
impl Pin33BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin33Bpen {
        match self.bits {
            false => Pin33Bpen::Fp,
            true => Pin33Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin33Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin33Bpen::Bp
    }
}
#[doc = "Field `PIN_33_BPEN` writer - LCD Pin 33 Back Plane Enable"]
pub type Pin33BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin33Bpen>;
impl<'a, REG> Pin33BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin33Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin33Bpen::Bp)
    }
}
#[doc = "LCD Pin 34 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin34Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin34Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin34Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_34_BPEN` reader - LCD Pin 34 Back Plane Enable"]
pub type Pin34BpenR = crate::BitReader<Pin34Bpen>;
impl Pin34BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin34Bpen {
        match self.bits {
            false => Pin34Bpen::Fp,
            true => Pin34Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin34Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin34Bpen::Bp
    }
}
#[doc = "Field `PIN_34_BPEN` writer - LCD Pin 34 Back Plane Enable"]
pub type Pin34BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin34Bpen>;
impl<'a, REG> Pin34BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin34Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin34Bpen::Bp)
    }
}
#[doc = "LCD Pin 35 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin35Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin35Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin35Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_35_BPEN` reader - LCD Pin 35 Back Plane Enable"]
pub type Pin35BpenR = crate::BitReader<Pin35Bpen>;
impl Pin35BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin35Bpen {
        match self.bits {
            false => Pin35Bpen::Fp,
            true => Pin35Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin35Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin35Bpen::Bp
    }
}
#[doc = "Field `PIN_35_BPEN` writer - LCD Pin 35 Back Plane Enable"]
pub type Pin35BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin35Bpen>;
impl<'a, REG> Pin35BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin35Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin35Bpen::Bp)
    }
}
#[doc = "LCD Pin 36 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin36Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin36Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin36Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_36_BPEN` reader - LCD Pin 36 Back Plane Enable"]
pub type Pin36BpenR = crate::BitReader<Pin36Bpen>;
impl Pin36BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin36Bpen {
        match self.bits {
            false => Pin36Bpen::Fp,
            true => Pin36Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin36Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin36Bpen::Bp
    }
}
#[doc = "Field `PIN_36_BPEN` writer - LCD Pin 36 Back Plane Enable"]
pub type Pin36BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin36Bpen>;
impl<'a, REG> Pin36BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin36Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin36Bpen::Bp)
    }
}
#[doc = "LCD Pin 37 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin37Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin37Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin37Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_37_BPEN` reader - LCD Pin 37 Back Plane Enable"]
pub type Pin37BpenR = crate::BitReader<Pin37Bpen>;
impl Pin37BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin37Bpen {
        match self.bits {
            false => Pin37Bpen::Fp,
            true => Pin37Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin37Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin37Bpen::Bp
    }
}
#[doc = "Field `PIN_37_BPEN` writer - LCD Pin 37 Back Plane Enable"]
pub type Pin37BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin37Bpen>;
impl<'a, REG> Pin37BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin37Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin37Bpen::Bp)
    }
}
#[doc = "LCD Pin 38 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin38Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin38Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin38Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_38_BPEN` reader - LCD Pin 38 Back Plane Enable"]
pub type Pin38BpenR = crate::BitReader<Pin38Bpen>;
impl Pin38BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin38Bpen {
        match self.bits {
            false => Pin38Bpen::Fp,
            true => Pin38Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin38Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin38Bpen::Bp
    }
}
#[doc = "Field `PIN_38_BPEN` writer - LCD Pin 38 Back Plane Enable"]
pub type Pin38BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin38Bpen>;
impl<'a, REG> Pin38BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin38Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin38Bpen::Bp)
    }
}
#[doc = "LCD Pin 39 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin39Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin39Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin39Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_39_BPEN` reader - LCD Pin 39 Back Plane Enable"]
pub type Pin39BpenR = crate::BitReader<Pin39Bpen>;
impl Pin39BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin39Bpen {
        match self.bits {
            false => Pin39Bpen::Fp,
            true => Pin39Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin39Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin39Bpen::Bp
    }
}
#[doc = "Field `PIN_39_BPEN` writer - LCD Pin 39 Back Plane Enable"]
pub type Pin39BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin39Bpen>;
impl<'a, REG> Pin39BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin39Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin39Bpen::Bp)
    }
}
#[doc = "LCD Pin 40 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin40Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin40Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin40Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_40_BPEN` reader - LCD Pin 40 Back Plane Enable"]
pub type Pin40BpenR = crate::BitReader<Pin40Bpen>;
impl Pin40BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin40Bpen {
        match self.bits {
            false => Pin40Bpen::Fp,
            true => Pin40Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin40Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin40Bpen::Bp
    }
}
#[doc = "Field `PIN_40_BPEN` writer - LCD Pin 40 Back Plane Enable"]
pub type Pin40BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin40Bpen>;
impl<'a, REG> Pin40BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin40Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin40Bpen::Bp)
    }
}
#[doc = "LCD Pin 41 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin41Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin41Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin41Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_41_BPEN` reader - LCD Pin 41 Back Plane Enable"]
pub type Pin41BpenR = crate::BitReader<Pin41Bpen>;
impl Pin41BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin41Bpen {
        match self.bits {
            false => Pin41Bpen::Fp,
            true => Pin41Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin41Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin41Bpen::Bp
    }
}
#[doc = "Field `PIN_41_BPEN` writer - LCD Pin 41 Back Plane Enable"]
pub type Pin41BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin41Bpen>;
impl<'a, REG> Pin41BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin41Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin41Bpen::Bp)
    }
}
#[doc = "LCD Pin 42 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin42Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin42Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin42Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_42_BPEN` reader - LCD Pin 42 Back Plane Enable"]
pub type Pin42BpenR = crate::BitReader<Pin42Bpen>;
impl Pin42BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin42Bpen {
        match self.bits {
            false => Pin42Bpen::Fp,
            true => Pin42Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin42Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin42Bpen::Bp
    }
}
#[doc = "Field `PIN_42_BPEN` writer - LCD Pin 42 Back Plane Enable"]
pub type Pin42BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin42Bpen>;
impl<'a, REG> Pin42BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin42Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin42Bpen::Bp)
    }
}
#[doc = "LCD Pin 43 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin43Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin43Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin43Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_43_BPEN` reader - LCD Pin 43 Back Plane Enable"]
pub type Pin43BpenR = crate::BitReader<Pin43Bpen>;
impl Pin43BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin43Bpen {
        match self.bits {
            false => Pin43Bpen::Fp,
            true => Pin43Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin43Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin43Bpen::Bp
    }
}
#[doc = "Field `PIN_43_BPEN` writer - LCD Pin 43 Back Plane Enable"]
pub type Pin43BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin43Bpen>;
impl<'a, REG> Pin43BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin43Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin43Bpen::Bp)
    }
}
#[doc = "LCD Pin 44 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin44Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin44Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin44Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_44_BPEN` reader - LCD Pin 44 Back Plane Enable"]
pub type Pin44BpenR = crate::BitReader<Pin44Bpen>;
impl Pin44BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin44Bpen {
        match self.bits {
            false => Pin44Bpen::Fp,
            true => Pin44Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin44Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin44Bpen::Bp
    }
}
#[doc = "Field `PIN_44_BPEN` writer - LCD Pin 44 Back Plane Enable"]
pub type Pin44BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin44Bpen>;
impl<'a, REG> Pin44BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin44Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin44Bpen::Bp)
    }
}
#[doc = "LCD Pin 45 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin45Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin45Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin45Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_45_BPEN` reader - LCD Pin 45 Back Plane Enable"]
pub type Pin45BpenR = crate::BitReader<Pin45Bpen>;
impl Pin45BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin45Bpen {
        match self.bits {
            false => Pin45Bpen::Fp,
            true => Pin45Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin45Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin45Bpen::Bp
    }
}
#[doc = "Field `PIN_45_BPEN` writer - LCD Pin 45 Back Plane Enable"]
pub type Pin45BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin45Bpen>;
impl<'a, REG> Pin45BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin45Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin45Bpen::Bp)
    }
}
#[doc = "LCD Pin 46 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin46Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin46Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin46Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_46_BPEN` reader - LCD Pin 46 Back Plane Enable"]
pub type Pin46BpenR = crate::BitReader<Pin46Bpen>;
impl Pin46BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin46Bpen {
        match self.bits {
            false => Pin46Bpen::Fp,
            true => Pin46Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin46Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin46Bpen::Bp
    }
}
#[doc = "Field `PIN_46_BPEN` writer - LCD Pin 46 Back Plane Enable"]
pub type Pin46BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin46Bpen>;
impl<'a, REG> Pin46BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin46Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin46Bpen::Bp)
    }
}
#[doc = "LCD Pin 47 Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin47Bpen {
    #[doc = "0: Pin as front plane."]
    Fp = 0,
    #[doc = "1: Pin as back plane."]
    Bp = 1,
}
impl From<Pin47Bpen> for bool {
    #[inline(always)]
    fn from(variant: Pin47Bpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PIN_47_BPEN` reader - LCD Pin 47 Back Plane Enable"]
pub type Pin47BpenR = crate::BitReader<Pin47Bpen>;
impl Pin47BpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pin47Bpen {
        match self.bits {
            false => Pin47Bpen::Fp,
            true => Pin47Bpen::Bp,
        }
    }
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn is_fp(&self) -> bool {
        *self == Pin47Bpen::Fp
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn is_bp(&self) -> bool {
        *self == Pin47Bpen::Bp
    }
}
#[doc = "Field `PIN_47_BPEN` writer - LCD Pin 47 Back Plane Enable"]
pub type Pin47BpenW<'a, REG> = crate::BitWriter<'a, REG, Pin47Bpen>;
impl<'a, REG> Pin47BpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pin as front plane."]
    #[inline(always)]
    pub fn fp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin47Bpen::Fp)
    }
    #[doc = "Pin as back plane."]
    #[inline(always)]
    pub fn bp(self) -> &'a mut crate::W<REG> {
        self.variant(Pin47Bpen::Bp)
    }
}
impl R {
    #[doc = "Bit 0 - LCD Pin 32 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_32_bpen(&self) -> Pin32BpenR {
        Pin32BpenR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - LCD Pin 33 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_33_bpen(&self) -> Pin33BpenR {
        Pin33BpenR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - LCD Pin 34 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_34_bpen(&self) -> Pin34BpenR {
        Pin34BpenR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - LCD Pin 35 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_35_bpen(&self) -> Pin35BpenR {
        Pin35BpenR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - LCD Pin 36 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_36_bpen(&self) -> Pin36BpenR {
        Pin36BpenR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - LCD Pin 37 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_37_bpen(&self) -> Pin37BpenR {
        Pin37BpenR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - LCD Pin 38 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_38_bpen(&self) -> Pin38BpenR {
        Pin38BpenR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - LCD Pin 39 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_39_bpen(&self) -> Pin39BpenR {
        Pin39BpenR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - LCD Pin 40 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_40_bpen(&self) -> Pin40BpenR {
        Pin40BpenR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - LCD Pin 41 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_41_bpen(&self) -> Pin41BpenR {
        Pin41BpenR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - LCD Pin 42 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_42_bpen(&self) -> Pin42BpenR {
        Pin42BpenR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - LCD Pin 43 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_43_bpen(&self) -> Pin43BpenR {
        Pin43BpenR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - LCD Pin 44 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_44_bpen(&self) -> Pin44BpenR {
        Pin44BpenR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - LCD Pin 45 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_45_bpen(&self) -> Pin45BpenR {
        Pin45BpenR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - LCD Pin 46 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_46_bpen(&self) -> Pin46BpenR {
        Pin46BpenR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - LCD Pin 47 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_47_bpen(&self) -> Pin47BpenR {
        Pin47BpenR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - LCD Pin 32 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_32_bpen(&mut self) -> Pin32BpenW<LcdBpen1Spec> {
        Pin32BpenW::new(self, 0)
    }
    #[doc = "Bit 1 - LCD Pin 33 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_33_bpen(&mut self) -> Pin33BpenW<LcdBpen1Spec> {
        Pin33BpenW::new(self, 1)
    }
    #[doc = "Bit 2 - LCD Pin 34 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_34_bpen(&mut self) -> Pin34BpenW<LcdBpen1Spec> {
        Pin34BpenW::new(self, 2)
    }
    #[doc = "Bit 3 - LCD Pin 35 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_35_bpen(&mut self) -> Pin35BpenW<LcdBpen1Spec> {
        Pin35BpenW::new(self, 3)
    }
    #[doc = "Bit 4 - LCD Pin 36 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_36_bpen(&mut self) -> Pin36BpenW<LcdBpen1Spec> {
        Pin36BpenW::new(self, 4)
    }
    #[doc = "Bit 5 - LCD Pin 37 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_37_bpen(&mut self) -> Pin37BpenW<LcdBpen1Spec> {
        Pin37BpenW::new(self, 5)
    }
    #[doc = "Bit 6 - LCD Pin 38 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_38_bpen(&mut self) -> Pin38BpenW<LcdBpen1Spec> {
        Pin38BpenW::new(self, 6)
    }
    #[doc = "Bit 7 - LCD Pin 39 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_39_bpen(&mut self) -> Pin39BpenW<LcdBpen1Spec> {
        Pin39BpenW::new(self, 7)
    }
    #[doc = "Bit 8 - LCD Pin 40 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_40_bpen(&mut self) -> Pin40BpenW<LcdBpen1Spec> {
        Pin40BpenW::new(self, 8)
    }
    #[doc = "Bit 9 - LCD Pin 41 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_41_bpen(&mut self) -> Pin41BpenW<LcdBpen1Spec> {
        Pin41BpenW::new(self, 9)
    }
    #[doc = "Bit 10 - LCD Pin 42 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_42_bpen(&mut self) -> Pin42BpenW<LcdBpen1Spec> {
        Pin42BpenW::new(self, 10)
    }
    #[doc = "Bit 11 - LCD Pin 43 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_43_bpen(&mut self) -> Pin43BpenW<LcdBpen1Spec> {
        Pin43BpenW::new(self, 11)
    }
    #[doc = "Bit 12 - LCD Pin 44 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_44_bpen(&mut self) -> Pin44BpenW<LcdBpen1Spec> {
        Pin44BpenW::new(self, 12)
    }
    #[doc = "Bit 13 - LCD Pin 45 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_45_bpen(&mut self) -> Pin45BpenW<LcdBpen1Spec> {
        Pin45BpenW::new(self, 13)
    }
    #[doc = "Bit 14 - LCD Pin 46 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_46_bpen(&mut self) -> Pin46BpenW<LcdBpen1Spec> {
        Pin46BpenW::new(self, 14)
    }
    #[doc = "Bit 15 - LCD Pin 47 Back Plane Enable"]
    #[inline(always)]
    pub fn pin_47_bpen(&mut self) -> Pin47BpenW<LcdBpen1Spec> {
        Pin47BpenW::new(self, 15)
    }
}
#[doc = "LCD Back Plane Enable Register 1\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_bpen1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_bpen1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LcdBpen1Spec;
impl crate::RegisterSpec for LcdBpen1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`lcd_bpen1::R`](R) reader structure"]
impl crate::Readable for LcdBpen1Spec {}
#[doc = "`write(|w| ..)` method takes [`lcd_bpen1::W`](W) writer structure"]
impl crate::Writable for LcdBpen1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LCD_BPEN1 to value 0"]
impl crate::Resettable for LcdBpen1Spec {}

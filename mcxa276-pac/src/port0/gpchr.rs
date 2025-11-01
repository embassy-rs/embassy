#[doc = "Register `GPCHR` reader"]
pub type R = crate::R<GpchrSpec>;
#[doc = "Register `GPCHR` writer"]
pub type W = crate::W<GpchrSpec>;
#[doc = "Field `GPWD` reader - Global Pin Write Data"]
pub type GpwdR = crate::FieldReader<u16>;
#[doc = "Field `GPWD` writer - Global Pin Write Data"]
pub type GpwdW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe16 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe16> for bool {
    #[inline(always)]
    fn from(variant: Gpwe16) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE16` reader - Global Pin Write Enable"]
pub type Gpwe16R = crate::BitReader<Gpwe16>;
impl Gpwe16R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe16 {
        match self.bits {
            false => Gpwe16::Gpwe0,
            true => Gpwe16::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe16::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe16::Gpwe1
    }
}
#[doc = "Field `GPWE16` writer - Global Pin Write Enable"]
pub type Gpwe16W<'a, REG> = crate::BitWriter<'a, REG, Gpwe16>;
impl<'a, REG> Gpwe16W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe16::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe16::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe17 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe17> for bool {
    #[inline(always)]
    fn from(variant: Gpwe17) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE17` reader - Global Pin Write Enable"]
pub type Gpwe17R = crate::BitReader<Gpwe17>;
impl Gpwe17R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe17 {
        match self.bits {
            false => Gpwe17::Gpwe0,
            true => Gpwe17::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe17::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe17::Gpwe1
    }
}
#[doc = "Field `GPWE17` writer - Global Pin Write Enable"]
pub type Gpwe17W<'a, REG> = crate::BitWriter<'a, REG, Gpwe17>;
impl<'a, REG> Gpwe17W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe17::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe17::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe18 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe18> for bool {
    #[inline(always)]
    fn from(variant: Gpwe18) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE18` reader - Global Pin Write Enable"]
pub type Gpwe18R = crate::BitReader<Gpwe18>;
impl Gpwe18R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe18 {
        match self.bits {
            false => Gpwe18::Gpwe0,
            true => Gpwe18::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe18::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe18::Gpwe1
    }
}
#[doc = "Field `GPWE18` writer - Global Pin Write Enable"]
pub type Gpwe18W<'a, REG> = crate::BitWriter<'a, REG, Gpwe18>;
impl<'a, REG> Gpwe18W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe18::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe18::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe19 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe19> for bool {
    #[inline(always)]
    fn from(variant: Gpwe19) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE19` reader - Global Pin Write Enable"]
pub type Gpwe19R = crate::BitReader<Gpwe19>;
impl Gpwe19R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe19 {
        match self.bits {
            false => Gpwe19::Gpwe0,
            true => Gpwe19::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe19::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe19::Gpwe1
    }
}
#[doc = "Field `GPWE19` writer - Global Pin Write Enable"]
pub type Gpwe19W<'a, REG> = crate::BitWriter<'a, REG, Gpwe19>;
impl<'a, REG> Gpwe19W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe19::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe19::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe20 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe20> for bool {
    #[inline(always)]
    fn from(variant: Gpwe20) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE20` reader - Global Pin Write Enable"]
pub type Gpwe20R = crate::BitReader<Gpwe20>;
impl Gpwe20R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe20 {
        match self.bits {
            false => Gpwe20::Gpwe0,
            true => Gpwe20::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe20::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe20::Gpwe1
    }
}
#[doc = "Field `GPWE20` writer - Global Pin Write Enable"]
pub type Gpwe20W<'a, REG> = crate::BitWriter<'a, REG, Gpwe20>;
impl<'a, REG> Gpwe20W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe20::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe20::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe21 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe21> for bool {
    #[inline(always)]
    fn from(variant: Gpwe21) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE21` reader - Global Pin Write Enable"]
pub type Gpwe21R = crate::BitReader<Gpwe21>;
impl Gpwe21R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe21 {
        match self.bits {
            false => Gpwe21::Gpwe0,
            true => Gpwe21::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe21::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe21::Gpwe1
    }
}
#[doc = "Field `GPWE21` writer - Global Pin Write Enable"]
pub type Gpwe21W<'a, REG> = crate::BitWriter<'a, REG, Gpwe21>;
impl<'a, REG> Gpwe21W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe21::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe21::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe22 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe22> for bool {
    #[inline(always)]
    fn from(variant: Gpwe22) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE22` reader - Global Pin Write Enable"]
pub type Gpwe22R = crate::BitReader<Gpwe22>;
impl Gpwe22R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe22 {
        match self.bits {
            false => Gpwe22::Gpwe0,
            true => Gpwe22::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe22::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe22::Gpwe1
    }
}
#[doc = "Field `GPWE22` writer - Global Pin Write Enable"]
pub type Gpwe22W<'a, REG> = crate::BitWriter<'a, REG, Gpwe22>;
impl<'a, REG> Gpwe22W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe22::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe22::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe23 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe23> for bool {
    #[inline(always)]
    fn from(variant: Gpwe23) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE23` reader - Global Pin Write Enable"]
pub type Gpwe23R = crate::BitReader<Gpwe23>;
impl Gpwe23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe23 {
        match self.bits {
            false => Gpwe23::Gpwe0,
            true => Gpwe23::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe23::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe23::Gpwe1
    }
}
#[doc = "Field `GPWE23` writer - Global Pin Write Enable"]
pub type Gpwe23W<'a, REG> = crate::BitWriter<'a, REG, Gpwe23>;
impl<'a, REG> Gpwe23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe23::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe23::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe24 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe24> for bool {
    #[inline(always)]
    fn from(variant: Gpwe24) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE24` reader - Global Pin Write Enable"]
pub type Gpwe24R = crate::BitReader<Gpwe24>;
impl Gpwe24R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe24 {
        match self.bits {
            false => Gpwe24::Gpwe0,
            true => Gpwe24::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe24::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe24::Gpwe1
    }
}
#[doc = "Field `GPWE24` writer - Global Pin Write Enable"]
pub type Gpwe24W<'a, REG> = crate::BitWriter<'a, REG, Gpwe24>;
impl<'a, REG> Gpwe24W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe24::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe24::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe25 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe25> for bool {
    #[inline(always)]
    fn from(variant: Gpwe25) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE25` reader - Global Pin Write Enable"]
pub type Gpwe25R = crate::BitReader<Gpwe25>;
impl Gpwe25R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe25 {
        match self.bits {
            false => Gpwe25::Gpwe0,
            true => Gpwe25::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe25::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe25::Gpwe1
    }
}
#[doc = "Field `GPWE25` writer - Global Pin Write Enable"]
pub type Gpwe25W<'a, REG> = crate::BitWriter<'a, REG, Gpwe25>;
impl<'a, REG> Gpwe25W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe25::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe25::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe26 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe26> for bool {
    #[inline(always)]
    fn from(variant: Gpwe26) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE26` reader - Global Pin Write Enable"]
pub type Gpwe26R = crate::BitReader<Gpwe26>;
impl Gpwe26R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe26 {
        match self.bits {
            false => Gpwe26::Gpwe0,
            true => Gpwe26::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe26::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe26::Gpwe1
    }
}
#[doc = "Field `GPWE26` writer - Global Pin Write Enable"]
pub type Gpwe26W<'a, REG> = crate::BitWriter<'a, REG, Gpwe26>;
impl<'a, REG> Gpwe26W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe26::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe26::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe27 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe27> for bool {
    #[inline(always)]
    fn from(variant: Gpwe27) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE27` reader - Global Pin Write Enable"]
pub type Gpwe27R = crate::BitReader<Gpwe27>;
impl Gpwe27R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe27 {
        match self.bits {
            false => Gpwe27::Gpwe0,
            true => Gpwe27::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe27::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe27::Gpwe1
    }
}
#[doc = "Field `GPWE27` writer - Global Pin Write Enable"]
pub type Gpwe27W<'a, REG> = crate::BitWriter<'a, REG, Gpwe27>;
impl<'a, REG> Gpwe27W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe27::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe27::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe28 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe28> for bool {
    #[inline(always)]
    fn from(variant: Gpwe28) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE28` reader - Global Pin Write Enable"]
pub type Gpwe28R = crate::BitReader<Gpwe28>;
impl Gpwe28R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe28 {
        match self.bits {
            false => Gpwe28::Gpwe0,
            true => Gpwe28::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe28::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe28::Gpwe1
    }
}
#[doc = "Field `GPWE28` writer - Global Pin Write Enable"]
pub type Gpwe28W<'a, REG> = crate::BitWriter<'a, REG, Gpwe28>;
impl<'a, REG> Gpwe28W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe28::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe28::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe29 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe29> for bool {
    #[inline(always)]
    fn from(variant: Gpwe29) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE29` reader - Global Pin Write Enable"]
pub type Gpwe29R = crate::BitReader<Gpwe29>;
impl Gpwe29R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe29 {
        match self.bits {
            false => Gpwe29::Gpwe0,
            true => Gpwe29::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe29::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe29::Gpwe1
    }
}
#[doc = "Field `GPWE29` writer - Global Pin Write Enable"]
pub type Gpwe29W<'a, REG> = crate::BitWriter<'a, REG, Gpwe29>;
impl<'a, REG> Gpwe29W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe29::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe29::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe30 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe30> for bool {
    #[inline(always)]
    fn from(variant: Gpwe30) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE30` reader - Global Pin Write Enable"]
pub type Gpwe30R = crate::BitReader<Gpwe30>;
impl Gpwe30R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe30 {
        match self.bits {
            false => Gpwe30::Gpwe0,
            true => Gpwe30::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe30::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe30::Gpwe1
    }
}
#[doc = "Field `GPWE30` writer - Global Pin Write Enable"]
pub type Gpwe30W<'a, REG> = crate::BitWriter<'a, REG, Gpwe30>;
impl<'a, REG> Gpwe30W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe30::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe30::Gpwe1)
    }
}
#[doc = "Global Pin Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpwe31 {
    #[doc = "0: Not updated"]
    Gpwe0 = 0,
    #[doc = "1: Updated"]
    Gpwe1 = 1,
}
impl From<Gpwe31> for bool {
    #[inline(always)]
    fn from(variant: Gpwe31) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GPWE31` reader - Global Pin Write Enable"]
pub type Gpwe31R = crate::BitReader<Gpwe31>;
impl Gpwe31R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Gpwe31 {
        match self.bits {
            false => Gpwe31::Gpwe0,
            true => Gpwe31::Gpwe1,
        }
    }
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn is_gpwe0(&self) -> bool {
        *self == Gpwe31::Gpwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_gpwe1(&self) -> bool {
        *self == Gpwe31::Gpwe1
    }
}
#[doc = "Field `GPWE31` writer - Global Pin Write Enable"]
pub type Gpwe31W<'a, REG> = crate::BitWriter<'a, REG, Gpwe31>;
impl<'a, REG> Gpwe31W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated"]
    #[inline(always)]
    pub fn gpwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe31::Gpwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn gpwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Gpwe31::Gpwe1)
    }
}
impl R {
    #[doc = "Bits 0:15 - Global Pin Write Data"]
    #[inline(always)]
    pub fn gpwd(&self) -> GpwdR {
        GpwdR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bit 16 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe16(&self) -> Gpwe16R {
        Gpwe16R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe17(&self) -> Gpwe17R {
        Gpwe17R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe18(&self) -> Gpwe18R {
        Gpwe18R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe19(&self) -> Gpwe19R {
        Gpwe19R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe20(&self) -> Gpwe20R {
        Gpwe20R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe21(&self) -> Gpwe21R {
        Gpwe21R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe22(&self) -> Gpwe22R {
        Gpwe22R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe23(&self) -> Gpwe23R {
        Gpwe23R::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bit 24 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe24(&self) -> Gpwe24R {
        Gpwe24R::new(((self.bits >> 24) & 1) != 0)
    }
    #[doc = "Bit 25 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe25(&self) -> Gpwe25R {
        Gpwe25R::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe26(&self) -> Gpwe26R {
        Gpwe26R::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe27(&self) -> Gpwe27R {
        Gpwe27R::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bit 28 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe28(&self) -> Gpwe28R {
        Gpwe28R::new(((self.bits >> 28) & 1) != 0)
    }
    #[doc = "Bit 29 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe29(&self) -> Gpwe29R {
        Gpwe29R::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe30(&self) -> Gpwe30R {
        Gpwe30R::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe31(&self) -> Gpwe31R {
        Gpwe31R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:15 - Global Pin Write Data"]
    #[inline(always)]
    pub fn gpwd(&mut self) -> GpwdW<GpchrSpec> {
        GpwdW::new(self, 0)
    }
    #[doc = "Bit 16 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe16(&mut self) -> Gpwe16W<GpchrSpec> {
        Gpwe16W::new(self, 16)
    }
    #[doc = "Bit 17 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe17(&mut self) -> Gpwe17W<GpchrSpec> {
        Gpwe17W::new(self, 17)
    }
    #[doc = "Bit 18 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe18(&mut self) -> Gpwe18W<GpchrSpec> {
        Gpwe18W::new(self, 18)
    }
    #[doc = "Bit 19 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe19(&mut self) -> Gpwe19W<GpchrSpec> {
        Gpwe19W::new(self, 19)
    }
    #[doc = "Bit 20 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe20(&mut self) -> Gpwe20W<GpchrSpec> {
        Gpwe20W::new(self, 20)
    }
    #[doc = "Bit 21 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe21(&mut self) -> Gpwe21W<GpchrSpec> {
        Gpwe21W::new(self, 21)
    }
    #[doc = "Bit 22 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe22(&mut self) -> Gpwe22W<GpchrSpec> {
        Gpwe22W::new(self, 22)
    }
    #[doc = "Bit 23 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe23(&mut self) -> Gpwe23W<GpchrSpec> {
        Gpwe23W::new(self, 23)
    }
    #[doc = "Bit 24 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe24(&mut self) -> Gpwe24W<GpchrSpec> {
        Gpwe24W::new(self, 24)
    }
    #[doc = "Bit 25 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe25(&mut self) -> Gpwe25W<GpchrSpec> {
        Gpwe25W::new(self, 25)
    }
    #[doc = "Bit 26 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe26(&mut self) -> Gpwe26W<GpchrSpec> {
        Gpwe26W::new(self, 26)
    }
    #[doc = "Bit 27 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe27(&mut self) -> Gpwe27W<GpchrSpec> {
        Gpwe27W::new(self, 27)
    }
    #[doc = "Bit 28 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe28(&mut self) -> Gpwe28W<GpchrSpec> {
        Gpwe28W::new(self, 28)
    }
    #[doc = "Bit 29 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe29(&mut self) -> Gpwe29W<GpchrSpec> {
        Gpwe29W::new(self, 29)
    }
    #[doc = "Bit 30 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe30(&mut self) -> Gpwe30W<GpchrSpec> {
        Gpwe30W::new(self, 30)
    }
    #[doc = "Bit 31 - Global Pin Write Enable"]
    #[inline(always)]
    pub fn gpwe31(&mut self) -> Gpwe31W<GpchrSpec> {
        Gpwe31W::new(self, 31)
    }
}
#[doc = "Global Pin Control High\n\nYou can [`read`](crate::Reg::read) this register and get [`gpchr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gpchr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct GpchrSpec;
impl crate::RegisterSpec for GpchrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`gpchr::R`](R) reader structure"]
impl crate::Readable for GpchrSpec {}
#[doc = "`write(|w| ..)` method takes [`gpchr::W`](W) writer structure"]
impl crate::Writable for GpchrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets GPCHR to value 0"]
impl crate::Resettable for GpchrSpec {}

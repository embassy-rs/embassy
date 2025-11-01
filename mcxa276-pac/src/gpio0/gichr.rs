#[doc = "Register `GICHR` reader"]
pub type R = crate::R<GichrSpec>;
#[doc = "Register `GICHR` writer"]
pub type W = crate::W<GichrSpec>;
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe16 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe16> for bool {
    #[inline(always)]
    fn from(variant: Giwe16) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE16` reader - Global Interrupt Write Enable"]
pub type Giwe16R = crate::BitReader<Giwe16>;
impl Giwe16R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe16 {
        match self.bits {
            false => Giwe16::Giwe0,
            true => Giwe16::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe16::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe16::Giwe1
    }
}
#[doc = "Field `GIWE16` writer - Global Interrupt Write Enable"]
pub type Giwe16W<'a, REG> = crate::BitWriter<'a, REG, Giwe16>;
impl<'a, REG> Giwe16W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe16::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe16::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe17 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe17> for bool {
    #[inline(always)]
    fn from(variant: Giwe17) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE17` reader - Global Interrupt Write Enable"]
pub type Giwe17R = crate::BitReader<Giwe17>;
impl Giwe17R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe17 {
        match self.bits {
            false => Giwe17::Giwe0,
            true => Giwe17::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe17::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe17::Giwe1
    }
}
#[doc = "Field `GIWE17` writer - Global Interrupt Write Enable"]
pub type Giwe17W<'a, REG> = crate::BitWriter<'a, REG, Giwe17>;
impl<'a, REG> Giwe17W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe17::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe17::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe18 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe18> for bool {
    #[inline(always)]
    fn from(variant: Giwe18) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE18` reader - Global Interrupt Write Enable"]
pub type Giwe18R = crate::BitReader<Giwe18>;
impl Giwe18R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe18 {
        match self.bits {
            false => Giwe18::Giwe0,
            true => Giwe18::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe18::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe18::Giwe1
    }
}
#[doc = "Field `GIWE18` writer - Global Interrupt Write Enable"]
pub type Giwe18W<'a, REG> = crate::BitWriter<'a, REG, Giwe18>;
impl<'a, REG> Giwe18W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe18::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe18::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe19 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe19> for bool {
    #[inline(always)]
    fn from(variant: Giwe19) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE19` reader - Global Interrupt Write Enable"]
pub type Giwe19R = crate::BitReader<Giwe19>;
impl Giwe19R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe19 {
        match self.bits {
            false => Giwe19::Giwe0,
            true => Giwe19::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe19::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe19::Giwe1
    }
}
#[doc = "Field `GIWE19` writer - Global Interrupt Write Enable"]
pub type Giwe19W<'a, REG> = crate::BitWriter<'a, REG, Giwe19>;
impl<'a, REG> Giwe19W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe19::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe19::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe20 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe20> for bool {
    #[inline(always)]
    fn from(variant: Giwe20) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE20` reader - Global Interrupt Write Enable"]
pub type Giwe20R = crate::BitReader<Giwe20>;
impl Giwe20R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe20 {
        match self.bits {
            false => Giwe20::Giwe0,
            true => Giwe20::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe20::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe20::Giwe1
    }
}
#[doc = "Field `GIWE20` writer - Global Interrupt Write Enable"]
pub type Giwe20W<'a, REG> = crate::BitWriter<'a, REG, Giwe20>;
impl<'a, REG> Giwe20W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe20::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe20::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe21 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe21> for bool {
    #[inline(always)]
    fn from(variant: Giwe21) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE21` reader - Global Interrupt Write Enable"]
pub type Giwe21R = crate::BitReader<Giwe21>;
impl Giwe21R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe21 {
        match self.bits {
            false => Giwe21::Giwe0,
            true => Giwe21::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe21::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe21::Giwe1
    }
}
#[doc = "Field `GIWE21` writer - Global Interrupt Write Enable"]
pub type Giwe21W<'a, REG> = crate::BitWriter<'a, REG, Giwe21>;
impl<'a, REG> Giwe21W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe21::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe21::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe22 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe22> for bool {
    #[inline(always)]
    fn from(variant: Giwe22) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE22` reader - Global Interrupt Write Enable"]
pub type Giwe22R = crate::BitReader<Giwe22>;
impl Giwe22R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe22 {
        match self.bits {
            false => Giwe22::Giwe0,
            true => Giwe22::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe22::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe22::Giwe1
    }
}
#[doc = "Field `GIWE22` writer - Global Interrupt Write Enable"]
pub type Giwe22W<'a, REG> = crate::BitWriter<'a, REG, Giwe22>;
impl<'a, REG> Giwe22W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe22::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe22::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe23 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe23> for bool {
    #[inline(always)]
    fn from(variant: Giwe23) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE23` reader - Global Interrupt Write Enable"]
pub type Giwe23R = crate::BitReader<Giwe23>;
impl Giwe23R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe23 {
        match self.bits {
            false => Giwe23::Giwe0,
            true => Giwe23::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe23::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe23::Giwe1
    }
}
#[doc = "Field `GIWE23` writer - Global Interrupt Write Enable"]
pub type Giwe23W<'a, REG> = crate::BitWriter<'a, REG, Giwe23>;
impl<'a, REG> Giwe23W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe23::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe23::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe24 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe24> for bool {
    #[inline(always)]
    fn from(variant: Giwe24) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE24` reader - Global Interrupt Write Enable"]
pub type Giwe24R = crate::BitReader<Giwe24>;
impl Giwe24R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe24 {
        match self.bits {
            false => Giwe24::Giwe0,
            true => Giwe24::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe24::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe24::Giwe1
    }
}
#[doc = "Field `GIWE24` writer - Global Interrupt Write Enable"]
pub type Giwe24W<'a, REG> = crate::BitWriter<'a, REG, Giwe24>;
impl<'a, REG> Giwe24W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe24::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe24::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe25 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe25> for bool {
    #[inline(always)]
    fn from(variant: Giwe25) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE25` reader - Global Interrupt Write Enable"]
pub type Giwe25R = crate::BitReader<Giwe25>;
impl Giwe25R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe25 {
        match self.bits {
            false => Giwe25::Giwe0,
            true => Giwe25::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe25::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe25::Giwe1
    }
}
#[doc = "Field `GIWE25` writer - Global Interrupt Write Enable"]
pub type Giwe25W<'a, REG> = crate::BitWriter<'a, REG, Giwe25>;
impl<'a, REG> Giwe25W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe25::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe25::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe26 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe26> for bool {
    #[inline(always)]
    fn from(variant: Giwe26) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE26` reader - Global Interrupt Write Enable"]
pub type Giwe26R = crate::BitReader<Giwe26>;
impl Giwe26R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe26 {
        match self.bits {
            false => Giwe26::Giwe0,
            true => Giwe26::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe26::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe26::Giwe1
    }
}
#[doc = "Field `GIWE26` writer - Global Interrupt Write Enable"]
pub type Giwe26W<'a, REG> = crate::BitWriter<'a, REG, Giwe26>;
impl<'a, REG> Giwe26W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe26::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe26::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe27 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe27> for bool {
    #[inline(always)]
    fn from(variant: Giwe27) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE27` reader - Global Interrupt Write Enable"]
pub type Giwe27R = crate::BitReader<Giwe27>;
impl Giwe27R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe27 {
        match self.bits {
            false => Giwe27::Giwe0,
            true => Giwe27::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe27::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe27::Giwe1
    }
}
#[doc = "Field `GIWE27` writer - Global Interrupt Write Enable"]
pub type Giwe27W<'a, REG> = crate::BitWriter<'a, REG, Giwe27>;
impl<'a, REG> Giwe27W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe27::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe27::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe28 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe28> for bool {
    #[inline(always)]
    fn from(variant: Giwe28) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE28` reader - Global Interrupt Write Enable"]
pub type Giwe28R = crate::BitReader<Giwe28>;
impl Giwe28R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe28 {
        match self.bits {
            false => Giwe28::Giwe0,
            true => Giwe28::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe28::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe28::Giwe1
    }
}
#[doc = "Field `GIWE28` writer - Global Interrupt Write Enable"]
pub type Giwe28W<'a, REG> = crate::BitWriter<'a, REG, Giwe28>;
impl<'a, REG> Giwe28W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe28::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe28::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe29 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe29> for bool {
    #[inline(always)]
    fn from(variant: Giwe29) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE29` reader - Global Interrupt Write Enable"]
pub type Giwe29R = crate::BitReader<Giwe29>;
impl Giwe29R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe29 {
        match self.bits {
            false => Giwe29::Giwe0,
            true => Giwe29::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe29::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe29::Giwe1
    }
}
#[doc = "Field `GIWE29` writer - Global Interrupt Write Enable"]
pub type Giwe29W<'a, REG> = crate::BitWriter<'a, REG, Giwe29>;
impl<'a, REG> Giwe29W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe29::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe29::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe30 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe30> for bool {
    #[inline(always)]
    fn from(variant: Giwe30) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE30` reader - Global Interrupt Write Enable"]
pub type Giwe30R = crate::BitReader<Giwe30>;
impl Giwe30R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe30 {
        match self.bits {
            false => Giwe30::Giwe0,
            true => Giwe30::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe30::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe30::Giwe1
    }
}
#[doc = "Field `GIWE30` writer - Global Interrupt Write Enable"]
pub type Giwe30W<'a, REG> = crate::BitWriter<'a, REG, Giwe30>;
impl<'a, REG> Giwe30W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe30::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe30::Giwe1)
    }
}
#[doc = "Global Interrupt Write Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Giwe31 {
    #[doc = "0: Not updated."]
    Giwe0 = 0,
    #[doc = "1: Updated"]
    Giwe1 = 1,
}
impl From<Giwe31> for bool {
    #[inline(always)]
    fn from(variant: Giwe31) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `GIWE31` reader - Global Interrupt Write Enable"]
pub type Giwe31R = crate::BitReader<Giwe31>;
impl Giwe31R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Giwe31 {
        match self.bits {
            false => Giwe31::Giwe0,
            true => Giwe31::Giwe1,
        }
    }
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn is_giwe0(&self) -> bool {
        *self == Giwe31::Giwe0
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn is_giwe1(&self) -> bool {
        *self == Giwe31::Giwe1
    }
}
#[doc = "Field `GIWE31` writer - Global Interrupt Write Enable"]
pub type Giwe31W<'a, REG> = crate::BitWriter<'a, REG, Giwe31>;
impl<'a, REG> Giwe31W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not updated."]
    #[inline(always)]
    pub fn giwe0(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe31::Giwe0)
    }
    #[doc = "Updated"]
    #[inline(always)]
    pub fn giwe1(self) -> &'a mut crate::W<REG> {
        self.variant(Giwe31::Giwe1)
    }
}
#[doc = "Field `GIWD` reader - Global Interrupt Write Data"]
pub type GiwdR = crate::FieldReader<u16>;
#[doc = "Field `GIWD` writer - Global Interrupt Write Data"]
pub type GiwdW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bit 0 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe16(&self) -> Giwe16R {
        Giwe16R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe17(&self) -> Giwe17R {
        Giwe17R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe18(&self) -> Giwe18R {
        Giwe18R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe19(&self) -> Giwe19R {
        Giwe19R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe20(&self) -> Giwe20R {
        Giwe20R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe21(&self) -> Giwe21R {
        Giwe21R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe22(&self) -> Giwe22R {
        Giwe22R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe23(&self) -> Giwe23R {
        Giwe23R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe24(&self) -> Giwe24R {
        Giwe24R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe25(&self) -> Giwe25R {
        Giwe25R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe26(&self) -> Giwe26R {
        Giwe26R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe27(&self) -> Giwe27R {
        Giwe27R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe28(&self) -> Giwe28R {
        Giwe28R::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe29(&self) -> Giwe29R {
        Giwe29R::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe30(&self) -> Giwe30R {
        Giwe30R::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe31(&self) -> Giwe31R {
        Giwe31R::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bits 16:31 - Global Interrupt Write Data"]
    #[inline(always)]
    pub fn giwd(&self) -> GiwdR {
        GiwdR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bit 0 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe16(&mut self) -> Giwe16W<GichrSpec> {
        Giwe16W::new(self, 0)
    }
    #[doc = "Bit 1 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe17(&mut self) -> Giwe17W<GichrSpec> {
        Giwe17W::new(self, 1)
    }
    #[doc = "Bit 2 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe18(&mut self) -> Giwe18W<GichrSpec> {
        Giwe18W::new(self, 2)
    }
    #[doc = "Bit 3 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe19(&mut self) -> Giwe19W<GichrSpec> {
        Giwe19W::new(self, 3)
    }
    #[doc = "Bit 4 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe20(&mut self) -> Giwe20W<GichrSpec> {
        Giwe20W::new(self, 4)
    }
    #[doc = "Bit 5 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe21(&mut self) -> Giwe21W<GichrSpec> {
        Giwe21W::new(self, 5)
    }
    #[doc = "Bit 6 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe22(&mut self) -> Giwe22W<GichrSpec> {
        Giwe22W::new(self, 6)
    }
    #[doc = "Bit 7 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe23(&mut self) -> Giwe23W<GichrSpec> {
        Giwe23W::new(self, 7)
    }
    #[doc = "Bit 8 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe24(&mut self) -> Giwe24W<GichrSpec> {
        Giwe24W::new(self, 8)
    }
    #[doc = "Bit 9 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe25(&mut self) -> Giwe25W<GichrSpec> {
        Giwe25W::new(self, 9)
    }
    #[doc = "Bit 10 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe26(&mut self) -> Giwe26W<GichrSpec> {
        Giwe26W::new(self, 10)
    }
    #[doc = "Bit 11 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe27(&mut self) -> Giwe27W<GichrSpec> {
        Giwe27W::new(self, 11)
    }
    #[doc = "Bit 12 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe28(&mut self) -> Giwe28W<GichrSpec> {
        Giwe28W::new(self, 12)
    }
    #[doc = "Bit 13 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe29(&mut self) -> Giwe29W<GichrSpec> {
        Giwe29W::new(self, 13)
    }
    #[doc = "Bit 14 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe30(&mut self) -> Giwe30W<GichrSpec> {
        Giwe30W::new(self, 14)
    }
    #[doc = "Bit 15 - Global Interrupt Write Enable"]
    #[inline(always)]
    pub fn giwe31(&mut self) -> Giwe31W<GichrSpec> {
        Giwe31W::new(self, 15)
    }
    #[doc = "Bits 16:31 - Global Interrupt Write Data"]
    #[inline(always)]
    pub fn giwd(&mut self) -> GiwdW<GichrSpec> {
        GiwdW::new(self, 16)
    }
}
#[doc = "Global Interrupt Control High\n\nYou can [`read`](crate::Reg::read) this register and get [`gichr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gichr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct GichrSpec;
impl crate::RegisterSpec for GichrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`gichr::R`](R) reader structure"]
impl crate::Readable for GichrSpec {}
#[doc = "`write(|w| ..)` method takes [`gichr::W`](W) writer structure"]
impl crate::Writable for GichrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets GICHR to value 0"]
impl crate::Resettable for GichrSpec {}

#[doc = "Register `PPR` reader"]
pub type R = crate::R<PprSpec>;
#[doc = "Register `PPR` writer"]
pub type W = crate::W<PprSpec>;
#[doc = "Tamper Pin n Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpp0 {
    #[doc = "0: Not inverted"]
    NoInvert = 0,
    #[doc = "1: Inverted"]
    Invert = 1,
}
impl From<Tpp0> for bool {
    #[inline(always)]
    fn from(variant: Tpp0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPP0` reader - Tamper Pin n Polarity"]
pub type Tpp0R = crate::BitReader<Tpp0>;
impl Tpp0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpp0 {
        match self.bits {
            false => Tpp0::NoInvert,
            true => Tpp0::Invert,
        }
    }
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn is_no_invert(&self) -> bool {
        *self == Tpp0::NoInvert
    }
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn is_invert(&self) -> bool {
        *self == Tpp0::Invert
    }
}
#[doc = "Field `TPP0` writer - Tamper Pin n Polarity"]
pub type Tpp0W<'a, REG> = crate::BitWriter<'a, REG, Tpp0>;
impl<'a, REG> Tpp0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn no_invert(self) -> &'a mut crate::W<REG> {
        self.variant(Tpp0::NoInvert)
    }
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn invert(self) -> &'a mut crate::W<REG> {
        self.variant(Tpp0::Invert)
    }
}
#[doc = "Tamper Pin n Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpp1 {
    #[doc = "0: Not inverted"]
    NoInvert = 0,
    #[doc = "1: Inverted"]
    Invert = 1,
}
impl From<Tpp1> for bool {
    #[inline(always)]
    fn from(variant: Tpp1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPP1` reader - Tamper Pin n Polarity"]
pub type Tpp1R = crate::BitReader<Tpp1>;
impl Tpp1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpp1 {
        match self.bits {
            false => Tpp1::NoInvert,
            true => Tpp1::Invert,
        }
    }
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn is_no_invert(&self) -> bool {
        *self == Tpp1::NoInvert
    }
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn is_invert(&self) -> bool {
        *self == Tpp1::Invert
    }
}
#[doc = "Field `TPP1` writer - Tamper Pin n Polarity"]
pub type Tpp1W<'a, REG> = crate::BitWriter<'a, REG, Tpp1>;
impl<'a, REG> Tpp1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn no_invert(self) -> &'a mut crate::W<REG> {
        self.variant(Tpp1::NoInvert)
    }
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn invert(self) -> &'a mut crate::W<REG> {
        self.variant(Tpp1::Invert)
    }
}
#[doc = "Tamper Pin n Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpp2 {
    #[doc = "0: Not inverted"]
    NoInvert = 0,
    #[doc = "1: Inverted"]
    Invert = 1,
}
impl From<Tpp2> for bool {
    #[inline(always)]
    fn from(variant: Tpp2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPP2` reader - Tamper Pin n Polarity"]
pub type Tpp2R = crate::BitReader<Tpp2>;
impl Tpp2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpp2 {
        match self.bits {
            false => Tpp2::NoInvert,
            true => Tpp2::Invert,
        }
    }
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn is_no_invert(&self) -> bool {
        *self == Tpp2::NoInvert
    }
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn is_invert(&self) -> bool {
        *self == Tpp2::Invert
    }
}
#[doc = "Field `TPP2` writer - Tamper Pin n Polarity"]
pub type Tpp2W<'a, REG> = crate::BitWriter<'a, REG, Tpp2>;
impl<'a, REG> Tpp2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn no_invert(self) -> &'a mut crate::W<REG> {
        self.variant(Tpp2::NoInvert)
    }
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn invert(self) -> &'a mut crate::W<REG> {
        self.variant(Tpp2::Invert)
    }
}
#[doc = "Tamper Pin n Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpp3 {
    #[doc = "0: Not inverted"]
    NoInvert = 0,
    #[doc = "1: Inverted"]
    Invert = 1,
}
impl From<Tpp3> for bool {
    #[inline(always)]
    fn from(variant: Tpp3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPP3` reader - Tamper Pin n Polarity"]
pub type Tpp3R = crate::BitReader<Tpp3>;
impl Tpp3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpp3 {
        match self.bits {
            false => Tpp3::NoInvert,
            true => Tpp3::Invert,
        }
    }
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn is_no_invert(&self) -> bool {
        *self == Tpp3::NoInvert
    }
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn is_invert(&self) -> bool {
        *self == Tpp3::Invert
    }
}
#[doc = "Field `TPP3` writer - Tamper Pin n Polarity"]
pub type Tpp3W<'a, REG> = crate::BitWriter<'a, REG, Tpp3>;
impl<'a, REG> Tpp3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn no_invert(self) -> &'a mut crate::W<REG> {
        self.variant(Tpp3::NoInvert)
    }
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn invert(self) -> &'a mut crate::W<REG> {
        self.variant(Tpp3::Invert)
    }
}
#[doc = "Tamper Pin n Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpp4 {
    #[doc = "0: Not inverted"]
    NoInvert = 0,
    #[doc = "1: Inverted"]
    Invert = 1,
}
impl From<Tpp4> for bool {
    #[inline(always)]
    fn from(variant: Tpp4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPP4` reader - Tamper Pin n Polarity"]
pub type Tpp4R = crate::BitReader<Tpp4>;
impl Tpp4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpp4 {
        match self.bits {
            false => Tpp4::NoInvert,
            true => Tpp4::Invert,
        }
    }
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn is_no_invert(&self) -> bool {
        *self == Tpp4::NoInvert
    }
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn is_invert(&self) -> bool {
        *self == Tpp4::Invert
    }
}
#[doc = "Field `TPP4` writer - Tamper Pin n Polarity"]
pub type Tpp4W<'a, REG> = crate::BitWriter<'a, REG, Tpp4>;
impl<'a, REG> Tpp4W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn no_invert(self) -> &'a mut crate::W<REG> {
        self.variant(Tpp4::NoInvert)
    }
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn invert(self) -> &'a mut crate::W<REG> {
        self.variant(Tpp4::Invert)
    }
}
#[doc = "Tamper Pin n Polarity\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpp5 {
    #[doc = "0: Not inverted"]
    NoInvert = 0,
    #[doc = "1: Inverted"]
    Invert = 1,
}
impl From<Tpp5> for bool {
    #[inline(always)]
    fn from(variant: Tpp5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPP5` reader - Tamper Pin n Polarity"]
pub type Tpp5R = crate::BitReader<Tpp5>;
impl Tpp5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpp5 {
        match self.bits {
            false => Tpp5::NoInvert,
            true => Tpp5::Invert,
        }
    }
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn is_no_invert(&self) -> bool {
        *self == Tpp5::NoInvert
    }
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn is_invert(&self) -> bool {
        *self == Tpp5::Invert
    }
}
#[doc = "Field `TPP5` writer - Tamper Pin n Polarity"]
pub type Tpp5W<'a, REG> = crate::BitWriter<'a, REG, Tpp5>;
impl<'a, REG> Tpp5W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not inverted"]
    #[inline(always)]
    pub fn no_invert(self) -> &'a mut crate::W<REG> {
        self.variant(Tpp5::NoInvert)
    }
    #[doc = "Inverted"]
    #[inline(always)]
    pub fn invert(self) -> &'a mut crate::W<REG> {
        self.variant(Tpp5::Invert)
    }
}
#[doc = "Tamper Pin n Input Data\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpid0 {
    #[doc = "0: Zero"]
    Zero = 0,
    #[doc = "1: One"]
    One = 1,
}
impl From<Tpid0> for bool {
    #[inline(always)]
    fn from(variant: Tpid0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPID0` reader - Tamper Pin n Input Data"]
pub type Tpid0R = crate::BitReader<Tpid0>;
impl Tpid0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpid0 {
        match self.bits {
            false => Tpid0::Zero,
            true => Tpid0::One,
        }
    }
    #[doc = "Zero"]
    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        *self == Tpid0::Zero
    }
    #[doc = "One"]
    #[inline(always)]
    pub fn is_one(&self) -> bool {
        *self == Tpid0::One
    }
}
#[doc = "Tamper Pin n Input Data\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpid1 {
    #[doc = "0: Zero"]
    Zero = 0,
    #[doc = "1: One"]
    One = 1,
}
impl From<Tpid1> for bool {
    #[inline(always)]
    fn from(variant: Tpid1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPID1` reader - Tamper Pin n Input Data"]
pub type Tpid1R = crate::BitReader<Tpid1>;
impl Tpid1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpid1 {
        match self.bits {
            false => Tpid1::Zero,
            true => Tpid1::One,
        }
    }
    #[doc = "Zero"]
    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        *self == Tpid1::Zero
    }
    #[doc = "One"]
    #[inline(always)]
    pub fn is_one(&self) -> bool {
        *self == Tpid1::One
    }
}
#[doc = "Tamper Pin n Input Data\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpid2 {
    #[doc = "0: Zero"]
    Zero = 0,
    #[doc = "1: One"]
    One = 1,
}
impl From<Tpid2> for bool {
    #[inline(always)]
    fn from(variant: Tpid2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPID2` reader - Tamper Pin n Input Data"]
pub type Tpid2R = crate::BitReader<Tpid2>;
impl Tpid2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpid2 {
        match self.bits {
            false => Tpid2::Zero,
            true => Tpid2::One,
        }
    }
    #[doc = "Zero"]
    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        *self == Tpid2::Zero
    }
    #[doc = "One"]
    #[inline(always)]
    pub fn is_one(&self) -> bool {
        *self == Tpid2::One
    }
}
#[doc = "Tamper Pin n Input Data\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpid3 {
    #[doc = "0: Zero"]
    Zero = 0,
    #[doc = "1: One"]
    One = 1,
}
impl From<Tpid3> for bool {
    #[inline(always)]
    fn from(variant: Tpid3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPID3` reader - Tamper Pin n Input Data"]
pub type Tpid3R = crate::BitReader<Tpid3>;
impl Tpid3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpid3 {
        match self.bits {
            false => Tpid3::Zero,
            true => Tpid3::One,
        }
    }
    #[doc = "Zero"]
    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        *self == Tpid3::Zero
    }
    #[doc = "One"]
    #[inline(always)]
    pub fn is_one(&self) -> bool {
        *self == Tpid3::One
    }
}
#[doc = "Tamper Pin n Input Data\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpid4 {
    #[doc = "0: Zero"]
    Zero = 0,
    #[doc = "1: One"]
    One = 1,
}
impl From<Tpid4> for bool {
    #[inline(always)]
    fn from(variant: Tpid4) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPID4` reader - Tamper Pin n Input Data"]
pub type Tpid4R = crate::BitReader<Tpid4>;
impl Tpid4R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpid4 {
        match self.bits {
            false => Tpid4::Zero,
            true => Tpid4::One,
        }
    }
    #[doc = "Zero"]
    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        *self == Tpid4::Zero
    }
    #[doc = "One"]
    #[inline(always)]
    pub fn is_one(&self) -> bool {
        *self == Tpid4::One
    }
}
#[doc = "Tamper Pin n Input Data\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tpid5 {
    #[doc = "0: Zero"]
    Zero = 0,
    #[doc = "1: One"]
    One = 1,
}
impl From<Tpid5> for bool {
    #[inline(always)]
    fn from(variant: Tpid5) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TPID5` reader - Tamper Pin n Input Data"]
pub type Tpid5R = crate::BitReader<Tpid5>;
impl Tpid5R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tpid5 {
        match self.bits {
            false => Tpid5::Zero,
            true => Tpid5::One,
        }
    }
    #[doc = "Zero"]
    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        *self == Tpid5::Zero
    }
    #[doc = "One"]
    #[inline(always)]
    pub fn is_one(&self) -> bool {
        *self == Tpid5::One
    }
}
impl R {
    #[doc = "Bit 0 - Tamper Pin n Polarity"]
    #[inline(always)]
    pub fn tpp0(&self) -> Tpp0R {
        Tpp0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Tamper Pin n Polarity"]
    #[inline(always)]
    pub fn tpp1(&self) -> Tpp1R {
        Tpp1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Tamper Pin n Polarity"]
    #[inline(always)]
    pub fn tpp2(&self) -> Tpp2R {
        Tpp2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Tamper Pin n Polarity"]
    #[inline(always)]
    pub fn tpp3(&self) -> Tpp3R {
        Tpp3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Tamper Pin n Polarity"]
    #[inline(always)]
    pub fn tpp4(&self) -> Tpp4R {
        Tpp4R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Tamper Pin n Polarity"]
    #[inline(always)]
    pub fn tpp5(&self) -> Tpp5R {
        Tpp5R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 16 - Tamper Pin n Input Data"]
    #[inline(always)]
    pub fn tpid0(&self) -> Tpid0R {
        Tpid0R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Tamper Pin n Input Data"]
    #[inline(always)]
    pub fn tpid1(&self) -> Tpid1R {
        Tpid1R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Tamper Pin n Input Data"]
    #[inline(always)]
    pub fn tpid2(&self) -> Tpid2R {
        Tpid2R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Tamper Pin n Input Data"]
    #[inline(always)]
    pub fn tpid3(&self) -> Tpid3R {
        Tpid3R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Tamper Pin n Input Data"]
    #[inline(always)]
    pub fn tpid4(&self) -> Tpid4R {
        Tpid4R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Tamper Pin n Input Data"]
    #[inline(always)]
    pub fn tpid5(&self) -> Tpid5R {
        Tpid5R::new(((self.bits >> 21) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Tamper Pin n Polarity"]
    #[inline(always)]
    pub fn tpp0(&mut self) -> Tpp0W<PprSpec> {
        Tpp0W::new(self, 0)
    }
    #[doc = "Bit 1 - Tamper Pin n Polarity"]
    #[inline(always)]
    pub fn tpp1(&mut self) -> Tpp1W<PprSpec> {
        Tpp1W::new(self, 1)
    }
    #[doc = "Bit 2 - Tamper Pin n Polarity"]
    #[inline(always)]
    pub fn tpp2(&mut self) -> Tpp2W<PprSpec> {
        Tpp2W::new(self, 2)
    }
    #[doc = "Bit 3 - Tamper Pin n Polarity"]
    #[inline(always)]
    pub fn tpp3(&mut self) -> Tpp3W<PprSpec> {
        Tpp3W::new(self, 3)
    }
    #[doc = "Bit 4 - Tamper Pin n Polarity"]
    #[inline(always)]
    pub fn tpp4(&mut self) -> Tpp4W<PprSpec> {
        Tpp4W::new(self, 4)
    }
    #[doc = "Bit 5 - Tamper Pin n Polarity"]
    #[inline(always)]
    pub fn tpp5(&mut self) -> Tpp5W<PprSpec> {
        Tpp5W::new(self, 5)
    }
}
#[doc = "Pin Polarity\n\nYou can [`read`](crate::Reg::read) this register and get [`ppr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ppr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PprSpec;
impl crate::RegisterSpec for PprSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ppr::R`](R) reader structure"]
impl crate::Readable for PprSpec {}
#[doc = "`write(|w| ..)` method takes [`ppr::W`](W) writer structure"]
impl crate::Writable for PprSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PPR to value 0"]
impl crate::Resettable for PprSpec {}

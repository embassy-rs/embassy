#[doc = "Register `CFG` reader"]
pub type R = crate::R<CfgSpec>;
#[doc = "Register `CFG` writer"]
pub type W = crate::W<CfgSpec>;
#[doc = "Enable Capture 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capen0 {
    #[doc = "0: Disable"]
    Capen0isdisabled = 0,
    #[doc = "1: Enable"]
    Capen0isenabled = 1,
}
impl From<Capen0> for bool {
    #[inline(always)]
    fn from(variant: Capen0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAPEN0` reader - Enable Capture 0"]
pub type Capen0R = crate::BitReader<Capen0>;
impl Capen0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Capen0 {
        match self.bits {
            false => Capen0::Capen0isdisabled,
            true => Capen0::Capen0isenabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_capen0isdisabled(&self) -> bool {
        *self == Capen0::Capen0isdisabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_capen0isenabled(&self) -> bool {
        *self == Capen0::Capen0isenabled
    }
}
#[doc = "Field `CAPEN0` writer - Enable Capture 0"]
pub type Capen0W<'a, REG> = crate::BitWriter<'a, REG, Capen0>;
impl<'a, REG> Capen0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn capen0isdisabled(self) -> &'a mut crate::W<REG> {
        self.variant(Capen0::Capen0isdisabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn capen0isenabled(self) -> &'a mut crate::W<REG> {
        self.variant(Capen0::Capen0isenabled)
    }
}
#[doc = "Enable Capture 1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capen1 {
    #[doc = "0: Disable"]
    Capen1isdisabled = 0,
    #[doc = "1: Enable"]
    Capen1isenabled = 1,
}
impl From<Capen1> for bool {
    #[inline(always)]
    fn from(variant: Capen1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAPEN1` reader - Enable Capture 1"]
pub type Capen1R = crate::BitReader<Capen1>;
impl Capen1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Capen1 {
        match self.bits {
            false => Capen1::Capen1isdisabled,
            true => Capen1::Capen1isenabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_capen1isdisabled(&self) -> bool {
        *self == Capen1::Capen1isdisabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_capen1isenabled(&self) -> bool {
        *self == Capen1::Capen1isenabled
    }
}
#[doc = "Field `CAPEN1` writer - Enable Capture 1"]
pub type Capen1W<'a, REG> = crate::BitWriter<'a, REG, Capen1>;
impl<'a, REG> Capen1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn capen1isdisabled(self) -> &'a mut crate::W<REG> {
        self.variant(Capen1::Capen1isdisabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn capen1isenabled(self) -> &'a mut crate::W<REG> {
        self.variant(Capen1::Capen1isenabled)
    }
}
#[doc = "Enable Capture 2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capen2 {
    #[doc = "0: Disable"]
    Capen2isdisabled = 0,
    #[doc = "1: Enable"]
    Capen2isenabled = 1,
}
impl From<Capen2> for bool {
    #[inline(always)]
    fn from(variant: Capen2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAPEN2` reader - Enable Capture 2"]
pub type Capen2R = crate::BitReader<Capen2>;
impl Capen2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Capen2 {
        match self.bits {
            false => Capen2::Capen2isdisabled,
            true => Capen2::Capen2isenabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_capen2isdisabled(&self) -> bool {
        *self == Capen2::Capen2isdisabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_capen2isenabled(&self) -> bool {
        *self == Capen2::Capen2isenabled
    }
}
#[doc = "Field `CAPEN2` writer - Enable Capture 2"]
pub type Capen2W<'a, REG> = crate::BitWriter<'a, REG, Capen2>;
impl<'a, REG> Capen2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn capen2isdisabled(self) -> &'a mut crate::W<REG> {
        self.variant(Capen2::Capen2isdisabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn capen2isenabled(self) -> &'a mut crate::W<REG> {
        self.variant(Capen2::Capen2isenabled)
    }
}
#[doc = "Enable Capture 3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capen3 {
    #[doc = "0: Disable"]
    Capen3isdisabled = 0,
    #[doc = "1: Enable"]
    Capen3isenabled = 1,
}
impl From<Capen3> for bool {
    #[inline(always)]
    fn from(variant: Capen3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAPEN3` reader - Enable Capture 3"]
pub type Capen3R = crate::BitReader<Capen3>;
impl Capen3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Capen3 {
        match self.bits {
            false => Capen3::Capen3isdisabled,
            true => Capen3::Capen3isenabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_capen3isdisabled(&self) -> bool {
        *self == Capen3::Capen3isdisabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_capen3isenabled(&self) -> bool {
        *self == Capen3::Capen3isenabled
    }
}
#[doc = "Field `CAPEN3` writer - Enable Capture 3"]
pub type Capen3W<'a, REG> = crate::BitWriter<'a, REG, Capen3>;
impl<'a, REG> Capen3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn capen3isdisabled(self) -> &'a mut crate::W<REG> {
        self.variant(Capen3::Capen3isdisabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn capen3isenabled(self) -> &'a mut crate::W<REG> {
        self.variant(Capen3::Capen3isenabled)
    }
}
#[doc = "Capture Polarity 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cappol0 {
    #[doc = "0: Positive"]
    Cappol0posedgecapture = 0,
    #[doc = "1: Negative"]
    Cappol0negedgecapture = 1,
}
impl From<Cappol0> for bool {
    #[inline(always)]
    fn from(variant: Cappol0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAPPOL0` reader - Capture Polarity 0"]
pub type Cappol0R = crate::BitReader<Cappol0>;
impl Cappol0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cappol0 {
        match self.bits {
            false => Cappol0::Cappol0posedgecapture,
            true => Cappol0::Cappol0negedgecapture,
        }
    }
    #[doc = "Positive"]
    #[inline(always)]
    pub fn is_cappol0posedgecapture(&self) -> bool {
        *self == Cappol0::Cappol0posedgecapture
    }
    #[doc = "Negative"]
    #[inline(always)]
    pub fn is_cappol0negedgecapture(&self) -> bool {
        *self == Cappol0::Cappol0negedgecapture
    }
}
#[doc = "Field `CAPPOL0` writer - Capture Polarity 0"]
pub type Cappol0W<'a, REG> = crate::BitWriter<'a, REG, Cappol0>;
impl<'a, REG> Cappol0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Positive"]
    #[inline(always)]
    pub fn cappol0posedgecapture(self) -> &'a mut crate::W<REG> {
        self.variant(Cappol0::Cappol0posedgecapture)
    }
    #[doc = "Negative"]
    #[inline(always)]
    pub fn cappol0negedgecapture(self) -> &'a mut crate::W<REG> {
        self.variant(Cappol0::Cappol0negedgecapture)
    }
}
#[doc = "Capture-Polarity 1\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cappol1 {
    #[doc = "0: Positive"]
    Cappol1posedgecapture = 0,
    #[doc = "1: Negative"]
    Cappol1negedgecapture = 1,
}
impl From<Cappol1> for bool {
    #[inline(always)]
    fn from(variant: Cappol1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAPPOL1` reader - Capture-Polarity 1"]
pub type Cappol1R = crate::BitReader<Cappol1>;
impl Cappol1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cappol1 {
        match self.bits {
            false => Cappol1::Cappol1posedgecapture,
            true => Cappol1::Cappol1negedgecapture,
        }
    }
    #[doc = "Positive"]
    #[inline(always)]
    pub fn is_cappol1posedgecapture(&self) -> bool {
        *self == Cappol1::Cappol1posedgecapture
    }
    #[doc = "Negative"]
    #[inline(always)]
    pub fn is_cappol1negedgecapture(&self) -> bool {
        *self == Cappol1::Cappol1negedgecapture
    }
}
#[doc = "Field `CAPPOL1` writer - Capture-Polarity 1"]
pub type Cappol1W<'a, REG> = crate::BitWriter<'a, REG, Cappol1>;
impl<'a, REG> Cappol1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Positive"]
    #[inline(always)]
    pub fn cappol1posedgecapture(self) -> &'a mut crate::W<REG> {
        self.variant(Cappol1::Cappol1posedgecapture)
    }
    #[doc = "Negative"]
    #[inline(always)]
    pub fn cappol1negedgecapture(self) -> &'a mut crate::W<REG> {
        self.variant(Cappol1::Cappol1negedgecapture)
    }
}
#[doc = "Capture Polarity 2\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cappol2 {
    #[doc = "0: Positive"]
    Cappol2posedgecapture = 0,
    #[doc = "1: Negative"]
    Cappol2negedgecapture = 1,
}
impl From<Cappol2> for bool {
    #[inline(always)]
    fn from(variant: Cappol2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAPPOL2` reader - Capture Polarity 2"]
pub type Cappol2R = crate::BitReader<Cappol2>;
impl Cappol2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cappol2 {
        match self.bits {
            false => Cappol2::Cappol2posedgecapture,
            true => Cappol2::Cappol2negedgecapture,
        }
    }
    #[doc = "Positive"]
    #[inline(always)]
    pub fn is_cappol2posedgecapture(&self) -> bool {
        *self == Cappol2::Cappol2posedgecapture
    }
    #[doc = "Negative"]
    #[inline(always)]
    pub fn is_cappol2negedgecapture(&self) -> bool {
        *self == Cappol2::Cappol2negedgecapture
    }
}
#[doc = "Field `CAPPOL2` writer - Capture Polarity 2"]
pub type Cappol2W<'a, REG> = crate::BitWriter<'a, REG, Cappol2>;
impl<'a, REG> Cappol2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Positive"]
    #[inline(always)]
    pub fn cappol2posedgecapture(self) -> &'a mut crate::W<REG> {
        self.variant(Cappol2::Cappol2posedgecapture)
    }
    #[doc = "Negative"]
    #[inline(always)]
    pub fn cappol2negedgecapture(self) -> &'a mut crate::W<REG> {
        self.variant(Cappol2::Cappol2negedgecapture)
    }
}
#[doc = "Capture Polarity 3\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cappol3 {
    #[doc = "0: Positive"]
    Cappol3posedgecapture = 0,
    #[doc = "1: Negative"]
    Cappol3negedgecapture = 1,
}
impl From<Cappol3> for bool {
    #[inline(always)]
    fn from(variant: Cappol3) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CAPPOL3` reader - Capture Polarity 3"]
pub type Cappol3R = crate::BitReader<Cappol3>;
impl Cappol3R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cappol3 {
        match self.bits {
            false => Cappol3::Cappol3posedgecapture,
            true => Cappol3::Cappol3negedgecapture,
        }
    }
    #[doc = "Positive"]
    #[inline(always)]
    pub fn is_cappol3posedgecapture(&self) -> bool {
        *self == Cappol3::Cappol3posedgecapture
    }
    #[doc = "Negative"]
    #[inline(always)]
    pub fn is_cappol3negedgecapture(&self) -> bool {
        *self == Cappol3::Cappol3negedgecapture
    }
}
#[doc = "Field `CAPPOL3` writer - Capture Polarity 3"]
pub type Cappol3W<'a, REG> = crate::BitWriter<'a, REG, Cappol3>;
impl<'a, REG> Cappol3W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Positive"]
    #[inline(always)]
    pub fn cappol3posedgecapture(self) -> &'a mut crate::W<REG> {
        self.variant(Cappol3::Cappol3posedgecapture)
    }
    #[doc = "Negative"]
    #[inline(always)]
    pub fn cappol3negedgecapture(self) -> &'a mut crate::W<REG> {
        self.variant(Cappol3::Cappol3negedgecapture)
    }
}
impl R {
    #[doc = "Bit 0 - Enable Capture 0"]
    #[inline(always)]
    pub fn capen0(&self) -> Capen0R {
        Capen0R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Enable Capture 1"]
    #[inline(always)]
    pub fn capen1(&self) -> Capen1R {
        Capen1R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Enable Capture 2"]
    #[inline(always)]
    pub fn capen2(&self) -> Capen2R {
        Capen2R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Enable Capture 3"]
    #[inline(always)]
    pub fn capen3(&self) -> Capen3R {
        Capen3R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 8 - Capture Polarity 0"]
    #[inline(always)]
    pub fn cappol0(&self) -> Cappol0R {
        Cappol0R::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Capture-Polarity 1"]
    #[inline(always)]
    pub fn cappol1(&self) -> Cappol1R {
        Cappol1R::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Capture Polarity 2"]
    #[inline(always)]
    pub fn cappol2(&self) -> Cappol2R {
        Cappol2R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Capture Polarity 3"]
    #[inline(always)]
    pub fn cappol3(&self) -> Cappol3R {
        Cappol3R::new(((self.bits >> 11) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Enable Capture 0"]
    #[inline(always)]
    pub fn capen0(&mut self) -> Capen0W<CfgSpec> {
        Capen0W::new(self, 0)
    }
    #[doc = "Bit 1 - Enable Capture 1"]
    #[inline(always)]
    pub fn capen1(&mut self) -> Capen1W<CfgSpec> {
        Capen1W::new(self, 1)
    }
    #[doc = "Bit 2 - Enable Capture 2"]
    #[inline(always)]
    pub fn capen2(&mut self) -> Capen2W<CfgSpec> {
        Capen2W::new(self, 2)
    }
    #[doc = "Bit 3 - Enable Capture 3"]
    #[inline(always)]
    pub fn capen3(&mut self) -> Capen3W<CfgSpec> {
        Capen3W::new(self, 3)
    }
    #[doc = "Bit 8 - Capture Polarity 0"]
    #[inline(always)]
    pub fn cappol0(&mut self) -> Cappol0W<CfgSpec> {
        Cappol0W::new(self, 8)
    }
    #[doc = "Bit 9 - Capture-Polarity 1"]
    #[inline(always)]
    pub fn cappol1(&mut self) -> Cappol1W<CfgSpec> {
        Cappol1W::new(self, 9)
    }
    #[doc = "Bit 10 - Capture Polarity 2"]
    #[inline(always)]
    pub fn cappol2(&mut self) -> Cappol2W<CfgSpec> {
        Cappol2W::new(self, 10)
    }
    #[doc = "Bit 11 - Capture Polarity 3"]
    #[inline(always)]
    pub fn cappol3(&mut self) -> Cappol3W<CfgSpec> {
        Cappol3W::new(self, 11)
    }
}
#[doc = "Capture Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`cfg::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`cfg::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CfgSpec;
impl crate::RegisterSpec for CfgSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`cfg::R`](R) reader structure"]
impl crate::Readable for CfgSpec {}
#[doc = "`write(|w| ..)` method takes [`cfg::W`](W) writer structure"]
impl crate::Writable for CfgSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CFG to value 0"]
impl crate::Resettable for CfgSpec {}

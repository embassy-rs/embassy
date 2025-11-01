#[doc = "Register `TST` reader"]
pub type R = crate::R<TstSpec>;
#[doc = "Register `TST` writer"]
pub type W = crate::W<TstSpec>;
#[doc = "Field `TEST_COUNT` reader - TEST_COUNT"]
pub type TestCountR = crate::FieldReader;
#[doc = "Field `TEST_COUNT` writer - TEST_COUNT"]
pub type TestCountW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `TEST_PERIOD` reader - TEST_PERIOD"]
pub type TestPeriodR = crate::FieldReader;
#[doc = "Field `TEST_PERIOD` writer - TEST_PERIOD"]
pub type TestPeriodW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Quadrature Decoder Negative Signal\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Qdn {
    #[doc = "0: Generates a positive quadrature decoder signal"]
    Qdn0 = 0,
    #[doc = "1: Generates a negative quadrature decoder signal"]
    Qdn1 = 1,
}
impl From<Qdn> for bool {
    #[inline(always)]
    fn from(variant: Qdn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `QDN` reader - Quadrature Decoder Negative Signal"]
pub type QdnR = crate::BitReader<Qdn>;
impl QdnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Qdn {
        match self.bits {
            false => Qdn::Qdn0,
            true => Qdn::Qdn1,
        }
    }
    #[doc = "Generates a positive quadrature decoder signal"]
    #[inline(always)]
    pub fn is_qdn0(&self) -> bool {
        *self == Qdn::Qdn0
    }
    #[doc = "Generates a negative quadrature decoder signal"]
    #[inline(always)]
    pub fn is_qdn1(&self) -> bool {
        *self == Qdn::Qdn1
    }
}
#[doc = "Field `QDN` writer - Quadrature Decoder Negative Signal"]
pub type QdnW<'a, REG> = crate::BitWriter<'a, REG, Qdn>;
impl<'a, REG> QdnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Generates a positive quadrature decoder signal"]
    #[inline(always)]
    pub fn qdn0(self) -> &'a mut crate::W<REG> {
        self.variant(Qdn::Qdn0)
    }
    #[doc = "Generates a negative quadrature decoder signal"]
    #[inline(always)]
    pub fn qdn1(self) -> &'a mut crate::W<REG> {
        self.variant(Qdn::Qdn1)
    }
}
#[doc = "Test Counter Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tce {
    #[doc = "0: Disabled"]
    Tce0 = 0,
    #[doc = "1: Enabled"]
    Tce1 = 1,
}
impl From<Tce> for bool {
    #[inline(always)]
    fn from(variant: Tce) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TCE` reader - Test Counter Enable"]
pub type TceR = crate::BitReader<Tce>;
impl TceR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tce {
        match self.bits {
            false => Tce::Tce0,
            true => Tce::Tce1,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_tce0(&self) -> bool {
        *self == Tce::Tce0
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_tce1(&self) -> bool {
        *self == Tce::Tce1
    }
}
#[doc = "Field `TCE` writer - Test Counter Enable"]
pub type TceW<'a, REG> = crate::BitWriter<'a, REG, Tce>;
impl<'a, REG> TceW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn tce0(self) -> &'a mut crate::W<REG> {
        self.variant(Tce::Tce0)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn tce1(self) -> &'a mut crate::W<REG> {
        self.variant(Tce::Tce1)
    }
}
#[doc = "Test Mode Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ten {
    #[doc = "0: Disabled"]
    Ten0 = 0,
    #[doc = "1: Enabled"]
    Ten1 = 1,
}
impl From<Ten> for bool {
    #[inline(always)]
    fn from(variant: Ten) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TEN` reader - Test Mode Enable"]
pub type TenR = crate::BitReader<Ten>;
impl TenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ten {
        match self.bits {
            false => Ten::Ten0,
            true => Ten::Ten1,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_ten0(&self) -> bool {
        *self == Ten::Ten0
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_ten1(&self) -> bool {
        *self == Ten::Ten1
    }
}
#[doc = "Field `TEN` writer - Test Mode Enable"]
pub type TenW<'a, REG> = crate::BitWriter<'a, REG, Ten>;
impl<'a, REG> TenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn ten0(self) -> &'a mut crate::W<REG> {
        self.variant(Ten::Ten0)
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn ten1(self) -> &'a mut crate::W<REG> {
        self.variant(Ten::Ten1)
    }
}
impl R {
    #[doc = "Bits 0:7 - TEST_COUNT"]
    #[inline(always)]
    pub fn test_count(&self) -> TestCountR {
        TestCountR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bits 8:12 - TEST_PERIOD"]
    #[inline(always)]
    pub fn test_period(&self) -> TestPeriodR {
        TestPeriodR::new(((self.bits >> 8) & 0x1f) as u8)
    }
    #[doc = "Bit 13 - Quadrature Decoder Negative Signal"]
    #[inline(always)]
    pub fn qdn(&self) -> QdnR {
        QdnR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Test Counter Enable"]
    #[inline(always)]
    pub fn tce(&self) -> TceR {
        TceR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Test Mode Enable"]
    #[inline(always)]
    pub fn ten(&self) -> TenR {
        TenR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:7 - TEST_COUNT"]
    #[inline(always)]
    pub fn test_count(&mut self) -> TestCountW<TstSpec> {
        TestCountW::new(self, 0)
    }
    #[doc = "Bits 8:12 - TEST_PERIOD"]
    #[inline(always)]
    pub fn test_period(&mut self) -> TestPeriodW<TstSpec> {
        TestPeriodW::new(self, 8)
    }
    #[doc = "Bit 13 - Quadrature Decoder Negative Signal"]
    #[inline(always)]
    pub fn qdn(&mut self) -> QdnW<TstSpec> {
        QdnW::new(self, 13)
    }
    #[doc = "Bit 14 - Test Counter Enable"]
    #[inline(always)]
    pub fn tce(&mut self) -> TceW<TstSpec> {
        TceW::new(self, 14)
    }
    #[doc = "Bit 15 - Test Mode Enable"]
    #[inline(always)]
    pub fn ten(&mut self) -> TenW<TstSpec> {
        TenW::new(self, 15)
    }
}
#[doc = "Test Register\n\nYou can [`read`](crate::Reg::read) this register and get [`tst::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`tst::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TstSpec;
impl crate::RegisterSpec for TstSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`tst::R`](R) reader structure"]
impl crate::Readable for TstSpec {}
#[doc = "`write(|w| ..)` method takes [`tst::W`](W) writer structure"]
impl crate::Writable for TstSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TST to value 0"]
impl crate::Resettable for TstSpec {}

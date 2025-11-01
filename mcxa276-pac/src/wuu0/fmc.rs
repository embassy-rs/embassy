#[doc = "Register `FMC` reader"]
pub type R = crate::R<FmcSpec>;
#[doc = "Register `FMC` writer"]
pub type W = crate::W<FmcSpec>;
#[doc = "Filter Mode for FILTn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filtm1 {
    #[doc = "0: Active only during Power Down/Deep Power Down mode"]
    LowPwrOnly = 0,
    #[doc = "1: Active during all power modes"]
    AnyPwr = 1,
}
impl From<Filtm1> for bool {
    #[inline(always)]
    fn from(variant: Filtm1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FILTM1` reader - Filter Mode for FILTn"]
pub type Filtm1R = crate::BitReader<Filtm1>;
impl Filtm1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Filtm1 {
        match self.bits {
            false => Filtm1::LowPwrOnly,
            true => Filtm1::AnyPwr,
        }
    }
    #[doc = "Active only during Power Down/Deep Power Down mode"]
    #[inline(always)]
    pub fn is_low_pwr_only(&self) -> bool {
        *self == Filtm1::LowPwrOnly
    }
    #[doc = "Active during all power modes"]
    #[inline(always)]
    pub fn is_any_pwr(&self) -> bool {
        *self == Filtm1::AnyPwr
    }
}
#[doc = "Field `FILTM1` writer - Filter Mode for FILTn"]
pub type Filtm1W<'a, REG> = crate::BitWriter<'a, REG, Filtm1>;
impl<'a, REG> Filtm1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Active only during Power Down/Deep Power Down mode"]
    #[inline(always)]
    pub fn low_pwr_only(self) -> &'a mut crate::W<REG> {
        self.variant(Filtm1::LowPwrOnly)
    }
    #[doc = "Active during all power modes"]
    #[inline(always)]
    pub fn any_pwr(self) -> &'a mut crate::W<REG> {
        self.variant(Filtm1::AnyPwr)
    }
}
#[doc = "Filter Mode for FILTn\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filtm2 {
    #[doc = "0: Active only during Power Down/Deep Power Down mode"]
    LowPwrOnly = 0,
    #[doc = "1: Active during all power modes"]
    AnyPwr = 1,
}
impl From<Filtm2> for bool {
    #[inline(always)]
    fn from(variant: Filtm2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FILTM2` reader - Filter Mode for FILTn"]
pub type Filtm2R = crate::BitReader<Filtm2>;
impl Filtm2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Filtm2 {
        match self.bits {
            false => Filtm2::LowPwrOnly,
            true => Filtm2::AnyPwr,
        }
    }
    #[doc = "Active only during Power Down/Deep Power Down mode"]
    #[inline(always)]
    pub fn is_low_pwr_only(&self) -> bool {
        *self == Filtm2::LowPwrOnly
    }
    #[doc = "Active during all power modes"]
    #[inline(always)]
    pub fn is_any_pwr(&self) -> bool {
        *self == Filtm2::AnyPwr
    }
}
#[doc = "Field `FILTM2` writer - Filter Mode for FILTn"]
pub type Filtm2W<'a, REG> = crate::BitWriter<'a, REG, Filtm2>;
impl<'a, REG> Filtm2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Active only during Power Down/Deep Power Down mode"]
    #[inline(always)]
    pub fn low_pwr_only(self) -> &'a mut crate::W<REG> {
        self.variant(Filtm2::LowPwrOnly)
    }
    #[doc = "Active during all power modes"]
    #[inline(always)]
    pub fn any_pwr(self) -> &'a mut crate::W<REG> {
        self.variant(Filtm2::AnyPwr)
    }
}
impl R {
    #[doc = "Bit 0 - Filter Mode for FILTn"]
    #[inline(always)]
    pub fn filtm1(&self) -> Filtm1R {
        Filtm1R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Filter Mode for FILTn"]
    #[inline(always)]
    pub fn filtm2(&self) -> Filtm2R {
        Filtm2R::new(((self.bits >> 1) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Filter Mode for FILTn"]
    #[inline(always)]
    pub fn filtm1(&mut self) -> Filtm1W<FmcSpec> {
        Filtm1W::new(self, 0)
    }
    #[doc = "Bit 1 - Filter Mode for FILTn"]
    #[inline(always)]
    pub fn filtm2(&mut self) -> Filtm2W<FmcSpec> {
        Filtm2W::new(self, 1)
    }
}
#[doc = "Pin Filter Mode Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`fmc::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fmc::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FmcSpec;
impl crate::RegisterSpec for FmcSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fmc::R`](R) reader structure"]
impl crate::Readable for FmcSpec {}
#[doc = "`write(|w| ..)` method takes [`fmc::W`](W) writer structure"]
impl crate::Writable for FmcSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FMC to value 0"]
impl crate::Resettable for FmcSpec {}

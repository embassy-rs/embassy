#[doc = "Register `FILT` reader"]
pub type R = crate::R<FiltSpec>;
#[doc = "Register `FILT` writer"]
pub type W = crate::W<FiltSpec>;
#[doc = "Field `FILTSEL1` reader - Filter 1 Pin Select"]
pub type Filtsel1R = crate::FieldReader;
#[doc = "Field `FILTSEL1` writer - Filter 1 Pin Select"]
pub type Filtsel1W<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Filter 1 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Filte1 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (Detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (Detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (Detect on any edge)"]
    EnAny = 3,
}
impl From<Filte1> for u8 {
    #[inline(always)]
    fn from(variant: Filte1) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Filte1 {
    type Ux = u8;
}
impl crate::IsEnum for Filte1 {}
#[doc = "Field `FILTE1` reader - Filter 1 Enable"]
pub type Filte1R = crate::FieldReader<Filte1>;
impl Filte1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Filte1 {
        match self.bits {
            0 => Filte1::Disable,
            1 => Filte1::EnRiseHi,
            2 => Filte1::EnFallLo,
            3 => Filte1::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Filte1::Disable
    }
    #[doc = "Enable (Detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Filte1::EnRiseHi
    }
    #[doc = "Enable (Detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Filte1::EnFallLo
    }
    #[doc = "Enable (Detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Filte1::EnAny
    }
}
#[doc = "Field `FILTE1` writer - Filter 1 Enable"]
pub type Filte1W<'a, REG> = crate::FieldWriter<'a, REG, 2, Filte1, crate::Safe>;
impl<'a, REG> Filte1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Filte1::Disable)
    }
    #[doc = "Enable (Detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Filte1::EnRiseHi)
    }
    #[doc = "Enable (Detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Filte1::EnFallLo)
    }
    #[doc = "Enable (Detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Filte1::EnAny)
    }
}
#[doc = "Filter 1 Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filtf1 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Filtf1> for bool {
    #[inline(always)]
    fn from(variant: Filtf1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FILTF1` reader - Filter 1 Flag"]
pub type Filtf1R = crate::BitReader<Filtf1>;
impl Filtf1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Filtf1 {
        match self.bits {
            false => Filtf1::NoFlag,
            true => Filtf1::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Filtf1::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Filtf1::Flag
    }
}
#[doc = "Field `FILTF1` writer - Filter 1 Flag"]
pub type Filtf1W<'a, REG> = crate::BitWriter1C<'a, REG, Filtf1>;
impl<'a, REG> Filtf1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Filtf1::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Filtf1::Flag)
    }
}
#[doc = "Field `FILTSEL2` reader - Filter 2 Pin Select"]
pub type Filtsel2R = crate::FieldReader;
#[doc = "Field `FILTSEL2` writer - Filter 2 Pin Select"]
pub type Filtsel2W<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Filter 2 Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Filte2 {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable (Detect on rising edge or high level)"]
    EnRiseHi = 1,
    #[doc = "2: Enable (Detect on falling edge or low level)"]
    EnFallLo = 2,
    #[doc = "3: Enable (Detect on any edge)"]
    EnAny = 3,
}
impl From<Filte2> for u8 {
    #[inline(always)]
    fn from(variant: Filte2) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Filte2 {
    type Ux = u8;
}
impl crate::IsEnum for Filte2 {}
#[doc = "Field `FILTE2` reader - Filter 2 Enable"]
pub type Filte2R = crate::FieldReader<Filte2>;
impl Filte2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Filte2 {
        match self.bits {
            0 => Filte2::Disable,
            1 => Filte2::EnRiseHi,
            2 => Filte2::EnFallLo,
            3 => Filte2::EnAny,
            _ => unreachable!(),
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Filte2::Disable
    }
    #[doc = "Enable (Detect on rising edge or high level)"]
    #[inline(always)]
    pub fn is_en_rise_hi(&self) -> bool {
        *self == Filte2::EnRiseHi
    }
    #[doc = "Enable (Detect on falling edge or low level)"]
    #[inline(always)]
    pub fn is_en_fall_lo(&self) -> bool {
        *self == Filte2::EnFallLo
    }
    #[doc = "Enable (Detect on any edge)"]
    #[inline(always)]
    pub fn is_en_any(&self) -> bool {
        *self == Filte2::EnAny
    }
}
#[doc = "Field `FILTE2` writer - Filter 2 Enable"]
pub type Filte2W<'a, REG> = crate::FieldWriter<'a, REG, 2, Filte2, crate::Safe>;
impl<'a, REG> Filte2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Filte2::Disable)
    }
    #[doc = "Enable (Detect on rising edge or high level)"]
    #[inline(always)]
    pub fn en_rise_hi(self) -> &'a mut crate::W<REG> {
        self.variant(Filte2::EnRiseHi)
    }
    #[doc = "Enable (Detect on falling edge or low level)"]
    #[inline(always)]
    pub fn en_fall_lo(self) -> &'a mut crate::W<REG> {
        self.variant(Filte2::EnFallLo)
    }
    #[doc = "Enable (Detect on any edge)"]
    #[inline(always)]
    pub fn en_any(self) -> &'a mut crate::W<REG> {
        self.variant(Filte2::EnAny)
    }
}
#[doc = "Filter 2 Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filtf2 {
    #[doc = "0: No"]
    NoFlag = 0,
    #[doc = "1: Yes"]
    Flag = 1,
}
impl From<Filtf2> for bool {
    #[inline(always)]
    fn from(variant: Filtf2) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FILTF2` reader - Filter 2 Flag"]
pub type Filtf2R = crate::BitReader<Filtf2>;
impl Filtf2R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Filtf2 {
        match self.bits {
            false => Filtf2::NoFlag,
            true => Filtf2::Flag,
        }
    }
    #[doc = "No"]
    #[inline(always)]
    pub fn is_no_flag(&self) -> bool {
        *self == Filtf2::NoFlag
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn is_flag(&self) -> bool {
        *self == Filtf2::Flag
    }
}
#[doc = "Field `FILTF2` writer - Filter 2 Flag"]
pub type Filtf2W<'a, REG> = crate::BitWriter1C<'a, REG, Filtf2>;
impl<'a, REG> Filtf2W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No"]
    #[inline(always)]
    pub fn no_flag(self) -> &'a mut crate::W<REG> {
        self.variant(Filtf2::NoFlag)
    }
    #[doc = "Yes"]
    #[inline(always)]
    pub fn flag(self) -> &'a mut crate::W<REG> {
        self.variant(Filtf2::Flag)
    }
}
impl R {
    #[doc = "Bits 0:4 - Filter 1 Pin Select"]
    #[inline(always)]
    pub fn filtsel1(&self) -> Filtsel1R {
        Filtsel1R::new((self.bits & 0x1f) as u8)
    }
    #[doc = "Bits 5:6 - Filter 1 Enable"]
    #[inline(always)]
    pub fn filte1(&self) -> Filte1R {
        Filte1R::new(((self.bits >> 5) & 3) as u8)
    }
    #[doc = "Bit 7 - Filter 1 Flag"]
    #[inline(always)]
    pub fn filtf1(&self) -> Filtf1R {
        Filtf1R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bits 8:12 - Filter 2 Pin Select"]
    #[inline(always)]
    pub fn filtsel2(&self) -> Filtsel2R {
        Filtsel2R::new(((self.bits >> 8) & 0x1f) as u8)
    }
    #[doc = "Bits 13:14 - Filter 2 Enable"]
    #[inline(always)]
    pub fn filte2(&self) -> Filte2R {
        Filte2R::new(((self.bits >> 13) & 3) as u8)
    }
    #[doc = "Bit 15 - Filter 2 Flag"]
    #[inline(always)]
    pub fn filtf2(&self) -> Filtf2R {
        Filtf2R::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:4 - Filter 1 Pin Select"]
    #[inline(always)]
    pub fn filtsel1(&mut self) -> Filtsel1W<FiltSpec> {
        Filtsel1W::new(self, 0)
    }
    #[doc = "Bits 5:6 - Filter 1 Enable"]
    #[inline(always)]
    pub fn filte1(&mut self) -> Filte1W<FiltSpec> {
        Filte1W::new(self, 5)
    }
    #[doc = "Bit 7 - Filter 1 Flag"]
    #[inline(always)]
    pub fn filtf1(&mut self) -> Filtf1W<FiltSpec> {
        Filtf1W::new(self, 7)
    }
    #[doc = "Bits 8:12 - Filter 2 Pin Select"]
    #[inline(always)]
    pub fn filtsel2(&mut self) -> Filtsel2W<FiltSpec> {
        Filtsel2W::new(self, 8)
    }
    #[doc = "Bits 13:14 - Filter 2 Enable"]
    #[inline(always)]
    pub fn filte2(&mut self) -> Filte2W<FiltSpec> {
        Filte2W::new(self, 13)
    }
    #[doc = "Bit 15 - Filter 2 Flag"]
    #[inline(always)]
    pub fn filtf2(&mut self) -> Filtf2W<FiltSpec> {
        Filtf2W::new(self, 15)
    }
}
#[doc = "Pin Filter\n\nYou can [`read`](crate::Reg::read) this register and get [`filt::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`filt::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FiltSpec;
impl crate::RegisterSpec for FiltSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`filt::R`](R) reader structure"]
impl crate::Readable for FiltSpec {}
#[doc = "`write(|w| ..)` method takes [`filt::W`](W) writer structure"]
impl crate::Writable for FiltSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x8080;
}
#[doc = "`reset()` method sets FILT to value 0"]
impl crate::Resettable for FiltSpec {}

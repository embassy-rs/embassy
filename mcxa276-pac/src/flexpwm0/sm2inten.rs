#[doc = "Register `SM2INTEN` reader"]
pub type R = crate::R<Sm2intenSpec>;
#[doc = "Register `SM2INTEN` writer"]
pub type W = crate::W<Sm2intenSpec>;
#[doc = "Compare Interrupt Enables\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Cmpie {
    #[doc = "0: The corresponding STS\\[CMPF\\] bit will not cause an interrupt request."]
    Disabled = 0,
    #[doc = "1: The corresponding STS\\[CMPF\\] bit will cause an interrupt request."]
    Enabled = 1,
}
impl From<Cmpie> for u8 {
    #[inline(always)]
    fn from(variant: Cmpie) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Cmpie {
    type Ux = u8;
}
impl crate::IsEnum for Cmpie {}
#[doc = "Field `CMPIE` reader - Compare Interrupt Enables"]
pub type CmpieR = crate::FieldReader<Cmpie>;
impl CmpieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Cmpie> {
        match self.bits {
            0 => Some(Cmpie::Disabled),
            1 => Some(Cmpie::Enabled),
            _ => None,
        }
    }
    #[doc = "The corresponding STS\\[CMPF\\] bit will not cause an interrupt request."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Cmpie::Disabled
    }
    #[doc = "The corresponding STS\\[CMPF\\] bit will cause an interrupt request."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Cmpie::Enabled
    }
}
#[doc = "Field `CMPIE` writer - Compare Interrupt Enables"]
pub type CmpieW<'a, REG> = crate::FieldWriter<'a, REG, 6, Cmpie>;
impl<'a, REG> CmpieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "The corresponding STS\\[CMPF\\] bit will not cause an interrupt request."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cmpie::Disabled)
    }
    #[doc = "The corresponding STS\\[CMPF\\] bit will cause an interrupt request."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cmpie::Enabled)
    }
}
#[doc = "Capture X 0 Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cx0ie {
    #[doc = "0: Interrupt request disabled for STS\\[CFX0\\]."]
    Disabled = 0,
    #[doc = "1: Interrupt request enabled for STS\\[CFX0\\]."]
    Enabled = 1,
}
impl From<Cx0ie> for bool {
    #[inline(always)]
    fn from(variant: Cx0ie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CX0IE` reader - Capture X 0 Interrupt Enable"]
pub type Cx0ieR = crate::BitReader<Cx0ie>;
impl Cx0ieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cx0ie {
        match self.bits {
            false => Cx0ie::Disabled,
            true => Cx0ie::Enabled,
        }
    }
    #[doc = "Interrupt request disabled for STS\\[CFX0\\]."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Cx0ie::Disabled
    }
    #[doc = "Interrupt request enabled for STS\\[CFX0\\]."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Cx0ie::Enabled
    }
}
#[doc = "Field `CX0IE` writer - Capture X 0 Interrupt Enable"]
pub type Cx0ieW<'a, REG> = crate::BitWriter<'a, REG, Cx0ie>;
impl<'a, REG> Cx0ieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt request disabled for STS\\[CFX0\\]."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cx0ie::Disabled)
    }
    #[doc = "Interrupt request enabled for STS\\[CFX0\\]."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cx0ie::Enabled)
    }
}
#[doc = "Capture X 1 Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cx1ie {
    #[doc = "0: Interrupt request disabled for STS\\[CFX1\\]."]
    Disabled = 0,
    #[doc = "1: Interrupt request enabled for STS\\[CFX1\\]."]
    Enabled = 1,
}
impl From<Cx1ie> for bool {
    #[inline(always)]
    fn from(variant: Cx1ie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CX1IE` reader - Capture X 1 Interrupt Enable"]
pub type Cx1ieR = crate::BitReader<Cx1ie>;
impl Cx1ieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cx1ie {
        match self.bits {
            false => Cx1ie::Disabled,
            true => Cx1ie::Enabled,
        }
    }
    #[doc = "Interrupt request disabled for STS\\[CFX1\\]."]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Cx1ie::Disabled
    }
    #[doc = "Interrupt request enabled for STS\\[CFX1\\]."]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Cx1ie::Enabled
    }
}
#[doc = "Field `CX1IE` writer - Capture X 1 Interrupt Enable"]
pub type Cx1ieW<'a, REG> = crate::BitWriter<'a, REG, Cx1ie>;
impl<'a, REG> Cx1ieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt request disabled for STS\\[CFX1\\]."]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cx1ie::Disabled)
    }
    #[doc = "Interrupt request enabled for STS\\[CFX1\\]."]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Cx1ie::Enabled)
    }
}
#[doc = "Reload Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rie {
    #[doc = "0: STS\\[RF\\] CPU interrupt requests disabled"]
    Disabled = 0,
    #[doc = "1: STS\\[RF\\] CPU interrupt requests enabled"]
    Enabled = 1,
}
impl From<Rie> for bool {
    #[inline(always)]
    fn from(variant: Rie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RIE` reader - Reload Interrupt Enable"]
pub type RieR = crate::BitReader<Rie>;
impl RieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rie {
        match self.bits {
            false => Rie::Disabled,
            true => Rie::Enabled,
        }
    }
    #[doc = "STS\\[RF\\] CPU interrupt requests disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rie::Disabled
    }
    #[doc = "STS\\[RF\\] CPU interrupt requests enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rie::Enabled
    }
}
#[doc = "Field `RIE` writer - Reload Interrupt Enable"]
pub type RieW<'a, REG> = crate::BitWriter<'a, REG, Rie>;
impl<'a, REG> RieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "STS\\[RF\\] CPU interrupt requests disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rie::Disabled)
    }
    #[doc = "STS\\[RF\\] CPU interrupt requests enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rie::Enabled)
    }
}
#[doc = "Reload Error Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reie {
    #[doc = "0: STS\\[REF\\] CPU interrupt requests disabled"]
    Disabled = 0,
    #[doc = "1: STS\\[REF\\] CPU interrupt requests enabled"]
    Enabled = 1,
}
impl From<Reie> for bool {
    #[inline(always)]
    fn from(variant: Reie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `REIE` reader - Reload Error Interrupt Enable"]
pub type ReieR = crate::BitReader<Reie>;
impl ReieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Reie {
        match self.bits {
            false => Reie::Disabled,
            true => Reie::Enabled,
        }
    }
    #[doc = "STS\\[REF\\] CPU interrupt requests disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Reie::Disabled
    }
    #[doc = "STS\\[REF\\] CPU interrupt requests enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Reie::Enabled
    }
}
#[doc = "Field `REIE` writer - Reload Error Interrupt Enable"]
pub type ReieW<'a, REG> = crate::BitWriter<'a, REG, Reie>;
impl<'a, REG> ReieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "STS\\[REF\\] CPU interrupt requests disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Reie::Disabled)
    }
    #[doc = "STS\\[REF\\] CPU interrupt requests enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Reie::Enabled)
    }
}
impl R {
    #[doc = "Bits 0:5 - Compare Interrupt Enables"]
    #[inline(always)]
    pub fn cmpie(&self) -> CmpieR {
        CmpieR::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bit 6 - Capture X 0 Interrupt Enable"]
    #[inline(always)]
    pub fn cx0ie(&self) -> Cx0ieR {
        Cx0ieR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Capture X 1 Interrupt Enable"]
    #[inline(always)]
    pub fn cx1ie(&self) -> Cx1ieR {
        Cx1ieR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 12 - Reload Interrupt Enable"]
    #[inline(always)]
    pub fn rie(&self) -> RieR {
        RieR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Reload Error Interrupt Enable"]
    #[inline(always)]
    pub fn reie(&self) -> ReieR {
        ReieR::new(((self.bits >> 13) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:5 - Compare Interrupt Enables"]
    #[inline(always)]
    pub fn cmpie(&mut self) -> CmpieW<Sm2intenSpec> {
        CmpieW::new(self, 0)
    }
    #[doc = "Bit 6 - Capture X 0 Interrupt Enable"]
    #[inline(always)]
    pub fn cx0ie(&mut self) -> Cx0ieW<Sm2intenSpec> {
        Cx0ieW::new(self, 6)
    }
    #[doc = "Bit 7 - Capture X 1 Interrupt Enable"]
    #[inline(always)]
    pub fn cx1ie(&mut self) -> Cx1ieW<Sm2intenSpec> {
        Cx1ieW::new(self, 7)
    }
    #[doc = "Bit 12 - Reload Interrupt Enable"]
    #[inline(always)]
    pub fn rie(&mut self) -> RieW<Sm2intenSpec> {
        RieW::new(self, 12)
    }
    #[doc = "Bit 13 - Reload Error Interrupt Enable"]
    #[inline(always)]
    pub fn reie(&mut self) -> ReieW<Sm2intenSpec> {
        ReieW::new(self, 13)
    }
}
#[doc = "Interrupt Enable Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sm2inten::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sm2inten::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Sm2intenSpec;
impl crate::RegisterSpec for Sm2intenSpec {
    type Ux = u16;
}
#[doc = "`read()` method returns [`sm2inten::R`](R) reader structure"]
impl crate::Readable for Sm2intenSpec {}
#[doc = "`write(|w| ..)` method takes [`sm2inten::W`](W) writer structure"]
impl crate::Writable for Sm2intenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SM2INTEN to value 0"]
impl crate::Resettable for Sm2intenSpec {}

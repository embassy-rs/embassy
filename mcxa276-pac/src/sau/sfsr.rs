#[doc = "Register `SFSR` reader"]
pub type R = crate::R<SfsrSpec>;
#[doc = "Register `SFSR` writer"]
pub type W = crate::W<SfsrSpec>;
#[doc = "Invalid entry point.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invep {
    #[doc = "0: Error has not occurred."]
    NoError = 0,
    #[doc = "1: Error has occurred."]
    Error = 1,
}
impl From<Invep> for bool {
    #[inline(always)]
    fn from(variant: Invep) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INVEP` reader - Invalid entry point."]
pub type InvepR = crate::BitReader<Invep>;
impl InvepR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Invep {
        match self.bits {
            false => Invep::NoError,
            true => Invep::Error,
        }
    }
    #[doc = "Error has not occurred."]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Invep::NoError
    }
    #[doc = "Error has occurred."]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Invep::Error
    }
}
#[doc = "Field `INVEP` writer - Invalid entry point."]
pub type InvepW<'a, REG> = crate::BitWriter<'a, REG, Invep>;
impl<'a, REG> InvepW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Error has not occurred."]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Invep::NoError)
    }
    #[doc = "Error has occurred."]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Invep::Error)
    }
}
#[doc = "Invalid integrity signature flag.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invis {
    #[doc = "0: Error has not occurred."]
    NoError = 0,
    #[doc = "1: Error has occurred."]
    Error = 1,
}
impl From<Invis> for bool {
    #[inline(always)]
    fn from(variant: Invis) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INVIS` reader - Invalid integrity signature flag."]
pub type InvisR = crate::BitReader<Invis>;
impl InvisR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Invis {
        match self.bits {
            false => Invis::NoError,
            true => Invis::Error,
        }
    }
    #[doc = "Error has not occurred."]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Invis::NoError
    }
    #[doc = "Error has occurred."]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Invis::Error
    }
}
#[doc = "Field `INVIS` writer - Invalid integrity signature flag."]
pub type InvisW<'a, REG> = crate::BitWriter<'a, REG, Invis>;
impl<'a, REG> InvisW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Error has not occurred."]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Invis::NoError)
    }
    #[doc = "Error has occurred."]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Invis::Error)
    }
}
#[doc = "Invalid exception return flag.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inver {
    #[doc = "0: Error has not occurred."]
    NoError = 0,
    #[doc = "1: Error has occurred."]
    Error = 1,
}
impl From<Inver> for bool {
    #[inline(always)]
    fn from(variant: Inver) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INVER` reader - Invalid exception return flag."]
pub type InverR = crate::BitReader<Inver>;
impl InverR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Inver {
        match self.bits {
            false => Inver::NoError,
            true => Inver::Error,
        }
    }
    #[doc = "Error has not occurred."]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Inver::NoError
    }
    #[doc = "Error has occurred."]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Inver::Error
    }
}
#[doc = "Field `INVER` writer - Invalid exception return flag."]
pub type InverW<'a, REG> = crate::BitWriter<'a, REG, Inver>;
impl<'a, REG> InverW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Error has not occurred."]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Inver::NoError)
    }
    #[doc = "Error has occurred."]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Inver::Error)
    }
}
#[doc = "Attribution unit violation flag.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Auviol {
    #[doc = "0: Error has not occurred."]
    NoError = 0,
    #[doc = "1: Error has occurred."]
    Error = 1,
}
impl From<Auviol> for bool {
    #[inline(always)]
    fn from(variant: Auviol) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AUVIOL` reader - Attribution unit violation flag."]
pub type AuviolR = crate::BitReader<Auviol>;
impl AuviolR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Auviol {
        match self.bits {
            false => Auviol::NoError,
            true => Auviol::Error,
        }
    }
    #[doc = "Error has not occurred."]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Auviol::NoError
    }
    #[doc = "Error has occurred."]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Auviol::Error
    }
}
#[doc = "Field `AUVIOL` writer - Attribution unit violation flag."]
pub type AuviolW<'a, REG> = crate::BitWriter<'a, REG, Auviol>;
impl<'a, REG> AuviolW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Error has not occurred."]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Auviol::NoError)
    }
    #[doc = "Error has occurred."]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Auviol::Error)
    }
}
#[doc = "Invalid transition flag.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invtran {
    #[doc = "0: Error has not occurred."]
    NoError = 0,
    #[doc = "1: Error has occurred."]
    Error = 1,
}
impl From<Invtran> for bool {
    #[inline(always)]
    fn from(variant: Invtran) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INVTRAN` reader - Invalid transition flag."]
pub type InvtranR = crate::BitReader<Invtran>;
impl InvtranR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Invtran {
        match self.bits {
            false => Invtran::NoError,
            true => Invtran::Error,
        }
    }
    #[doc = "Error has not occurred."]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Invtran::NoError
    }
    #[doc = "Error has occurred."]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Invtran::Error
    }
}
#[doc = "Field `INVTRAN` writer - Invalid transition flag."]
pub type InvtranW<'a, REG> = crate::BitWriter<'a, REG, Invtran>;
impl<'a, REG> InvtranW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Error has not occurred."]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Invtran::NoError)
    }
    #[doc = "Error has occurred."]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Invtran::Error)
    }
}
#[doc = "Lazy state preservation error flag.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lsperr {
    #[doc = "0: Error has not occurred."]
    NoError = 0,
    #[doc = "1: Error has occurred."]
    Error = 1,
}
impl From<Lsperr> for bool {
    #[inline(always)]
    fn from(variant: Lsperr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LSPERR` reader - Lazy state preservation error flag."]
pub type LsperrR = crate::BitReader<Lsperr>;
impl LsperrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lsperr {
        match self.bits {
            false => Lsperr::NoError,
            true => Lsperr::Error,
        }
    }
    #[doc = "Error has not occurred."]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Lsperr::NoError
    }
    #[doc = "Error has occurred."]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Lsperr::Error
    }
}
#[doc = "Field `LSPERR` writer - Lazy state preservation error flag."]
pub type LsperrW<'a, REG> = crate::BitWriter<'a, REG, Lsperr>;
impl<'a, REG> LsperrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Error has not occurred."]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Lsperr::NoError)
    }
    #[doc = "Error has occurred."]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Lsperr::Error)
    }
}
#[doc = "Secure fault address valid.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sfarvalid {
    #[doc = "0: SFAR content not valid."]
    NotValid = 0,
    #[doc = "1: SFAR content valid."]
    Valid = 1,
}
impl From<Sfarvalid> for bool {
    #[inline(always)]
    fn from(variant: Sfarvalid) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SFARVALID` reader - Secure fault address valid."]
pub type SfarvalidR = crate::BitReader<Sfarvalid>;
impl SfarvalidR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sfarvalid {
        match self.bits {
            false => Sfarvalid::NotValid,
            true => Sfarvalid::Valid,
        }
    }
    #[doc = "SFAR content not valid."]
    #[inline(always)]
    pub fn is_not_valid(&self) -> bool {
        *self == Sfarvalid::NotValid
    }
    #[doc = "SFAR content valid."]
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        *self == Sfarvalid::Valid
    }
}
#[doc = "Field `SFARVALID` writer - Secure fault address valid."]
pub type SfarvalidW<'a, REG> = crate::BitWriter<'a, REG, Sfarvalid>;
impl<'a, REG> SfarvalidW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SFAR content not valid."]
    #[inline(always)]
    pub fn not_valid(self) -> &'a mut crate::W<REG> {
        self.variant(Sfarvalid::NotValid)
    }
    #[doc = "SFAR content valid."]
    #[inline(always)]
    pub fn valid(self) -> &'a mut crate::W<REG> {
        self.variant(Sfarvalid::Valid)
    }
}
#[doc = "Lazy state error flag.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lserr {
    #[doc = "0: Error has not occurred"]
    NoError = 0,
    #[doc = "1: Error has occurred."]
    Error = 1,
}
impl From<Lserr> for bool {
    #[inline(always)]
    fn from(variant: Lserr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LSERR` reader - Lazy state error flag."]
pub type LserrR = crate::BitReader<Lserr>;
impl LserrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lserr {
        match self.bits {
            false => Lserr::NoError,
            true => Lserr::Error,
        }
    }
    #[doc = "Error has not occurred"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Lserr::NoError
    }
    #[doc = "Error has occurred."]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Lserr::Error
    }
}
#[doc = "Field `LSERR` writer - Lazy state error flag."]
pub type LserrW<'a, REG> = crate::BitWriter<'a, REG, Lserr>;
impl<'a, REG> LserrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Error has not occurred"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Lserr::NoError)
    }
    #[doc = "Error has occurred."]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Lserr::Error)
    }
}
impl R {
    #[doc = "Bit 0 - Invalid entry point."]
    #[inline(always)]
    pub fn invep(&self) -> InvepR {
        InvepR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Invalid integrity signature flag."]
    #[inline(always)]
    pub fn invis(&self) -> InvisR {
        InvisR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Invalid exception return flag."]
    #[inline(always)]
    pub fn inver(&self) -> InverR {
        InverR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Attribution unit violation flag."]
    #[inline(always)]
    pub fn auviol(&self) -> AuviolR {
        AuviolR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Invalid transition flag."]
    #[inline(always)]
    pub fn invtran(&self) -> InvtranR {
        InvtranR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Lazy state preservation error flag."]
    #[inline(always)]
    pub fn lsperr(&self) -> LsperrR {
        LsperrR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Secure fault address valid."]
    #[inline(always)]
    pub fn sfarvalid(&self) -> SfarvalidR {
        SfarvalidR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Lazy state error flag."]
    #[inline(always)]
    pub fn lserr(&self) -> LserrR {
        LserrR::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Invalid entry point."]
    #[inline(always)]
    pub fn invep(&mut self) -> InvepW<SfsrSpec> {
        InvepW::new(self, 0)
    }
    #[doc = "Bit 1 - Invalid integrity signature flag."]
    #[inline(always)]
    pub fn invis(&mut self) -> InvisW<SfsrSpec> {
        InvisW::new(self, 1)
    }
    #[doc = "Bit 2 - Invalid exception return flag."]
    #[inline(always)]
    pub fn inver(&mut self) -> InverW<SfsrSpec> {
        InverW::new(self, 2)
    }
    #[doc = "Bit 3 - Attribution unit violation flag."]
    #[inline(always)]
    pub fn auviol(&mut self) -> AuviolW<SfsrSpec> {
        AuviolW::new(self, 3)
    }
    #[doc = "Bit 4 - Invalid transition flag."]
    #[inline(always)]
    pub fn invtran(&mut self) -> InvtranW<SfsrSpec> {
        InvtranW::new(self, 4)
    }
    #[doc = "Bit 5 - Lazy state preservation error flag."]
    #[inline(always)]
    pub fn lsperr(&mut self) -> LsperrW<SfsrSpec> {
        LsperrW::new(self, 5)
    }
    #[doc = "Bit 6 - Secure fault address valid."]
    #[inline(always)]
    pub fn sfarvalid(&mut self) -> SfarvalidW<SfsrSpec> {
        SfarvalidW::new(self, 6)
    }
    #[doc = "Bit 7 - Lazy state error flag."]
    #[inline(always)]
    pub fn lserr(&mut self) -> LserrW<SfsrSpec> {
        LserrW::new(self, 7)
    }
}
#[doc = "Secure Fault Status Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sfsr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sfsr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SfsrSpec;
impl crate::RegisterSpec for SfsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sfsr::R`](R) reader structure"]
impl crate::Readable for SfsrSpec {}
#[doc = "`write(|w| ..)` method takes [`sfsr::W`](W) writer structure"]
impl crate::Writable for SfsrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SFSR to value 0"]
impl crate::Resettable for SfsrSpec {}

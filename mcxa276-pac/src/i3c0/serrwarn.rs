#[doc = "Register `SERRWARN` reader"]
pub type R = crate::R<SerrwarnSpec>;
#[doc = "Register `SERRWARN` writer"]
pub type W = crate::W<SerrwarnSpec>;
#[doc = "Overrun Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orun {
    #[doc = "0: No overrun error"]
    NoError = 0,
    #[doc = "1: Overrun error"]
    Error = 1,
}
impl From<Orun> for bool {
    #[inline(always)]
    fn from(variant: Orun) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ORUN` reader - Overrun Error Flag"]
pub type OrunR = crate::BitReader<Orun>;
impl OrunR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Orun {
        match self.bits {
            false => Orun::NoError,
            true => Orun::Error,
        }
    }
    #[doc = "No overrun error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Orun::NoError
    }
    #[doc = "Overrun error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Orun::Error
    }
}
#[doc = "Field `ORUN` writer - Overrun Error Flag"]
pub type OrunW<'a, REG> = crate::BitWriter1C<'a, REG, Orun>;
impl<'a, REG> OrunW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No overrun error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Orun::NoError)
    }
    #[doc = "Overrun error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Orun::Error)
    }
}
#[doc = "Underrun Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Urun {
    #[doc = "0: No underrun error"]
    NoError = 0,
    #[doc = "1: Underrun error"]
    Error = 1,
}
impl From<Urun> for bool {
    #[inline(always)]
    fn from(variant: Urun) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `URUN` reader - Underrun Error Flag"]
pub type UrunR = crate::BitReader<Urun>;
impl UrunR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Urun {
        match self.bits {
            false => Urun::NoError,
            true => Urun::Error,
        }
    }
    #[doc = "No underrun error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Urun::NoError
    }
    #[doc = "Underrun error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Urun::Error
    }
}
#[doc = "Field `URUN` writer - Underrun Error Flag"]
pub type UrunW<'a, REG> = crate::BitWriter1C<'a, REG, Urun>;
impl<'a, REG> UrunW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No underrun error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Urun::NoError)
    }
    #[doc = "Underrun error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Urun::Error)
    }
}
#[doc = "Underrun and Not Acknowledged (NACKed) Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Urunnack {
    #[doc = "0: No underrun; not acknowledged error"]
    NoError = 0,
    #[doc = "1: Underrun; not acknowledged error"]
    Error = 1,
}
impl From<Urunnack> for bool {
    #[inline(always)]
    fn from(variant: Urunnack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `URUNNACK` reader - Underrun and Not Acknowledged (NACKed) Error Flag"]
pub type UrunnackR = crate::BitReader<Urunnack>;
impl UrunnackR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Urunnack {
        match self.bits {
            false => Urunnack::NoError,
            true => Urunnack::Error,
        }
    }
    #[doc = "No underrun; not acknowledged error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Urunnack::NoError
    }
    #[doc = "Underrun; not acknowledged error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Urunnack::Error
    }
}
#[doc = "Field `URUNNACK` writer - Underrun and Not Acknowledged (NACKed) Error Flag"]
pub type UrunnackW<'a, REG> = crate::BitWriter1C<'a, REG, Urunnack>;
impl<'a, REG> UrunnackW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No underrun; not acknowledged error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Urunnack::NoError)
    }
    #[doc = "Underrun; not acknowledged error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Urunnack::Error)
    }
}
#[doc = "Terminated Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Term {
    #[doc = "0: No terminated error"]
    NoError = 0,
    #[doc = "1: Terminated error"]
    Error = 1,
}
impl From<Term> for bool {
    #[inline(always)]
    fn from(variant: Term) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TERM` reader - Terminated Error Flag"]
pub type TermR = crate::BitReader<Term>;
impl TermR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Term {
        match self.bits {
            false => Term::NoError,
            true => Term::Error,
        }
    }
    #[doc = "No terminated error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Term::NoError
    }
    #[doc = "Terminated error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Term::Error
    }
}
#[doc = "Field `TERM` writer - Terminated Error Flag"]
pub type TermW<'a, REG> = crate::BitWriter1C<'a, REG, Term>;
impl<'a, REG> TermW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No terminated error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Term::NoError)
    }
    #[doc = "Terminated error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Term::Error)
    }
}
#[doc = "Invalid Start Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invstart {
    #[doc = "0: No invalid start error"]
    NoError = 0,
    #[doc = "1: Invalid start error"]
    Error = 1,
}
impl From<Invstart> for bool {
    #[inline(always)]
    fn from(variant: Invstart) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INVSTART` reader - Invalid Start Error Flag"]
pub type InvstartR = crate::BitReader<Invstart>;
impl InvstartR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Invstart {
        match self.bits {
            false => Invstart::NoError,
            true => Invstart::Error,
        }
    }
    #[doc = "No invalid start error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Invstart::NoError
    }
    #[doc = "Invalid start error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Invstart::Error
    }
}
#[doc = "Field `INVSTART` writer - Invalid Start Error Flag"]
pub type InvstartW<'a, REG> = crate::BitWriter1C<'a, REG, Invstart>;
impl<'a, REG> InvstartW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No invalid start error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Invstart::NoError)
    }
    #[doc = "Invalid start error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Invstart::Error)
    }
}
#[doc = "SDR Parity Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spar {
    #[doc = "0: No SDR parity error"]
    NoError = 0,
    #[doc = "1: SDR parity error"]
    Error = 1,
}
impl From<Spar> for bool {
    #[inline(always)]
    fn from(variant: Spar) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPAR` reader - SDR Parity Error Flag"]
pub type SparR = crate::BitReader<Spar>;
impl SparR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Spar {
        match self.bits {
            false => Spar::NoError,
            true => Spar::Error,
        }
    }
    #[doc = "No SDR parity error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Spar::NoError
    }
    #[doc = "SDR parity error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Spar::Error
    }
}
#[doc = "Field `SPAR` writer - SDR Parity Error Flag"]
pub type SparW<'a, REG> = crate::BitWriter1C<'a, REG, Spar>;
impl<'a, REG> SparW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No SDR parity error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Spar::NoError)
    }
    #[doc = "SDR parity error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Spar::Error)
    }
}
#[doc = "HDR Parity Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hpar {
    #[doc = "0: No HDR parity error"]
    NoError = 0,
    #[doc = "1: HDR parity error"]
    Error = 1,
}
impl From<Hpar> for bool {
    #[inline(always)]
    fn from(variant: Hpar) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HPAR` reader - HDR Parity Error Flag"]
pub type HparR = crate::BitReader<Hpar>;
impl HparR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hpar {
        match self.bits {
            false => Hpar::NoError,
            true => Hpar::Error,
        }
    }
    #[doc = "No HDR parity error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Hpar::NoError
    }
    #[doc = "HDR parity error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Hpar::Error
    }
}
#[doc = "Field `HPAR` writer - HDR Parity Error Flag"]
pub type HparW<'a, REG> = crate::BitWriter1C<'a, REG, Hpar>;
impl<'a, REG> HparW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No HDR parity error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Hpar::NoError)
    }
    #[doc = "HDR parity error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Hpar::Error)
    }
}
#[doc = "HDR-DDR CRC Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hcrc {
    #[doc = "0: No HDR-DDR CRC error occurred"]
    NoError = 0,
    #[doc = "1: HDR-DDR CRC error occurred"]
    Error = 1,
}
impl From<Hcrc> for bool {
    #[inline(always)]
    fn from(variant: Hcrc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HCRC` reader - HDR-DDR CRC Error Flag"]
pub type HcrcR = crate::BitReader<Hcrc>;
impl HcrcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hcrc {
        match self.bits {
            false => Hcrc::NoError,
            true => Hcrc::Error,
        }
    }
    #[doc = "No HDR-DDR CRC error occurred"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Hcrc::NoError
    }
    #[doc = "HDR-DDR CRC error occurred"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Hcrc::Error
    }
}
#[doc = "Field `HCRC` writer - HDR-DDR CRC Error Flag"]
pub type HcrcW<'a, REG> = crate::BitWriter1C<'a, REG, Hcrc>;
impl<'a, REG> HcrcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No HDR-DDR CRC error occurred"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Hcrc::NoError)
    }
    #[doc = "HDR-DDR CRC error occurred"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Hcrc::Error)
    }
}
#[doc = "TE0 or TE1 Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum S0s1 {
    #[doc = "0: No TE0 or TE1 error occurred"]
    NoError = 0,
    #[doc = "1: TE0 or TE1 error occurred"]
    Error = 1,
}
impl From<S0s1> for bool {
    #[inline(always)]
    fn from(variant: S0s1) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `S0S1` reader - TE0 or TE1 Error Flag"]
pub type S0s1R = crate::BitReader<S0s1>;
impl S0s1R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> S0s1 {
        match self.bits {
            false => S0s1::NoError,
            true => S0s1::Error,
        }
    }
    #[doc = "No TE0 or TE1 error occurred"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == S0s1::NoError
    }
    #[doc = "TE0 or TE1 error occurred"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == S0s1::Error
    }
}
#[doc = "Field `S0S1` writer - TE0 or TE1 Error Flag"]
pub type S0s1W<'a, REG> = crate::BitWriter1C<'a, REG, S0s1>;
impl<'a, REG> S0s1W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No TE0 or TE1 error occurred"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(S0s1::NoError)
    }
    #[doc = "TE0 or TE1 error occurred"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(S0s1::Error)
    }
}
#[doc = "Over-Read Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Oread {
    #[doc = "0: No over-read error"]
    NoError = 0,
    #[doc = "1: Over-read error"]
    Error = 1,
}
impl From<Oread> for bool {
    #[inline(always)]
    fn from(variant: Oread) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OREAD` reader - Over-Read Error Flag"]
pub type OreadR = crate::BitReader<Oread>;
impl OreadR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Oread {
        match self.bits {
            false => Oread::NoError,
            true => Oread::Error,
        }
    }
    #[doc = "No over-read error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Oread::NoError
    }
    #[doc = "Over-read error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Oread::Error
    }
}
#[doc = "Field `OREAD` writer - Over-Read Error Flag"]
pub type OreadW<'a, REG> = crate::BitWriter1C<'a, REG, Oread>;
impl<'a, REG> OreadW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No over-read error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Oread::NoError)
    }
    #[doc = "Over-read error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Oread::Error)
    }
}
#[doc = "Over-Write Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owrite {
    #[doc = "0: No overwrite error"]
    NoError = 0,
    #[doc = "1: Overwrite error"]
    Error = 1,
}
impl From<Owrite> for bool {
    #[inline(always)]
    fn from(variant: Owrite) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OWRITE` reader - Over-Write Error Flag"]
pub type OwriteR = crate::BitReader<Owrite>;
impl OwriteR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Owrite {
        match self.bits {
            false => Owrite::NoError,
            true => Owrite::Error,
        }
    }
    #[doc = "No overwrite error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Owrite::NoError
    }
    #[doc = "Overwrite error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Owrite::Error
    }
}
#[doc = "Field `OWRITE` writer - Over-Write Error Flag"]
pub type OwriteW<'a, REG> = crate::BitWriter1C<'a, REG, Owrite>;
impl<'a, REG> OwriteW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No overwrite error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Owrite::NoError)
    }
    #[doc = "Overwrite error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Owrite::Error)
    }
}
impl R {
    #[doc = "Bit 0 - Overrun Error Flag"]
    #[inline(always)]
    pub fn orun(&self) -> OrunR {
        OrunR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Underrun Error Flag"]
    #[inline(always)]
    pub fn urun(&self) -> UrunR {
        UrunR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Underrun and Not Acknowledged (NACKed) Error Flag"]
    #[inline(always)]
    pub fn urunnack(&self) -> UrunnackR {
        UrunnackR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Terminated Error Flag"]
    #[inline(always)]
    pub fn term(&self) -> TermR {
        TermR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Invalid Start Error Flag"]
    #[inline(always)]
    pub fn invstart(&self) -> InvstartR {
        InvstartR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 8 - SDR Parity Error Flag"]
    #[inline(always)]
    pub fn spar(&self) -> SparR {
        SparR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - HDR Parity Error Flag"]
    #[inline(always)]
    pub fn hpar(&self) -> HparR {
        HparR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - HDR-DDR CRC Error Flag"]
    #[inline(always)]
    pub fn hcrc(&self) -> HcrcR {
        HcrcR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - TE0 or TE1 Error Flag"]
    #[inline(always)]
    pub fn s0s1(&self) -> S0s1R {
        S0s1R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 16 - Over-Read Error Flag"]
    #[inline(always)]
    pub fn oread(&self) -> OreadR {
        OreadR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Over-Write Error Flag"]
    #[inline(always)]
    pub fn owrite(&self) -> OwriteR {
        OwriteR::new(((self.bits >> 17) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Overrun Error Flag"]
    #[inline(always)]
    pub fn orun(&mut self) -> OrunW<SerrwarnSpec> {
        OrunW::new(self, 0)
    }
    #[doc = "Bit 1 - Underrun Error Flag"]
    #[inline(always)]
    pub fn urun(&mut self) -> UrunW<SerrwarnSpec> {
        UrunW::new(self, 1)
    }
    #[doc = "Bit 2 - Underrun and Not Acknowledged (NACKed) Error Flag"]
    #[inline(always)]
    pub fn urunnack(&mut self) -> UrunnackW<SerrwarnSpec> {
        UrunnackW::new(self, 2)
    }
    #[doc = "Bit 3 - Terminated Error Flag"]
    #[inline(always)]
    pub fn term(&mut self) -> TermW<SerrwarnSpec> {
        TermW::new(self, 3)
    }
    #[doc = "Bit 4 - Invalid Start Error Flag"]
    #[inline(always)]
    pub fn invstart(&mut self) -> InvstartW<SerrwarnSpec> {
        InvstartW::new(self, 4)
    }
    #[doc = "Bit 8 - SDR Parity Error Flag"]
    #[inline(always)]
    pub fn spar(&mut self) -> SparW<SerrwarnSpec> {
        SparW::new(self, 8)
    }
    #[doc = "Bit 9 - HDR Parity Error Flag"]
    #[inline(always)]
    pub fn hpar(&mut self) -> HparW<SerrwarnSpec> {
        HparW::new(self, 9)
    }
    #[doc = "Bit 10 - HDR-DDR CRC Error Flag"]
    #[inline(always)]
    pub fn hcrc(&mut self) -> HcrcW<SerrwarnSpec> {
        HcrcW::new(self, 10)
    }
    #[doc = "Bit 11 - TE0 or TE1 Error Flag"]
    #[inline(always)]
    pub fn s0s1(&mut self) -> S0s1W<SerrwarnSpec> {
        S0s1W::new(self, 11)
    }
    #[doc = "Bit 16 - Over-Read Error Flag"]
    #[inline(always)]
    pub fn oread(&mut self) -> OreadW<SerrwarnSpec> {
        OreadW::new(self, 16)
    }
    #[doc = "Bit 17 - Over-Write Error Flag"]
    #[inline(always)]
    pub fn owrite(&mut self) -> OwriteW<SerrwarnSpec> {
        OwriteW::new(self, 17)
    }
}
#[doc = "Target Errors and Warnings\n\nYou can [`read`](crate::Reg::read) this register and get [`serrwarn::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`serrwarn::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SerrwarnSpec;
impl crate::RegisterSpec for SerrwarnSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`serrwarn::R`](R) reader structure"]
impl crate::Readable for SerrwarnSpec {}
#[doc = "`write(|w| ..)` method takes [`serrwarn::W`](W) writer structure"]
impl crate::Writable for SerrwarnSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0003_0f1f;
}
#[doc = "`reset()` method sets SERRWARN to value 0"]
impl crate::Resettable for SerrwarnSpec {}

#[doc = "Register `MERRWARN` reader"]
pub type R = crate::R<MerrwarnSpec>;
#[doc = "Register `MERRWARN` writer"]
pub type W = crate::W<MerrwarnSpec>;
#[doc = "Underrun Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Urun {
    #[doc = "0: No error"]
    NoError = 0,
    #[doc = "1: Error"]
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
    #[doc = "No error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Urun::NoError
    }
    #[doc = "Error"]
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
    #[doc = "No error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Urun::NoError)
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Urun::Error)
    }
}
#[doc = "Not Acknowledge Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nack {
    #[doc = "0: No error"]
    NoError = 0,
    #[doc = "1: Error"]
    Error = 1,
}
impl From<Nack> for bool {
    #[inline(always)]
    fn from(variant: Nack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NACK` reader - Not Acknowledge Error Flag"]
pub type NackR = crate::BitReader<Nack>;
impl NackR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nack {
        match self.bits {
            false => Nack::NoError,
            true => Nack::Error,
        }
    }
    #[doc = "No error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Nack::NoError
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Nack::Error
    }
}
#[doc = "Field `NACK` writer - Not Acknowledge Error Flag"]
pub type NackW<'a, REG> = crate::BitWriter1C<'a, REG, Nack>;
impl<'a, REG> NackW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Nack::NoError)
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Nack::Error)
    }
}
#[doc = "Write Abort Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wrabt {
    #[doc = "0: No error"]
    NoError = 0,
    #[doc = "1: Error"]
    Error = 1,
}
impl From<Wrabt> for bool {
    #[inline(always)]
    fn from(variant: Wrabt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WRABT` reader - Write Abort Error Flag"]
pub type WrabtR = crate::BitReader<Wrabt>;
impl WrabtR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wrabt {
        match self.bits {
            false => Wrabt::NoError,
            true => Wrabt::Error,
        }
    }
    #[doc = "No error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Wrabt::NoError
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Wrabt::Error
    }
}
#[doc = "Field `WRABT` writer - Write Abort Error Flag"]
pub type WrabtW<'a, REG> = crate::BitWriter1C<'a, REG, Wrabt>;
impl<'a, REG> WrabtW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Wrabt::NoError)
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Wrabt::Error)
    }
}
#[doc = "Terminate Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Term {
    #[doc = "0: No error"]
    NoError = 0,
    #[doc = "1: Error"]
    Error = 1,
}
impl From<Term> for bool {
    #[inline(always)]
    fn from(variant: Term) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TERM` reader - Terminate Error Flag"]
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
    #[doc = "No error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Term::NoError
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Term::Error
    }
}
#[doc = "Field `TERM` writer - Terminate Error Flag"]
pub type TermW<'a, REG> = crate::BitWriter1C<'a, REG, Term>;
impl<'a, REG> TermW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Term::NoError)
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Term::Error)
    }
}
#[doc = "High Data Rate Parity Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hpar {
    #[doc = "0: No error"]
    NoError = 0,
    #[doc = "1: Error"]
    Error = 1,
}
impl From<Hpar> for bool {
    #[inline(always)]
    fn from(variant: Hpar) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HPAR` reader - High Data Rate Parity Flag"]
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
    #[doc = "No error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Hpar::NoError
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Hpar::Error
    }
}
#[doc = "Field `HPAR` writer - High Data Rate Parity Flag"]
pub type HparW<'a, REG> = crate::BitWriter1C<'a, REG, Hpar>;
impl<'a, REG> HparW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Hpar::NoError)
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Hpar::Error)
    }
}
#[doc = "High Data Rate CRC Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hcrc {
    #[doc = "0: No error"]
    NoError = 0,
    #[doc = "1: Error"]
    Error = 1,
}
impl From<Hcrc> for bool {
    #[inline(always)]
    fn from(variant: Hcrc) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HCRC` reader - High Data Rate CRC Error Flag"]
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
    #[doc = "No error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Hcrc::NoError
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Hcrc::Error
    }
}
#[doc = "Field `HCRC` writer - High Data Rate CRC Error Flag"]
pub type HcrcW<'a, REG> = crate::BitWriter1C<'a, REG, Hcrc>;
impl<'a, REG> HcrcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Hcrc::NoError)
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Hcrc::Error)
    }
}
#[doc = "Overread Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Oread {
    #[doc = "0: No error"]
    NoError = 0,
    #[doc = "1: Error"]
    Error = 1,
}
impl From<Oread> for bool {
    #[inline(always)]
    fn from(variant: Oread) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OREAD` reader - Overread Error Flag"]
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
    #[doc = "No error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Oread::NoError
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Oread::Error
    }
}
#[doc = "Field `OREAD` writer - Overread Error Flag"]
pub type OreadW<'a, REG> = crate::BitWriter1C<'a, REG, Oread>;
impl<'a, REG> OreadW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Oread::NoError)
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Oread::Error)
    }
}
#[doc = "Overwrite Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owrite {
    #[doc = "0: No error"]
    NoError = 0,
    #[doc = "1: Error"]
    Error = 1,
}
impl From<Owrite> for bool {
    #[inline(always)]
    fn from(variant: Owrite) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OWRITE` reader - Overwrite Error Flag"]
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
    #[doc = "No error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Owrite::NoError
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Owrite::Error
    }
}
#[doc = "Field `OWRITE` writer - Overwrite Error Flag"]
pub type OwriteW<'a, REG> = crate::BitWriter1C<'a, REG, Owrite>;
impl<'a, REG> OwriteW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Owrite::NoError)
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Owrite::Error)
    }
}
#[doc = "Message Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Msgerr {
    #[doc = "0: No error"]
    NoError = 0,
    #[doc = "1: Error"]
    Error = 1,
}
impl From<Msgerr> for bool {
    #[inline(always)]
    fn from(variant: Msgerr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MSGERR` reader - Message Error Flag"]
pub type MsgerrR = crate::BitReader<Msgerr>;
impl MsgerrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Msgerr {
        match self.bits {
            false => Msgerr::NoError,
            true => Msgerr::Error,
        }
    }
    #[doc = "No error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Msgerr::NoError
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Msgerr::Error
    }
}
#[doc = "Field `MSGERR` writer - Message Error Flag"]
pub type MsgerrW<'a, REG> = crate::BitWriter1C<'a, REG, Msgerr>;
impl<'a, REG> MsgerrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Msgerr::NoError)
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Msgerr::Error)
    }
}
#[doc = "Invalid Request Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invreq {
    #[doc = "0: No error"]
    NoError = 0,
    #[doc = "1: Error"]
    Error = 1,
}
impl From<Invreq> for bool {
    #[inline(always)]
    fn from(variant: Invreq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INVREQ` reader - Invalid Request Error Flag"]
pub type InvreqR = crate::BitReader<Invreq>;
impl InvreqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Invreq {
        match self.bits {
            false => Invreq::NoError,
            true => Invreq::Error,
        }
    }
    #[doc = "No error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Invreq::NoError
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Invreq::Error
    }
}
#[doc = "Field `INVREQ` writer - Invalid Request Error Flag"]
pub type InvreqW<'a, REG> = crate::BitWriter1C<'a, REG, Invreq>;
impl<'a, REG> InvreqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Invreq::NoError)
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Invreq::Error)
    }
}
#[doc = "Timeout Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timeout {
    #[doc = "0: No error"]
    NoError = 0,
    #[doc = "1: Error"]
    Error = 1,
}
impl From<Timeout> for bool {
    #[inline(always)]
    fn from(variant: Timeout) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIMEOUT` reader - Timeout Error Flag"]
pub type TimeoutR = crate::BitReader<Timeout>;
impl TimeoutR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Timeout {
        match self.bits {
            false => Timeout::NoError,
            true => Timeout::Error,
        }
    }
    #[doc = "No error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Timeout::NoError
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Timeout::Error
    }
}
#[doc = "Field `TIMEOUT` writer - Timeout Error Flag"]
pub type TimeoutW<'a, REG> = crate::BitWriter1C<'a, REG, Timeout>;
impl<'a, REG> TimeoutW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No error"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Timeout::NoError)
    }
    #[doc = "Error"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Timeout::Error)
    }
}
impl R {
    #[doc = "Bit 1 - Underrun Error Flag"]
    #[inline(always)]
    pub fn urun(&self) -> UrunR {
        UrunR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Not Acknowledge Error Flag"]
    #[inline(always)]
    pub fn nack(&self) -> NackR {
        NackR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Write Abort Error Flag"]
    #[inline(always)]
    pub fn wrabt(&self) -> WrabtR {
        WrabtR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Terminate Error Flag"]
    #[inline(always)]
    pub fn term(&self) -> TermR {
        TermR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 9 - High Data Rate Parity Flag"]
    #[inline(always)]
    pub fn hpar(&self) -> HparR {
        HparR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - High Data Rate CRC Error Flag"]
    #[inline(always)]
    pub fn hcrc(&self) -> HcrcR {
        HcrcR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 16 - Overread Error Flag"]
    #[inline(always)]
    pub fn oread(&self) -> OreadR {
        OreadR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Overwrite Error Flag"]
    #[inline(always)]
    pub fn owrite(&self) -> OwriteR {
        OwriteR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Message Error Flag"]
    #[inline(always)]
    pub fn msgerr(&self) -> MsgerrR {
        MsgerrR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Invalid Request Error Flag"]
    #[inline(always)]
    pub fn invreq(&self) -> InvreqR {
        InvreqR::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Timeout Error Flag"]
    #[inline(always)]
    pub fn timeout(&self) -> TimeoutR {
        TimeoutR::new(((self.bits >> 20) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - Underrun Error Flag"]
    #[inline(always)]
    pub fn urun(&mut self) -> UrunW<MerrwarnSpec> {
        UrunW::new(self, 1)
    }
    #[doc = "Bit 2 - Not Acknowledge Error Flag"]
    #[inline(always)]
    pub fn nack(&mut self) -> NackW<MerrwarnSpec> {
        NackW::new(self, 2)
    }
    #[doc = "Bit 3 - Write Abort Error Flag"]
    #[inline(always)]
    pub fn wrabt(&mut self) -> WrabtW<MerrwarnSpec> {
        WrabtW::new(self, 3)
    }
    #[doc = "Bit 4 - Terminate Error Flag"]
    #[inline(always)]
    pub fn term(&mut self) -> TermW<MerrwarnSpec> {
        TermW::new(self, 4)
    }
    #[doc = "Bit 9 - High Data Rate Parity Flag"]
    #[inline(always)]
    pub fn hpar(&mut self) -> HparW<MerrwarnSpec> {
        HparW::new(self, 9)
    }
    #[doc = "Bit 10 - High Data Rate CRC Error Flag"]
    #[inline(always)]
    pub fn hcrc(&mut self) -> HcrcW<MerrwarnSpec> {
        HcrcW::new(self, 10)
    }
    #[doc = "Bit 16 - Overread Error Flag"]
    #[inline(always)]
    pub fn oread(&mut self) -> OreadW<MerrwarnSpec> {
        OreadW::new(self, 16)
    }
    #[doc = "Bit 17 - Overwrite Error Flag"]
    #[inline(always)]
    pub fn owrite(&mut self) -> OwriteW<MerrwarnSpec> {
        OwriteW::new(self, 17)
    }
    #[doc = "Bit 18 - Message Error Flag"]
    #[inline(always)]
    pub fn msgerr(&mut self) -> MsgerrW<MerrwarnSpec> {
        MsgerrW::new(self, 18)
    }
    #[doc = "Bit 19 - Invalid Request Error Flag"]
    #[inline(always)]
    pub fn invreq(&mut self) -> InvreqW<MerrwarnSpec> {
        InvreqW::new(self, 19)
    }
    #[doc = "Bit 20 - Timeout Error Flag"]
    #[inline(always)]
    pub fn timeout(&mut self) -> TimeoutW<MerrwarnSpec> {
        TimeoutW::new(self, 20)
    }
}
#[doc = "Controller Errors and Warnings\n\nYou can [`read`](crate::Reg::read) this register and get [`merrwarn::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`merrwarn::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MerrwarnSpec;
impl crate::RegisterSpec for MerrwarnSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`merrwarn::R`](R) reader structure"]
impl crate::Readable for MerrwarnSpec {}
#[doc = "`write(|w| ..)` method takes [`merrwarn::W`](W) writer structure"]
impl crate::Writable for MerrwarnSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x001f_061e;
}
#[doc = "`reset()` method sets MERRWARN to value 0"]
impl crate::Resettable for MerrwarnSpec {}

#[doc = "Register `SR` reader"]
pub type R = crate::R<SrSpec>;
#[doc = "Register `SR` writer"]
pub type W = crate::W<SrSpec>;
#[doc = "Transmit Data Flag\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdf {
    #[doc = "0: Transmit data not requested"]
    TxdataNotReqst = 0,
    #[doc = "1: Transmit data requested"]
    TxdataReqst = 1,
}
impl From<Tdf> for bool {
    #[inline(always)]
    fn from(variant: Tdf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TDF` reader - Transmit Data Flag"]
pub type TdfR = crate::BitReader<Tdf>;
impl TdfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tdf {
        match self.bits {
            false => Tdf::TxdataNotReqst,
            true => Tdf::TxdataReqst,
        }
    }
    #[doc = "Transmit data not requested"]
    #[inline(always)]
    pub fn is_txdata_not_reqst(&self) -> bool {
        *self == Tdf::TxdataNotReqst
    }
    #[doc = "Transmit data requested"]
    #[inline(always)]
    pub fn is_txdata_reqst(&self) -> bool {
        *self == Tdf::TxdataReqst
    }
}
#[doc = "Receive Data Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdf {
    #[doc = "0: Receive data not ready"]
    Notready = 0,
    #[doc = "1: Receive data ready"]
    Ready = 1,
}
impl From<Rdf> for bool {
    #[inline(always)]
    fn from(variant: Rdf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RDF` reader - Receive Data Flag"]
pub type RdfR = crate::BitReader<Rdf>;
impl RdfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rdf {
        match self.bits {
            false => Rdf::Notready,
            true => Rdf::Ready,
        }
    }
    #[doc = "Receive data not ready"]
    #[inline(always)]
    pub fn is_notready(&self) -> bool {
        *self == Rdf::Notready
    }
    #[doc = "Receive data ready"]
    #[inline(always)]
    pub fn is_ready(&self) -> bool {
        *self == Rdf::Ready
    }
}
#[doc = "Word Complete Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wcf {
    #[doc = "0: Not complete"]
    NotCompleted = 0,
    #[doc = "1: Complete"]
    Completed = 1,
}
impl From<Wcf> for bool {
    #[inline(always)]
    fn from(variant: Wcf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WCF` reader - Word Complete Flag"]
pub type WcfR = crate::BitReader<Wcf>;
impl WcfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wcf {
        match self.bits {
            false => Wcf::NotCompleted,
            true => Wcf::Completed,
        }
    }
    #[doc = "Not complete"]
    #[inline(always)]
    pub fn is_not_completed(&self) -> bool {
        *self == Wcf::NotCompleted
    }
    #[doc = "Complete"]
    #[inline(always)]
    pub fn is_completed(&self) -> bool {
        *self == Wcf::Completed
    }
}
#[doc = "Field `WCF` writer - Word Complete Flag"]
pub type WcfW<'a, REG> = crate::BitWriter1C<'a, REG, Wcf>;
impl<'a, REG> WcfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not complete"]
    #[inline(always)]
    pub fn not_completed(self) -> &'a mut crate::W<REG> {
        self.variant(Wcf::NotCompleted)
    }
    #[doc = "Complete"]
    #[inline(always)]
    pub fn completed(self) -> &'a mut crate::W<REG> {
        self.variant(Wcf::Completed)
    }
}
#[doc = "Frame Complete Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fcf {
    #[doc = "0: Not complete"]
    NotCompleted = 0,
    #[doc = "1: Complete"]
    Completed = 1,
}
impl From<Fcf> for bool {
    #[inline(always)]
    fn from(variant: Fcf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FCF` reader - Frame Complete Flag"]
pub type FcfR = crate::BitReader<Fcf>;
impl FcfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fcf {
        match self.bits {
            false => Fcf::NotCompleted,
            true => Fcf::Completed,
        }
    }
    #[doc = "Not complete"]
    #[inline(always)]
    pub fn is_not_completed(&self) -> bool {
        *self == Fcf::NotCompleted
    }
    #[doc = "Complete"]
    #[inline(always)]
    pub fn is_completed(&self) -> bool {
        *self == Fcf::Completed
    }
}
#[doc = "Field `FCF` writer - Frame Complete Flag"]
pub type FcfW<'a, REG> = crate::BitWriter1C<'a, REG, Fcf>;
impl<'a, REG> FcfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not complete"]
    #[inline(always)]
    pub fn not_completed(self) -> &'a mut crate::W<REG> {
        self.variant(Fcf::NotCompleted)
    }
    #[doc = "Complete"]
    #[inline(always)]
    pub fn completed(self) -> &'a mut crate::W<REG> {
        self.variant(Fcf::Completed)
    }
}
#[doc = "Transfer Complete Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tcf {
    #[doc = "0: Not complete"]
    NotCompleted = 0,
    #[doc = "1: Complete"]
    Completed = 1,
}
impl From<Tcf> for bool {
    #[inline(always)]
    fn from(variant: Tcf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TCF` reader - Transfer Complete Flag"]
pub type TcfR = crate::BitReader<Tcf>;
impl TcfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tcf {
        match self.bits {
            false => Tcf::NotCompleted,
            true => Tcf::Completed,
        }
    }
    #[doc = "Not complete"]
    #[inline(always)]
    pub fn is_not_completed(&self) -> bool {
        *self == Tcf::NotCompleted
    }
    #[doc = "Complete"]
    #[inline(always)]
    pub fn is_completed(&self) -> bool {
        *self == Tcf::Completed
    }
}
#[doc = "Field `TCF` writer - Transfer Complete Flag"]
pub type TcfW<'a, REG> = crate::BitWriter1C<'a, REG, Tcf>;
impl<'a, REG> TcfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not complete"]
    #[inline(always)]
    pub fn not_completed(self) -> &'a mut crate::W<REG> {
        self.variant(Tcf::NotCompleted)
    }
    #[doc = "Complete"]
    #[inline(always)]
    pub fn completed(self) -> &'a mut crate::W<REG> {
        self.variant(Tcf::Completed)
    }
}
#[doc = "Transmit Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tef {
    #[doc = "0: No underrun"]
    NoUnderrun = 0,
    #[doc = "1: Underrun"]
    Underrun = 1,
}
impl From<Tef> for bool {
    #[inline(always)]
    fn from(variant: Tef) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TEF` reader - Transmit Error Flag"]
pub type TefR = crate::BitReader<Tef>;
impl TefR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tef {
        match self.bits {
            false => Tef::NoUnderrun,
            true => Tef::Underrun,
        }
    }
    #[doc = "No underrun"]
    #[inline(always)]
    pub fn is_no_underrun(&self) -> bool {
        *self == Tef::NoUnderrun
    }
    #[doc = "Underrun"]
    #[inline(always)]
    pub fn is_underrun(&self) -> bool {
        *self == Tef::Underrun
    }
}
#[doc = "Field `TEF` writer - Transmit Error Flag"]
pub type TefW<'a, REG> = crate::BitWriter1C<'a, REG, Tef>;
impl<'a, REG> TefW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No underrun"]
    #[inline(always)]
    pub fn no_underrun(self) -> &'a mut crate::W<REG> {
        self.variant(Tef::NoUnderrun)
    }
    #[doc = "Underrun"]
    #[inline(always)]
    pub fn underrun(self) -> &'a mut crate::W<REG> {
        self.variant(Tef::Underrun)
    }
}
#[doc = "Receive Error Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ref {
    #[doc = "0: No overflow"]
    NotOverflowed = 0,
    #[doc = "1: Overflow"]
    Overflowed = 1,
}
impl From<Ref> for bool {
    #[inline(always)]
    fn from(variant: Ref) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `REF` reader - Receive Error Flag"]
pub type RefR = crate::BitReader<Ref>;
impl RefR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ref {
        match self.bits {
            false => Ref::NotOverflowed,
            true => Ref::Overflowed,
        }
    }
    #[doc = "No overflow"]
    #[inline(always)]
    pub fn is_not_overflowed(&self) -> bool {
        *self == Ref::NotOverflowed
    }
    #[doc = "Overflow"]
    #[inline(always)]
    pub fn is_overflowed(&self) -> bool {
        *self == Ref::Overflowed
    }
}
#[doc = "Field `REF` writer - Receive Error Flag"]
pub type RefW<'a, REG> = crate::BitWriter1C<'a, REG, Ref>;
impl<'a, REG> RefW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No overflow"]
    #[inline(always)]
    pub fn not_overflowed(self) -> &'a mut crate::W<REG> {
        self.variant(Ref::NotOverflowed)
    }
    #[doc = "Overflow"]
    #[inline(always)]
    pub fn overflowed(self) -> &'a mut crate::W<REG> {
        self.variant(Ref::Overflowed)
    }
}
#[doc = "Data Match Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dmf {
    #[doc = "0: No match"]
    NoMatch = 0,
    #[doc = "1: Match"]
    Match = 1,
}
impl From<Dmf> for bool {
    #[inline(always)]
    fn from(variant: Dmf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DMF` reader - Data Match Flag"]
pub type DmfR = crate::BitReader<Dmf>;
impl DmfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dmf {
        match self.bits {
            false => Dmf::NoMatch,
            true => Dmf::Match,
        }
    }
    #[doc = "No match"]
    #[inline(always)]
    pub fn is_no_match(&self) -> bool {
        *self == Dmf::NoMatch
    }
    #[doc = "Match"]
    #[inline(always)]
    pub fn is_match(&self) -> bool {
        *self == Dmf::Match
    }
}
#[doc = "Field `DMF` writer - Data Match Flag"]
pub type DmfW<'a, REG> = crate::BitWriter1C<'a, REG, Dmf>;
impl<'a, REG> DmfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No match"]
    #[inline(always)]
    pub fn no_match(self) -> &'a mut crate::W<REG> {
        self.variant(Dmf::NoMatch)
    }
    #[doc = "Match"]
    #[inline(always)]
    pub fn match_(self) -> &'a mut crate::W<REG> {
        self.variant(Dmf::Match)
    }
}
#[doc = "Module Busy Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mbf {
    #[doc = "0: LPSPI is idle"]
    Idle = 0,
    #[doc = "1: LPSPI is busy"]
    Busy = 1,
}
impl From<Mbf> for bool {
    #[inline(always)]
    fn from(variant: Mbf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MBF` reader - Module Busy Flag"]
pub type MbfR = crate::BitReader<Mbf>;
impl MbfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mbf {
        match self.bits {
            false => Mbf::Idle,
            true => Mbf::Busy,
        }
    }
    #[doc = "LPSPI is idle"]
    #[inline(always)]
    pub fn is_idle(&self) -> bool {
        *self == Mbf::Idle
    }
    #[doc = "LPSPI is busy"]
    #[inline(always)]
    pub fn is_busy(&self) -> bool {
        *self == Mbf::Busy
    }
}
impl R {
    #[doc = "Bit 0 - Transmit Data Flag"]
    #[inline(always)]
    pub fn tdf(&self) -> TdfR {
        TdfR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Receive Data Flag"]
    #[inline(always)]
    pub fn rdf(&self) -> RdfR {
        RdfR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 8 - Word Complete Flag"]
    #[inline(always)]
    pub fn wcf(&self) -> WcfR {
        WcfR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Frame Complete Flag"]
    #[inline(always)]
    pub fn fcf(&self) -> FcfR {
        FcfR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Transfer Complete Flag"]
    #[inline(always)]
    pub fn tcf(&self) -> TcfR {
        TcfR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Transmit Error Flag"]
    #[inline(always)]
    pub fn tef(&self) -> TefR {
        TefR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Receive Error Flag"]
    #[inline(always)]
    pub fn ref_(&self) -> RefR {
        RefR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Data Match Flag"]
    #[inline(always)]
    pub fn dmf(&self) -> DmfR {
        DmfR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 24 - Module Busy Flag"]
    #[inline(always)]
    pub fn mbf(&self) -> MbfR {
        MbfR::new(((self.bits >> 24) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 8 - Word Complete Flag"]
    #[inline(always)]
    pub fn wcf(&mut self) -> WcfW<SrSpec> {
        WcfW::new(self, 8)
    }
    #[doc = "Bit 9 - Frame Complete Flag"]
    #[inline(always)]
    pub fn fcf(&mut self) -> FcfW<SrSpec> {
        FcfW::new(self, 9)
    }
    #[doc = "Bit 10 - Transfer Complete Flag"]
    #[inline(always)]
    pub fn tcf(&mut self) -> TcfW<SrSpec> {
        TcfW::new(self, 10)
    }
    #[doc = "Bit 11 - Transmit Error Flag"]
    #[inline(always)]
    pub fn tef(&mut self) -> TefW<SrSpec> {
        TefW::new(self, 11)
    }
    #[doc = "Bit 12 - Receive Error Flag"]
    #[inline(always)]
    pub fn ref_(&mut self) -> RefW<SrSpec> {
        RefW::new(self, 12)
    }
    #[doc = "Bit 13 - Data Match Flag"]
    #[inline(always)]
    pub fn dmf(&mut self) -> DmfW<SrSpec> {
        DmfW::new(self, 13)
    }
}
#[doc = "Status\n\nYou can [`read`](crate::Reg::read) this register and get [`sr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SrSpec;
impl crate::RegisterSpec for SrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sr::R`](R) reader structure"]
impl crate::Readable for SrSpec {}
#[doc = "`write(|w| ..)` method takes [`sr::W`](W) writer structure"]
impl crate::Writable for SrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x3f00;
}
#[doc = "`reset()` method sets SR to value 0x01"]
impl crate::Resettable for SrSpec {
    const RESET_VALUE: u32 = 0x01;
}

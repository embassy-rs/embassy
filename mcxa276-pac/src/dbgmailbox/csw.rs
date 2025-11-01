#[doc = "Register `CSW` reader"]
pub type R = crate::R<CswSpec>;
#[doc = "Register `CSW` writer"]
pub type W = crate::W<CswSpec>;
#[doc = "Resynchronization Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResynchReq {
    #[doc = "0: No request"]
    NoRequest = 0,
    #[doc = "1: Request for resynchronization"]
    Request = 1,
}
impl From<ResynchReq> for bool {
    #[inline(always)]
    fn from(variant: ResynchReq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RESYNCH_REQ` reader - Resynchronization Request"]
pub type ResynchReqR = crate::BitReader<ResynchReq>;
impl ResynchReqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> ResynchReq {
        match self.bits {
            false => ResynchReq::NoRequest,
            true => ResynchReq::Request,
        }
    }
    #[doc = "No request"]
    #[inline(always)]
    pub fn is_no_request(&self) -> bool {
        *self == ResynchReq::NoRequest
    }
    #[doc = "Request for resynchronization"]
    #[inline(always)]
    pub fn is_request(&self) -> bool {
        *self == ResynchReq::Request
    }
}
#[doc = "Field `RESYNCH_REQ` writer - Resynchronization Request"]
pub type ResynchReqW<'a, REG> = crate::BitWriter<'a, REG, ResynchReq>;
impl<'a, REG> ResynchReqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No request"]
    #[inline(always)]
    pub fn no_request(self) -> &'a mut crate::W<REG> {
        self.variant(ResynchReq::NoRequest)
    }
    #[doc = "Request for resynchronization"]
    #[inline(always)]
    pub fn request(self) -> &'a mut crate::W<REG> {
        self.variant(ResynchReq::Request)
    }
}
#[doc = "Request Pending\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReqPending {
    #[doc = "0: No request pending"]
    NoRequestPending = 0,
    #[doc = "1: Request for resynchronization pending"]
    RequestPending = 1,
}
impl From<ReqPending> for bool {
    #[inline(always)]
    fn from(variant: ReqPending) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `REQ_PENDING` reader - Request Pending"]
pub type ReqPendingR = crate::BitReader<ReqPending>;
impl ReqPendingR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> ReqPending {
        match self.bits {
            false => ReqPending::NoRequestPending,
            true => ReqPending::RequestPending,
        }
    }
    #[doc = "No request pending"]
    #[inline(always)]
    pub fn is_no_request_pending(&self) -> bool {
        *self == ReqPending::NoRequestPending
    }
    #[doc = "Request for resynchronization pending"]
    #[inline(always)]
    pub fn is_request_pending(&self) -> bool {
        *self == ReqPending::RequestPending
    }
}
#[doc = "Field `REQ_PENDING` writer - Request Pending"]
pub type ReqPendingW<'a, REG> = crate::BitWriter<'a, REG, ReqPending>;
impl<'a, REG> ReqPendingW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No request pending"]
    #[inline(always)]
    pub fn no_request_pending(self) -> &'a mut crate::W<REG> {
        self.variant(ReqPending::NoRequestPending)
    }
    #[doc = "Request for resynchronization pending"]
    #[inline(always)]
    pub fn request_pending(self) -> &'a mut crate::W<REG> {
        self.variant(ReqPending::RequestPending)
    }
}
#[doc = "DBGMB Overrun Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DbgOrErr {
    #[doc = "0: No overrun"]
    NoOverrunErr = 0,
    #[doc = "1: Overrun occurred"]
    OverrunErr = 1,
}
impl From<DbgOrErr> for bool {
    #[inline(always)]
    fn from(variant: DbgOrErr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DBG_OR_ERR` reader - DBGMB Overrun Error"]
pub type DbgOrErrR = crate::BitReader<DbgOrErr>;
impl DbgOrErrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> DbgOrErr {
        match self.bits {
            false => DbgOrErr::NoOverrunErr,
            true => DbgOrErr::OverrunErr,
        }
    }
    #[doc = "No overrun"]
    #[inline(always)]
    pub fn is_no_overrun_err(&self) -> bool {
        *self == DbgOrErr::NoOverrunErr
    }
    #[doc = "Overrun occurred"]
    #[inline(always)]
    pub fn is_overrun_err(&self) -> bool {
        *self == DbgOrErr::OverrunErr
    }
}
#[doc = "Field `DBG_OR_ERR` writer - DBGMB Overrun Error"]
pub type DbgOrErrW<'a, REG> = crate::BitWriter<'a, REG, DbgOrErr>;
impl<'a, REG> DbgOrErrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No overrun"]
    #[inline(always)]
    pub fn no_overrun_err(self) -> &'a mut crate::W<REG> {
        self.variant(DbgOrErr::NoOverrunErr)
    }
    #[doc = "Overrun occurred"]
    #[inline(always)]
    pub fn overrun_err(self) -> &'a mut crate::W<REG> {
        self.variant(DbgOrErr::OverrunErr)
    }
}
#[doc = "AHB Overrun Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AhbOrErr {
    #[doc = "0: No overrun"]
    NoAhbOverrunErr = 0,
    #[doc = "1: Overrun occurred"]
    AhbOverrunErr = 1,
}
impl From<AhbOrErr> for bool {
    #[inline(always)]
    fn from(variant: AhbOrErr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `AHB_OR_ERR` reader - AHB Overrun Error"]
pub type AhbOrErrR = crate::BitReader<AhbOrErr>;
impl AhbOrErrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> AhbOrErr {
        match self.bits {
            false => AhbOrErr::NoAhbOverrunErr,
            true => AhbOrErr::AhbOverrunErr,
        }
    }
    #[doc = "No overrun"]
    #[inline(always)]
    pub fn is_no_ahb_overrun_err(&self) -> bool {
        *self == AhbOrErr::NoAhbOverrunErr
    }
    #[doc = "Overrun occurred"]
    #[inline(always)]
    pub fn is_ahb_overrun_err(&self) -> bool {
        *self == AhbOrErr::AhbOverrunErr
    }
}
#[doc = "Field `AHB_OR_ERR` writer - AHB Overrun Error"]
pub type AhbOrErrW<'a, REG> = crate::BitWriter<'a, REG, AhbOrErr>;
impl<'a, REG> AhbOrErrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No overrun"]
    #[inline(always)]
    pub fn no_ahb_overrun_err(self) -> &'a mut crate::W<REG> {
        self.variant(AhbOrErr::NoAhbOverrunErr)
    }
    #[doc = "Overrun occurred"]
    #[inline(always)]
    pub fn ahb_overrun_err(self) -> &'a mut crate::W<REG> {
        self.variant(AhbOrErr::AhbOverrunErr)
    }
}
#[doc = "Soft Reset\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoftReset {
    #[doc = "0: No effect"]
    NoEff = 0,
    #[doc = "1: Reset"]
    Reset = 1,
}
impl From<SoftReset> for bool {
    #[inline(always)]
    fn from(variant: SoftReset) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOFT_RESET` reader - Soft Reset"]
pub type SoftResetR = crate::BitReader<SoftReset>;
impl SoftResetR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SoftReset {
        match self.bits {
            false => SoftReset::NoEff,
            true => SoftReset::Reset,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_no_eff(&self) -> bool {
        *self == SoftReset::NoEff
    }
    #[doc = "Reset"]
    #[inline(always)]
    pub fn is_reset(&self) -> bool {
        *self == SoftReset::Reset
    }
}
#[doc = "Field `SOFT_RESET` writer - Soft Reset"]
pub type SoftResetW<'a, REG> = crate::BitWriter<'a, REG, SoftReset>;
impl<'a, REG> SoftResetW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn no_eff(self) -> &'a mut crate::W<REG> {
        self.variant(SoftReset::NoEff)
    }
    #[doc = "Reset"]
    #[inline(always)]
    pub fn reset(self) -> &'a mut crate::W<REG> {
        self.variant(SoftReset::Reset)
    }
}
#[doc = "Chip Reset Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChipResetReq {
    #[doc = "0: No effect"]
    NoEff = 0,
    #[doc = "1: Reset"]
    Reset = 1,
}
impl From<ChipResetReq> for bool {
    #[inline(always)]
    fn from(variant: ChipResetReq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CHIP_RESET_REQ` reader - Chip Reset Request"]
pub type ChipResetReqR = crate::BitReader<ChipResetReq>;
impl ChipResetReqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> ChipResetReq {
        match self.bits {
            false => ChipResetReq::NoEff,
            true => ChipResetReq::Reset,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_no_eff(&self) -> bool {
        *self == ChipResetReq::NoEff
    }
    #[doc = "Reset"]
    #[inline(always)]
    pub fn is_reset(&self) -> bool {
        *self == ChipResetReq::Reset
    }
}
#[doc = "Field `CHIP_RESET_REQ` writer - Chip Reset Request"]
pub type ChipResetReqW<'a, REG> = crate::BitWriter<'a, REG, ChipResetReq>;
impl<'a, REG> ChipResetReqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn no_eff(self) -> &'a mut crate::W<REG> {
        self.variant(ChipResetReq::NoEff)
    }
    #[doc = "Reset"]
    #[inline(always)]
    pub fn reset(self) -> &'a mut crate::W<REG> {
        self.variant(ChipResetReq::Reset)
    }
}
impl R {
    #[doc = "Bit 0 - Resynchronization Request"]
    #[inline(always)]
    pub fn resynch_req(&self) -> ResynchReqR {
        ResynchReqR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Request Pending"]
    #[inline(always)]
    pub fn req_pending(&self) -> ReqPendingR {
        ReqPendingR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - DBGMB Overrun Error"]
    #[inline(always)]
    pub fn dbg_or_err(&self) -> DbgOrErrR {
        DbgOrErrR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - AHB Overrun Error"]
    #[inline(always)]
    pub fn ahb_or_err(&self) -> AhbOrErrR {
        AhbOrErrR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Soft Reset"]
    #[inline(always)]
    pub fn soft_reset(&self) -> SoftResetR {
        SoftResetR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Chip Reset Request"]
    #[inline(always)]
    pub fn chip_reset_req(&self) -> ChipResetReqR {
        ChipResetReqR::new(((self.bits >> 5) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Resynchronization Request"]
    #[inline(always)]
    pub fn resynch_req(&mut self) -> ResynchReqW<CswSpec> {
        ResynchReqW::new(self, 0)
    }
    #[doc = "Bit 1 - Request Pending"]
    #[inline(always)]
    pub fn req_pending(&mut self) -> ReqPendingW<CswSpec> {
        ReqPendingW::new(self, 1)
    }
    #[doc = "Bit 2 - DBGMB Overrun Error"]
    #[inline(always)]
    pub fn dbg_or_err(&mut self) -> DbgOrErrW<CswSpec> {
        DbgOrErrW::new(self, 2)
    }
    #[doc = "Bit 3 - AHB Overrun Error"]
    #[inline(always)]
    pub fn ahb_or_err(&mut self) -> AhbOrErrW<CswSpec> {
        AhbOrErrW::new(self, 3)
    }
    #[doc = "Bit 4 - Soft Reset"]
    #[inline(always)]
    pub fn soft_reset(&mut self) -> SoftResetW<CswSpec> {
        SoftResetW::new(self, 4)
    }
    #[doc = "Bit 5 - Chip Reset Request"]
    #[inline(always)]
    pub fn chip_reset_req(&mut self) -> ChipResetReqW<CswSpec> {
        ChipResetReqW::new(self, 5)
    }
}
#[doc = "Command and Status Word\n\nYou can [`read`](crate::Reg::read) this register and get [`csw::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`csw::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CswSpec;
impl crate::RegisterSpec for CswSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`csw::R`](R) reader structure"]
impl crate::Readable for CswSpec {}
#[doc = "`write(|w| ..)` method takes [`csw::W`](W) writer structure"]
impl crate::Writable for CswSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CSW to value 0"]
impl crate::Resettable for CswSpec {}

#[doc = "Register `PD_STATUS0` reader"]
pub type R = crate::R<PdStatus0Spec>;
#[doc = "Register `PD_STATUS0` writer"]
pub type W = crate::W<PdStatus0Spec>;
#[doc = "Power Request Status Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PwrReqStatus {
    #[doc = "0: Did not request"]
    ReqNo = 0,
    #[doc = "1: Requested"]
    ReqYes = 1,
}
impl From<PwrReqStatus> for bool {
    #[inline(always)]
    fn from(variant: PwrReqStatus) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PWR_REQ_STATUS` reader - Power Request Status Flag"]
pub type PwrReqStatusR = crate::BitReader<PwrReqStatus>;
impl PwrReqStatusR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> PwrReqStatus {
        match self.bits {
            false => PwrReqStatus::ReqNo,
            true => PwrReqStatus::ReqYes,
        }
    }
    #[doc = "Did not request"]
    #[inline(always)]
    pub fn is_req_no(&self) -> bool {
        *self == PwrReqStatus::ReqNo
    }
    #[doc = "Requested"]
    #[inline(always)]
    pub fn is_req_yes(&self) -> bool {
        *self == PwrReqStatus::ReqYes
    }
}
#[doc = "Power Domain Low Power Request Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PdLpReq {
    #[doc = "0: Did not request"]
    ReqNo = 0,
    #[doc = "1: Requested"]
    ReqYes = 1,
}
impl From<PdLpReq> for bool {
    #[inline(always)]
    fn from(variant: PdLpReq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PD_LP_REQ` reader - Power Domain Low Power Request Flag"]
pub type PdLpReqR = crate::BitReader<PdLpReq>;
impl PdLpReqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> PdLpReq {
        match self.bits {
            false => PdLpReq::ReqNo,
            true => PdLpReq::ReqYes,
        }
    }
    #[doc = "Did not request"]
    #[inline(always)]
    pub fn is_req_no(&self) -> bool {
        *self == PdLpReq::ReqNo
    }
    #[doc = "Requested"]
    #[inline(always)]
    pub fn is_req_yes(&self) -> bool {
        *self == PdLpReq::ReqYes
    }
}
#[doc = "Field `PD_LP_REQ` writer - Power Domain Low Power Request Flag"]
pub type PdLpReqW<'a, REG> = crate::BitWriter1C<'a, REG, PdLpReq>;
impl<'a, REG> PdLpReqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Did not request"]
    #[inline(always)]
    pub fn req_no(self) -> &'a mut crate::W<REG> {
        self.variant(PdLpReq::ReqNo)
    }
    #[doc = "Requested"]
    #[inline(always)]
    pub fn req_yes(self) -> &'a mut crate::W<REG> {
        self.variant(PdLpReq::ReqYes)
    }
}
#[doc = "Power Domain Low Power Mode Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LpMode {
    #[doc = "0: SLEEP with system clock running"]
    Mode0 = 0,
    #[doc = "1: DSLEEP with system clock off"]
    Mode1 = 1,
    #[doc = "2: PDOWN with system clock off"]
    Mode2 = 2,
    #[doc = "8: DPDOWN with system clock off"]
    Mode8 = 8,
}
impl From<LpMode> for u8 {
    #[inline(always)]
    fn from(variant: LpMode) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for LpMode {
    type Ux = u8;
}
impl crate::IsEnum for LpMode {}
#[doc = "Field `LP_MODE` reader - Power Domain Low Power Mode Request"]
pub type LpModeR = crate::FieldReader<LpMode>;
impl LpModeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<LpMode> {
        match self.bits {
            0 => Some(LpMode::Mode0),
            1 => Some(LpMode::Mode1),
            2 => Some(LpMode::Mode2),
            8 => Some(LpMode::Mode8),
            _ => None,
        }
    }
    #[doc = "SLEEP with system clock running"]
    #[inline(always)]
    pub fn is_mode0(&self) -> bool {
        *self == LpMode::Mode0
    }
    #[doc = "DSLEEP with system clock off"]
    #[inline(always)]
    pub fn is_mode1(&self) -> bool {
        *self == LpMode::Mode1
    }
    #[doc = "PDOWN with system clock off"]
    #[inline(always)]
    pub fn is_mode2(&self) -> bool {
        *self == LpMode::Mode2
    }
    #[doc = "DPDOWN with system clock off"]
    #[inline(always)]
    pub fn is_mode8(&self) -> bool {
        *self == LpMode::Mode8
    }
}
impl R {
    #[doc = "Bit 0 - Power Request Status Flag"]
    #[inline(always)]
    pub fn pwr_req_status(&self) -> PwrReqStatusR {
        PwrReqStatusR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 4 - Power Domain Low Power Request Flag"]
    #[inline(always)]
    pub fn pd_lp_req(&self) -> PdLpReqR {
        PdLpReqR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bits 8:11 - Power Domain Low Power Mode Request"]
    #[inline(always)]
    pub fn lp_mode(&self) -> LpModeR {
        LpModeR::new(((self.bits >> 8) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bit 4 - Power Domain Low Power Request Flag"]
    #[inline(always)]
    pub fn pd_lp_req(&mut self) -> PdLpReqW<PdStatus0Spec> {
        PdLpReqW::new(self, 4)
    }
}
#[doc = "SPC Power Domain Mode Status\n\nYou can [`read`](crate::Reg::read) this register and get [`pd_status0::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pd_status0::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PdStatus0Spec;
impl crate::RegisterSpec for PdStatus0Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pd_status0::R`](R) reader structure"]
impl crate::Readable for PdStatus0Spec {}
#[doc = "`write(|w| ..)` method takes [`pd_status0::W`](W) writer structure"]
impl crate::Writable for PdStatus0Spec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x10;
}
#[doc = "`reset()` method sets PD_STATUS0 to value 0"]
impl crate::Resettable for PdStatus0Spec {}

#[doc = "Register `SC` reader"]
pub type R = crate::R<ScSpec>;
#[doc = "Register `SC` writer"]
pub type W = crate::W<ScSpec>;
#[doc = "SPC Busy Status Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Busy {
    #[doc = "0: Not busy"]
    BusyNo = 0,
    #[doc = "1: Busy"]
    BusyYes = 1,
}
impl From<Busy> for bool {
    #[inline(always)]
    fn from(variant: Busy) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BUSY` reader - SPC Busy Status Flag"]
pub type BusyR = crate::BitReader<Busy>;
impl BusyR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Busy {
        match self.bits {
            false => Busy::BusyNo,
            true => Busy::BusyYes,
        }
    }
    #[doc = "Not busy"]
    #[inline(always)]
    pub fn is_busy_no(&self) -> bool {
        *self == Busy::BusyNo
    }
    #[doc = "Busy"]
    #[inline(always)]
    pub fn is_busy_yes(&self) -> bool {
        *self == Busy::BusyYes
    }
}
#[doc = "SPC Power Mode Configuration Status Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpcLpReq {
    #[doc = "0: SPC is in Active mode; the ACTIVE_CFG register has control"]
    Active = 0,
    #[doc = "1: All power domains requested low-power mode; SPC entered a low-power state; power-mode configuration based on the LP_CFG register"]
    LowPower = 1,
}
impl From<SpcLpReq> for bool {
    #[inline(always)]
    fn from(variant: SpcLpReq) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SPC_LP_REQ` reader - SPC Power Mode Configuration Status Flag"]
pub type SpcLpReqR = crate::BitReader<SpcLpReq>;
impl SpcLpReqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> SpcLpReq {
        match self.bits {
            false => SpcLpReq::Active,
            true => SpcLpReq::LowPower,
        }
    }
    #[doc = "SPC is in Active mode; the ACTIVE_CFG register has control"]
    #[inline(always)]
    pub fn is_active(&self) -> bool {
        *self == SpcLpReq::Active
    }
    #[doc = "All power domains requested low-power mode; SPC entered a low-power state; power-mode configuration based on the LP_CFG register"]
    #[inline(always)]
    pub fn is_low_power(&self) -> bool {
        *self == SpcLpReq::LowPower
    }
}
#[doc = "Field `SPC_LP_REQ` writer - SPC Power Mode Configuration Status Flag"]
pub type SpcLpReqW<'a, REG> = crate::BitWriter1C<'a, REG, SpcLpReq>;
impl<'a, REG> SpcLpReqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "SPC is in Active mode; the ACTIVE_CFG register has control"]
    #[inline(always)]
    pub fn active(self) -> &'a mut crate::W<REG> {
        self.variant(SpcLpReq::Active)
    }
    #[doc = "All power domains requested low-power mode; SPC entered a low-power state; power-mode configuration based on the LP_CFG register"]
    #[inline(always)]
    pub fn low_power(self) -> &'a mut crate::W<REG> {
        self.variant(SpcLpReq::LowPower)
    }
}
#[doc = "Power Domain Low-Power Mode Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SpcLpMode {
    #[doc = "0: Sleep mode with system clock running"]
    Mode0 = 0,
    #[doc = "1: DSLEEP with system clock off"]
    Mode1 = 1,
    #[doc = "2: PDOWN with system clock off"]
    Mode2 = 2,
    #[doc = "8: DPDOWN with system clock off"]
    Mode8 = 8,
}
impl From<SpcLpMode> for u8 {
    #[inline(always)]
    fn from(variant: SpcLpMode) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for SpcLpMode {
    type Ux = u8;
}
impl crate::IsEnum for SpcLpMode {}
#[doc = "Field `SPC_LP_MODE` reader - Power Domain Low-Power Mode Request"]
pub type SpcLpModeR = crate::FieldReader<SpcLpMode>;
impl SpcLpModeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<SpcLpMode> {
        match self.bits {
            0 => Some(SpcLpMode::Mode0),
            1 => Some(SpcLpMode::Mode1),
            2 => Some(SpcLpMode::Mode2),
            8 => Some(SpcLpMode::Mode8),
            _ => None,
        }
    }
    #[doc = "Sleep mode with system clock running"]
    #[inline(always)]
    pub fn is_mode0(&self) -> bool {
        *self == SpcLpMode::Mode0
    }
    #[doc = "DSLEEP with system clock off"]
    #[inline(always)]
    pub fn is_mode1(&self) -> bool {
        *self == SpcLpMode::Mode1
    }
    #[doc = "PDOWN with system clock off"]
    #[inline(always)]
    pub fn is_mode2(&self) -> bool {
        *self == SpcLpMode::Mode2
    }
    #[doc = "DPDOWN with system clock off"]
    #[inline(always)]
    pub fn is_mode8(&self) -> bool {
        *self == SpcLpMode::Mode8
    }
}
#[doc = "Field `ISO_CLR` reader - Isolation Clear Flags"]
pub type IsoClrR = crate::BitReader;
#[doc = "Field `ISO_CLR` writer - Isolation Clear Flags"]
pub type IsoClrW<'a, REG> = crate::BitWriter1C<'a, REG>;
impl R {
    #[doc = "Bit 0 - SPC Busy Status Flag"]
    #[inline(always)]
    pub fn busy(&self) -> BusyR {
        BusyR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - SPC Power Mode Configuration Status Flag"]
    #[inline(always)]
    pub fn spc_lp_req(&self) -> SpcLpReqR {
        SpcLpReqR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 4:7 - Power Domain Low-Power Mode Request"]
    #[inline(always)]
    pub fn spc_lp_mode(&self) -> SpcLpModeR {
        SpcLpModeR::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bit 16 - Isolation Clear Flags"]
    #[inline(always)]
    pub fn iso_clr(&self) -> IsoClrR {
        IsoClrR::new(((self.bits >> 16) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 1 - SPC Power Mode Configuration Status Flag"]
    #[inline(always)]
    pub fn spc_lp_req(&mut self) -> SpcLpReqW<ScSpec> {
        SpcLpReqW::new(self, 1)
    }
    #[doc = "Bit 16 - Isolation Clear Flags"]
    #[inline(always)]
    pub fn iso_clr(&mut self) -> IsoClrW<ScSpec> {
        IsoClrW::new(self, 16)
    }
}
#[doc = "Status Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sc::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sc::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ScSpec;
impl crate::RegisterSpec for ScSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sc::R`](R) reader structure"]
impl crate::Readable for ScSpec {}
#[doc = "`write(|w| ..)` method takes [`sc::W`](W) writer structure"]
impl crate::Writable for ScSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0001_0002;
}
#[doc = "`reset()` method sets SC to value 0"]
impl crate::Resettable for ScSpec {}

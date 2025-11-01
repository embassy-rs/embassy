#[doc = "Register `OBSERVE` reader"]
pub type R = crate::R<ObserveSpec>;
#[doc = "D- Pulldown\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dmpd {
    #[doc = "0: Disabled"]
    DmPdDisStat = 0,
    #[doc = "1: Enabled"]
    DmPdEnStat = 1,
}
impl From<Dmpd> for bool {
    #[inline(always)]
    fn from(variant: Dmpd) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DMPD` reader - D- Pulldown"]
pub type DmpdR = crate::BitReader<Dmpd>;
impl DmpdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dmpd {
        match self.bits {
            false => Dmpd::DmPdDisStat,
            true => Dmpd::DmPdEnStat,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_dm_pd_dis_stat(&self) -> bool {
        *self == Dmpd::DmPdDisStat
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_dm_pd_en_stat(&self) -> bool {
        *self == Dmpd::DmPdEnStat
    }
}
#[doc = "D+ Pulldown\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dppd {
    #[doc = "0: Disabled"]
    DpPdDisStat = 0,
    #[doc = "1: Enabled"]
    DpPdEnStat = 1,
}
impl From<Dppd> for bool {
    #[inline(always)]
    fn from(variant: Dppd) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DPPD` reader - D+ Pulldown"]
pub type DppdR = crate::BitReader<Dppd>;
impl DppdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dppd {
        match self.bits {
            false => Dppd::DpPdDisStat,
            true => Dppd::DpPdEnStat,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_dp_pd_dis_stat(&self) -> bool {
        *self == Dppd::DpPdDisStat
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_dp_pd_en_stat(&self) -> bool {
        *self == Dppd::DpPdEnStat
    }
}
#[doc = "D+ Pullup\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dppu {
    #[doc = "0: Disabled"]
    DpPuDisStat = 0,
    #[doc = "1: Enabled"]
    DpPuEnStat = 1,
}
impl From<Dppu> for bool {
    #[inline(always)]
    fn from(variant: Dppu) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DPPU` reader - D+ Pullup"]
pub type DppuR = crate::BitReader<Dppu>;
impl DppuR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dppu {
        match self.bits {
            false => Dppu::DpPuDisStat,
            true => Dppu::DpPuEnStat,
        }
    }
    #[doc = "Disabled"]
    #[inline(always)]
    pub fn is_dp_pu_dis_stat(&self) -> bool {
        *self == Dppu::DpPuDisStat
    }
    #[doc = "Enabled"]
    #[inline(always)]
    pub fn is_dp_pu_en_stat(&self) -> bool {
        *self == Dppu::DpPuEnStat
    }
}
impl R {
    #[doc = "Bit 4 - D- Pulldown"]
    #[inline(always)]
    pub fn dmpd(&self) -> DmpdR {
        DmpdR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 6 - D+ Pulldown"]
    #[inline(always)]
    pub fn dppd(&self) -> DppdR {
        DppdR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - D+ Pullup"]
    #[inline(always)]
    pub fn dppu(&self) -> DppuR {
        DppuR::new(((self.bits >> 7) & 1) != 0)
    }
}
#[doc = "USB OTG Observe\n\nYou can [`read`](crate::Reg::read) this register and get [`observe::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ObserveSpec;
impl crate::RegisterSpec for ObserveSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`observe::R`](R) reader structure"]
impl crate::Readable for ObserveSpec {}
#[doc = "`reset()` method sets OBSERVE to value 0x50"]
impl crate::Resettable for ObserveSpec {
    const RESET_VALUE: u8 = 0x50;
}

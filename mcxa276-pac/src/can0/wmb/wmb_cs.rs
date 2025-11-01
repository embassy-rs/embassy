#[doc = "Register `WMB_CS` reader"]
pub type R = crate::R<WmbCsSpec>;
#[doc = "Field `DLC` reader - Length of Data in Bytes"]
pub type DlcR = crate::FieldReader;
#[doc = "Remote Transmission Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rtr {
    #[doc = "0: Data"]
    NotRemote = 0,
    #[doc = "1: Remote"]
    Remote = 1,
}
impl From<Rtr> for bool {
    #[inline(always)]
    fn from(variant: Rtr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RTR` reader - Remote Transmission Request"]
pub type RtrR = crate::BitReader<Rtr>;
impl RtrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rtr {
        match self.bits {
            false => Rtr::NotRemote,
            true => Rtr::Remote,
        }
    }
    #[doc = "Data"]
    #[inline(always)]
    pub fn is_not_remote(&self) -> bool {
        *self == Rtr::NotRemote
    }
    #[doc = "Remote"]
    #[inline(always)]
    pub fn is_remote(&self) -> bool {
        *self == Rtr::Remote
    }
}
#[doc = "ID Extended Bit\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ide {
    #[doc = "0: Standard"]
    Standard = 0,
    #[doc = "1: Extended"]
    Extended = 1,
}
impl From<Ide> for bool {
    #[inline(always)]
    fn from(variant: Ide) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IDE` reader - ID Extended Bit"]
pub type IdeR = crate::BitReader<Ide>;
impl IdeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ide {
        match self.bits {
            false => Ide::Standard,
            true => Ide::Extended,
        }
    }
    #[doc = "Standard"]
    #[inline(always)]
    pub fn is_standard(&self) -> bool {
        *self == Ide::Standard
    }
    #[doc = "Extended"]
    #[inline(always)]
    pub fn is_extended(&self) -> bool {
        *self == Ide::Extended
    }
}
#[doc = "Substitute Remote Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Srr {
    #[doc = "0: Dominant"]
    Dominant = 0,
    #[doc = "1: Recessive"]
    Recessive = 1,
}
impl From<Srr> for bool {
    #[inline(always)]
    fn from(variant: Srr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SRR` reader - Substitute Remote Request"]
pub type SrrR = crate::BitReader<Srr>;
impl SrrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Srr {
        match self.bits {
            false => Srr::Dominant,
            true => Srr::Recessive,
        }
    }
    #[doc = "Dominant"]
    #[inline(always)]
    pub fn is_dominant(&self) -> bool {
        *self == Srr::Dominant
    }
    #[doc = "Recessive"]
    #[inline(always)]
    pub fn is_recessive(&self) -> bool {
        *self == Srr::Recessive
    }
}
impl R {
    #[doc = "Bits 16:19 - Length of Data in Bytes"]
    #[inline(always)]
    pub fn dlc(&self) -> DlcR {
        DlcR::new(((self.bits >> 16) & 0x0f) as u8)
    }
    #[doc = "Bit 20 - Remote Transmission Request"]
    #[inline(always)]
    pub fn rtr(&self) -> RtrR {
        RtrR::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - ID Extended Bit"]
    #[inline(always)]
    pub fn ide(&self) -> IdeR {
        IdeR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Substitute Remote Request"]
    #[inline(always)]
    pub fn srr(&self) -> SrrR {
        SrrR::new(((self.bits >> 22) & 1) != 0)
    }
}
#[doc = "Wake-Up Message Buffer\n\nYou can [`read`](crate::Reg::read) this register and get [`wmb_cs::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct WmbCsSpec;
impl crate::RegisterSpec for WmbCsSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`wmb_cs::R`](R) reader structure"]
impl crate::Readable for WmbCsSpec {}
#[doc = "`reset()` method sets WMB_CS to value 0"]
impl crate::Resettable for WmbCsSpec {}

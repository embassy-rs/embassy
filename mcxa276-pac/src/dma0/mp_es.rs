#[doc = "Register `MP_ES` reader"]
pub type R = crate::R<MpEsSpec>;
#[doc = "Destination Bus Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dbe {
    #[doc = "0: No destination bus error"]
    NoError = 0,
    #[doc = "1: Last recorded error was a bus error on a destination write"]
    BusError = 1,
}
impl From<Dbe> for bool {
    #[inline(always)]
    fn from(variant: Dbe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DBE` reader - Destination Bus Error"]
pub type DbeR = crate::BitReader<Dbe>;
impl DbeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dbe {
        match self.bits {
            false => Dbe::NoError,
            true => Dbe::BusError,
        }
    }
    #[doc = "No destination bus error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Dbe::NoError
    }
    #[doc = "Last recorded error was a bus error on a destination write"]
    #[inline(always)]
    pub fn is_bus_error(&self) -> bool {
        *self == Dbe::BusError
    }
}
#[doc = "Source Bus Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sbe {
    #[doc = "0: No source bus error"]
    NoError = 0,
    #[doc = "1: Last recorded error was a bus error on a source read"]
    BusError = 1,
}
impl From<Sbe> for bool {
    #[inline(always)]
    fn from(variant: Sbe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SBE` reader - Source Bus Error"]
pub type SbeR = crate::BitReader<Sbe>;
impl SbeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sbe {
        match self.bits {
            false => Sbe::NoError,
            true => Sbe::BusError,
        }
    }
    #[doc = "No source bus error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Sbe::NoError
    }
    #[doc = "Last recorded error was a bus error on a source read"]
    #[inline(always)]
    pub fn is_bus_error(&self) -> bool {
        *self == Sbe::BusError
    }
}
#[doc = "Scatter/Gather Configuration Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sge {
    #[doc = "0: No scatter/gather configuration error"]
    NoError = 0,
    #[doc = "1: Last recorded error was a configuration error detected in the TCDn_DLAST_SGA field"]
    ConfigurationError = 1,
}
impl From<Sge> for bool {
    #[inline(always)]
    fn from(variant: Sge) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SGE` reader - Scatter/Gather Configuration Error"]
pub type SgeR = crate::BitReader<Sge>;
impl SgeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sge {
        match self.bits {
            false => Sge::NoError,
            true => Sge::ConfigurationError,
        }
    }
    #[doc = "No scatter/gather configuration error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Sge::NoError
    }
    #[doc = "Last recorded error was a configuration error detected in the TCDn_DLAST_SGA field"]
    #[inline(always)]
    pub fn is_configuration_error(&self) -> bool {
        *self == Sge::ConfigurationError
    }
}
#[doc = "NBYTES/CITER Configuration Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nce {
    #[doc = "0: No NBYTES/CITER configuration error"]
    NoError = 0,
    #[doc = "1: The last recorded error was NBYTES equal to zero or a CITER not equal to BITER error"]
    ConfigurationError = 1,
}
impl From<Nce> for bool {
    #[inline(always)]
    fn from(variant: Nce) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NCE` reader - NBYTES/CITER Configuration Error"]
pub type NceR = crate::BitReader<Nce>;
impl NceR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nce {
        match self.bits {
            false => Nce::NoError,
            true => Nce::ConfigurationError,
        }
    }
    #[doc = "No NBYTES/CITER configuration error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Nce::NoError
    }
    #[doc = "The last recorded error was NBYTES equal to zero or a CITER not equal to BITER error"]
    #[inline(always)]
    pub fn is_configuration_error(&self) -> bool {
        *self == Nce::ConfigurationError
    }
}
#[doc = "Destination Offset Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Doe {
    #[doc = "0: No destination offset configuration error"]
    NoError = 0,
    #[doc = "1: Last recorded error was a configuration error detected in the TCDn_DOFF field"]
    ConfigurationError = 1,
}
impl From<Doe> for bool {
    #[inline(always)]
    fn from(variant: Doe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DOE` reader - Destination Offset Error"]
pub type DoeR = crate::BitReader<Doe>;
impl DoeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Doe {
        match self.bits {
            false => Doe::NoError,
            true => Doe::ConfigurationError,
        }
    }
    #[doc = "No destination offset configuration error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Doe::NoError
    }
    #[doc = "Last recorded error was a configuration error detected in the TCDn_DOFF field"]
    #[inline(always)]
    pub fn is_configuration_error(&self) -> bool {
        *self == Doe::ConfigurationError
    }
}
#[doc = "Destination Address Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dae {
    #[doc = "0: No destination address configuration error"]
    NoError = 0,
    #[doc = "1: Last recorded error was a configuration error detected in the TCDn_DADDR field"]
    ConfigurationError = 1,
}
impl From<Dae> for bool {
    #[inline(always)]
    fn from(variant: Dae) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DAE` reader - Destination Address Error"]
pub type DaeR = crate::BitReader<Dae>;
impl DaeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dae {
        match self.bits {
            false => Dae::NoError,
            true => Dae::ConfigurationError,
        }
    }
    #[doc = "No destination address configuration error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Dae::NoError
    }
    #[doc = "Last recorded error was a configuration error detected in the TCDn_DADDR field"]
    #[inline(always)]
    pub fn is_configuration_error(&self) -> bool {
        *self == Dae::ConfigurationError
    }
}
#[doc = "Source Offset Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Soe {
    #[doc = "0: No source offset configuration error"]
    NoError = 0,
    #[doc = "1: Last recorded error was a configuration error detected in the TCDn_SOFF field"]
    ConfigurationError = 1,
}
impl From<Soe> for bool {
    #[inline(always)]
    fn from(variant: Soe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SOE` reader - Source Offset Error"]
pub type SoeR = crate::BitReader<Soe>;
impl SoeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Soe {
        match self.bits {
            false => Soe::NoError,
            true => Soe::ConfigurationError,
        }
    }
    #[doc = "No source offset configuration error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Soe::NoError
    }
    #[doc = "Last recorded error was a configuration error detected in the TCDn_SOFF field"]
    #[inline(always)]
    pub fn is_configuration_error(&self) -> bool {
        *self == Soe::ConfigurationError
    }
}
#[doc = "Source Address Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sae {
    #[doc = "0: No source address configuration error"]
    NoError = 0,
    #[doc = "1: Last recorded error was a configuration error detected in the TCDn_SADDR field"]
    ConfigurationError = 1,
}
impl From<Sae> for bool {
    #[inline(always)]
    fn from(variant: Sae) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SAE` reader - Source Address Error"]
pub type SaeR = crate::BitReader<Sae>;
impl SaeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sae {
        match self.bits {
            false => Sae::NoError,
            true => Sae::ConfigurationError,
        }
    }
    #[doc = "No source address configuration error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Sae::NoError
    }
    #[doc = "Last recorded error was a configuration error detected in the TCDn_SADDR field"]
    #[inline(always)]
    pub fn is_configuration_error(&self) -> bool {
        *self == Sae::ConfigurationError
    }
}
#[doc = "Transfer Canceled\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ecx {
    #[doc = "0: No canceled transfers"]
    NoCanceledTransfers = 0,
    #[doc = "1: Last recorded entry was a canceled transfer by the error cancel transfer input"]
    CanceledTransfer = 1,
}
impl From<Ecx> for bool {
    #[inline(always)]
    fn from(variant: Ecx) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ECX` reader - Transfer Canceled"]
pub type EcxR = crate::BitReader<Ecx>;
impl EcxR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ecx {
        match self.bits {
            false => Ecx::NoCanceledTransfers,
            true => Ecx::CanceledTransfer,
        }
    }
    #[doc = "No canceled transfers"]
    #[inline(always)]
    pub fn is_no_canceled_transfers(&self) -> bool {
        *self == Ecx::NoCanceledTransfers
    }
    #[doc = "Last recorded entry was a canceled transfer by the error cancel transfer input"]
    #[inline(always)]
    pub fn is_canceled_transfer(&self) -> bool {
        *self == Ecx::CanceledTransfer
    }
}
#[doc = "Field `ERRCHN` reader - Error Channel Number or Canceled Channel Number"]
pub type ErrchnR = crate::FieldReader;
#[doc = "Valid\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vld {
    #[doc = "0: No CHn_ES\\[ERR\\] fields are set to 1"]
    NoFieldSetOne = 0,
    #[doc = "1: At least one CHn_ES\\[ERR\\] field is set to 1, indicating a valid error exists that software has not cleared"]
    AtleastOneField = 1,
}
impl From<Vld> for bool {
    #[inline(always)]
    fn from(variant: Vld) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `VLD` reader - Valid"]
pub type VldR = crate::BitReader<Vld>;
impl VldR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Vld {
        match self.bits {
            false => Vld::NoFieldSetOne,
            true => Vld::AtleastOneField,
        }
    }
    #[doc = "No CHn_ES\\[ERR\\] fields are set to 1"]
    #[inline(always)]
    pub fn is_no_field_set_one(&self) -> bool {
        *self == Vld::NoFieldSetOne
    }
    #[doc = "At least one CHn_ES\\[ERR\\] field is set to 1, indicating a valid error exists that software has not cleared"]
    #[inline(always)]
    pub fn is_atleast_one_field(&self) -> bool {
        *self == Vld::AtleastOneField
    }
}
impl R {
    #[doc = "Bit 0 - Destination Bus Error"]
    #[inline(always)]
    pub fn dbe(&self) -> DbeR {
        DbeR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Source Bus Error"]
    #[inline(always)]
    pub fn sbe(&self) -> SbeR {
        SbeR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Scatter/Gather Configuration Error"]
    #[inline(always)]
    pub fn sge(&self) -> SgeR {
        SgeR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - NBYTES/CITER Configuration Error"]
    #[inline(always)]
    pub fn nce(&self) -> NceR {
        NceR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Destination Offset Error"]
    #[inline(always)]
    pub fn doe(&self) -> DoeR {
        DoeR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Destination Address Error"]
    #[inline(always)]
    pub fn dae(&self) -> DaeR {
        DaeR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Source Offset Error"]
    #[inline(always)]
    pub fn soe(&self) -> SoeR {
        SoeR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Source Address Error"]
    #[inline(always)]
    pub fn sae(&self) -> SaeR {
        SaeR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - Transfer Canceled"]
    #[inline(always)]
    pub fn ecx(&self) -> EcxR {
        EcxR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bits 24:26 - Error Channel Number or Canceled Channel Number"]
    #[inline(always)]
    pub fn errchn(&self) -> ErrchnR {
        ErrchnR::new(((self.bits >> 24) & 7) as u8)
    }
    #[doc = "Bit 31 - Valid"]
    #[inline(always)]
    pub fn vld(&self) -> VldR {
        VldR::new(((self.bits >> 31) & 1) != 0)
    }
}
#[doc = "Management Page Error Status\n\nYou can [`read`](crate::Reg::read) this register and get [`mp_es::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MpEsSpec;
impl crate::RegisterSpec for MpEsSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mp_es::R`](R) reader structure"]
impl crate::Readable for MpEsSpec {}
#[doc = "`reset()` method sets MP_ES to value 0"]
impl crate::Resettable for MpEsSpec {}

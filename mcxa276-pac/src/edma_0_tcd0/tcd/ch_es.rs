#[doc = "Register `CH_ES` reader"]
pub type R = crate::R<ChEsSpec>;
#[doc = "Register `CH_ES` writer"]
pub type W = crate::W<ChEsSpec>;
#[doc = "Destination Bus Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dbe {
    #[doc = "0: No destination bus error"]
    NoError = 0,
    #[doc = "1: Last recorded error was bus error on destination write"]
    Error = 1,
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
            true => Dbe::Error,
        }
    }
    #[doc = "No destination bus error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Dbe::NoError
    }
    #[doc = "Last recorded error was bus error on destination write"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Dbe::Error
    }
}
#[doc = "Source Bus Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sbe {
    #[doc = "0: No source bus error"]
    NoError = 0,
    #[doc = "1: Last recorded error was bus error on source read"]
    Error = 1,
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
            true => Sbe::Error,
        }
    }
    #[doc = "No source bus error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Sbe::NoError
    }
    #[doc = "Last recorded error was bus error on source read"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Sbe::Error
    }
}
#[doc = "Scatter/Gather Configuration Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sge {
    #[doc = "0: No scatter/gather configuration error"]
    NoError = 0,
    #[doc = "1: Last recorded error was a configuration error detected in the TCDn_DLAST_SGA field"]
    Error = 1,
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
            true => Sge::Error,
        }
    }
    #[doc = "No scatter/gather configuration error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Sge::NoError
    }
    #[doc = "Last recorded error was a configuration error detected in the TCDn_DLAST_SGA field"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Sge::Error
    }
}
#[doc = "NBYTES/CITER Configuration Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nce {
    #[doc = "0: No NBYTES/CITER configuration error"]
    NoError = 0,
    #[doc = "1: Last recorded error was a configuration error detected in the TCDn_NBYTES or TCDn_CITER fields"]
    Error = 1,
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
            true => Nce::Error,
        }
    }
    #[doc = "No NBYTES/CITER configuration error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Nce::NoError
    }
    #[doc = "Last recorded error was a configuration error detected in the TCDn_NBYTES or TCDn_CITER fields"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Nce::Error
    }
}
#[doc = "Destination Offset Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Doe {
    #[doc = "0: No destination offset configuration error"]
    NoError = 0,
    #[doc = "1: Last recorded error was a configuration error detected in the TCDn_DOFF field"]
    Error = 1,
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
            true => Doe::Error,
        }
    }
    #[doc = "No destination offset configuration error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Doe::NoError
    }
    #[doc = "Last recorded error was a configuration error detected in the TCDn_DOFF field"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Doe::Error
    }
}
#[doc = "Destination Address Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dae {
    #[doc = "0: No destination address configuration error"]
    NoError = 0,
    #[doc = "1: Last recorded error was a configuration error detected in the TCDn_DADDR field"]
    Error = 1,
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
            true => Dae::Error,
        }
    }
    #[doc = "No destination address configuration error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Dae::NoError
    }
    #[doc = "Last recorded error was a configuration error detected in the TCDn_DADDR field"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Dae::Error
    }
}
#[doc = "Source Offset Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Soe {
    #[doc = "0: No source offset configuration error"]
    NoError = 0,
    #[doc = "1: Last recorded error was a configuration error detected in the TCDn_SOFF field"]
    Error = 1,
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
            true => Soe::Error,
        }
    }
    #[doc = "No source offset configuration error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Soe::NoError
    }
    #[doc = "Last recorded error was a configuration error detected in the TCDn_SOFF field"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Soe::Error
    }
}
#[doc = "Source Address Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sae {
    #[doc = "0: No source address configuration error"]
    NoError = 0,
    #[doc = "1: Last recorded error was a configuration error detected in the TCDn_SADDR field"]
    Error = 1,
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
            true => Sae::Error,
        }
    }
    #[doc = "No source address configuration error"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Sae::NoError
    }
    #[doc = "Last recorded error was a configuration error detected in the TCDn_SADDR field"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Sae::Error
    }
}
#[doc = "Error In Channel\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Err {
    #[doc = "0: An error in this channel has not occurred"]
    NoError = 0,
    #[doc = "1: An error in this channel has occurred"]
    Error = 1,
}
impl From<Err> for bool {
    #[inline(always)]
    fn from(variant: Err) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERR` reader - Error In Channel"]
pub type ErrR = crate::BitReader<Err>;
impl ErrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Err {
        match self.bits {
            false => Err::NoError,
            true => Err::Error,
        }
    }
    #[doc = "An error in this channel has not occurred"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Err::NoError
    }
    #[doc = "An error in this channel has occurred"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Err::Error
    }
}
#[doc = "Field `ERR` writer - Error In Channel"]
pub type ErrW<'a, REG> = crate::BitWriter1C<'a, REG, Err>;
impl<'a, REG> ErrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "An error in this channel has not occurred"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Err::NoError)
    }
    #[doc = "An error in this channel has occurred"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Err::Error)
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
    #[doc = "Bit 31 - Error In Channel"]
    #[inline(always)]
    pub fn err(&self) -> ErrR {
        ErrR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 31 - Error In Channel"]
    #[inline(always)]
    pub fn err(&mut self) -> ErrW<ChEsSpec> {
        ErrW::new(self, 31)
    }
}
#[doc = "Channel Error Status\n\nYou can [`read`](crate::Reg::read) this register and get [`ch_es::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch_es::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ChEsSpec;
impl crate::RegisterSpec for ChEsSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ch_es::R`](R) reader structure"]
impl crate::Readable for ChEsSpec {}
#[doc = "`write(|w| ..)` method takes [`ch_es::W`](W) writer structure"]
impl crate::Writable for ChEsSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x8000_0000;
}
#[doc = "`reset()` method sets CH_ES to value 0"]
impl crate::Resettable for ChEsSpec {}

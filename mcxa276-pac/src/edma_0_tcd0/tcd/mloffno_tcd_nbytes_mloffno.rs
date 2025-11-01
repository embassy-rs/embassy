#[doc = "Register `TCD_NBYTES_MLOFFNO` reader"]
pub type R = crate::R<MloffnoTcdNbytesMloffnoSpec>;
#[doc = "Register `TCD_NBYTES_MLOFFNO` writer"]
pub type W = crate::W<MloffnoTcdNbytesMloffnoSpec>;
#[doc = "Field `NBYTES` reader - Number of Bytes To Transfer Per Service Request"]
pub type NbytesR = crate::FieldReader<u32>;
#[doc = "Field `NBYTES` writer - Number of Bytes To Transfer Per Service Request"]
pub type NbytesW<'a, REG> = crate::FieldWriter<'a, REG, 30, u32>;
#[doc = "Destination Minor Loop Offset Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dmloe {
    #[doc = "0: Minor loop offset not applied to DADDR"]
    OffsetNotApplied = 0,
    #[doc = "1: Minor loop offset applied to DADDR"]
    OffsetApplied = 1,
}
impl From<Dmloe> for bool {
    #[inline(always)]
    fn from(variant: Dmloe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DMLOE` reader - Destination Minor Loop Offset Enable"]
pub type DmloeR = crate::BitReader<Dmloe>;
impl DmloeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dmloe {
        match self.bits {
            false => Dmloe::OffsetNotApplied,
            true => Dmloe::OffsetApplied,
        }
    }
    #[doc = "Minor loop offset not applied to DADDR"]
    #[inline(always)]
    pub fn is_offset_not_applied(&self) -> bool {
        *self == Dmloe::OffsetNotApplied
    }
    #[doc = "Minor loop offset applied to DADDR"]
    #[inline(always)]
    pub fn is_offset_applied(&self) -> bool {
        *self == Dmloe::OffsetApplied
    }
}
#[doc = "Field `DMLOE` writer - Destination Minor Loop Offset Enable"]
pub type DmloeW<'a, REG> = crate::BitWriter<'a, REG, Dmloe>;
impl<'a, REG> DmloeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Minor loop offset not applied to DADDR"]
    #[inline(always)]
    pub fn offset_not_applied(self) -> &'a mut crate::W<REG> {
        self.variant(Dmloe::OffsetNotApplied)
    }
    #[doc = "Minor loop offset applied to DADDR"]
    #[inline(always)]
    pub fn offset_applied(self) -> &'a mut crate::W<REG> {
        self.variant(Dmloe::OffsetApplied)
    }
}
#[doc = "Source Minor Loop Offset Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Smloe {
    #[doc = "0: Minor loop offset not applied to SADDR"]
    OffsetNotApplied = 0,
    #[doc = "1: Minor loop offset applied to SADDR"]
    OffsetApplied = 1,
}
impl From<Smloe> for bool {
    #[inline(always)]
    fn from(variant: Smloe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SMLOE` reader - Source Minor Loop Offset Enable"]
pub type SmloeR = crate::BitReader<Smloe>;
impl SmloeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Smloe {
        match self.bits {
            false => Smloe::OffsetNotApplied,
            true => Smloe::OffsetApplied,
        }
    }
    #[doc = "Minor loop offset not applied to SADDR"]
    #[inline(always)]
    pub fn is_offset_not_applied(&self) -> bool {
        *self == Smloe::OffsetNotApplied
    }
    #[doc = "Minor loop offset applied to SADDR"]
    #[inline(always)]
    pub fn is_offset_applied(&self) -> bool {
        *self == Smloe::OffsetApplied
    }
}
#[doc = "Field `SMLOE` writer - Source Minor Loop Offset Enable"]
pub type SmloeW<'a, REG> = crate::BitWriter<'a, REG, Smloe>;
impl<'a, REG> SmloeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Minor loop offset not applied to SADDR"]
    #[inline(always)]
    pub fn offset_not_applied(self) -> &'a mut crate::W<REG> {
        self.variant(Smloe::OffsetNotApplied)
    }
    #[doc = "Minor loop offset applied to SADDR"]
    #[inline(always)]
    pub fn offset_applied(self) -> &'a mut crate::W<REG> {
        self.variant(Smloe::OffsetApplied)
    }
}
impl R {
    #[doc = "Bits 0:29 - Number of Bytes To Transfer Per Service Request"]
    #[inline(always)]
    pub fn nbytes(&self) -> NbytesR {
        NbytesR::new(self.bits & 0x3fff_ffff)
    }
    #[doc = "Bit 30 - Destination Minor Loop Offset Enable"]
    #[inline(always)]
    pub fn dmloe(&self) -> DmloeR {
        DmloeR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Source Minor Loop Offset Enable"]
    #[inline(always)]
    pub fn smloe(&self) -> SmloeR {
        SmloeR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:29 - Number of Bytes To Transfer Per Service Request"]
    #[inline(always)]
    pub fn nbytes(&mut self) -> NbytesW<MloffnoTcdNbytesMloffnoSpec> {
        NbytesW::new(self, 0)
    }
    #[doc = "Bit 30 - Destination Minor Loop Offset Enable"]
    #[inline(always)]
    pub fn dmloe(&mut self) -> DmloeW<MloffnoTcdNbytesMloffnoSpec> {
        DmloeW::new(self, 30)
    }
    #[doc = "Bit 31 - Source Minor Loop Offset Enable"]
    #[inline(always)]
    pub fn smloe(&mut self) -> SmloeW<MloffnoTcdNbytesMloffnoSpec> {
        SmloeW::new(self, 31)
    }
}
#[doc = "TCD Transfer Size Without Minor Loop Offsets\n\nYou can [`read`](crate::Reg::read) this register and get [`mloffno_tcd_nbytes_mloffno::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mloffno_tcd_nbytes_mloffno::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MloffnoTcdNbytesMloffnoSpec;
impl crate::RegisterSpec for MloffnoTcdNbytesMloffnoSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mloffno_tcd_nbytes_mloffno::R`](R) reader structure"]
impl crate::Readable for MloffnoTcdNbytesMloffnoSpec {}
#[doc = "`write(|w| ..)` method takes [`mloffno_tcd_nbytes_mloffno::W`](W) writer structure"]
impl crate::Writable for MloffnoTcdNbytesMloffnoSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TCD_NBYTES_MLOFFNO to value 0"]
impl crate::Resettable for MloffnoTcdNbytesMloffnoSpec {}

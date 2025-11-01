#[doc = "Register `TCD_NBYTES_MLOFFYES` reader"]
pub type R = crate::R<MloffyesTcdNbytesMloffyesSpec>;
#[doc = "Register `TCD_NBYTES_MLOFFYES` writer"]
pub type W = crate::W<MloffyesTcdNbytesMloffyesSpec>;
#[doc = "Field `NBYTES` reader - Number of Bytes To Transfer Per Service Request"]
pub type NbytesR = crate::FieldReader<u16>;
#[doc = "Field `NBYTES` writer - Number of Bytes To Transfer Per Service Request"]
pub type NbytesW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
#[doc = "Field `MLOFF` reader - Minor Loop Offset"]
pub type MloffR = crate::FieldReader<u32>;
#[doc = "Field `MLOFF` writer - Minor Loop Offset"]
pub type MloffW<'a, REG> = crate::FieldWriter<'a, REG, 20, u32>;
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
    #[doc = "Bits 0:9 - Number of Bytes To Transfer Per Service Request"]
    #[inline(always)]
    pub fn nbytes(&self) -> NbytesR {
        NbytesR::new((self.bits & 0x03ff) as u16)
    }
    #[doc = "Bits 10:29 - Minor Loop Offset"]
    #[inline(always)]
    pub fn mloff(&self) -> MloffR {
        MloffR::new((self.bits >> 10) & 0x000f_ffff)
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
    #[doc = "Bits 0:9 - Number of Bytes To Transfer Per Service Request"]
    #[inline(always)]
    pub fn nbytes(&mut self) -> NbytesW<MloffyesTcdNbytesMloffyesSpec> {
        NbytesW::new(self, 0)
    }
    #[doc = "Bits 10:29 - Minor Loop Offset"]
    #[inline(always)]
    pub fn mloff(&mut self) -> MloffW<MloffyesTcdNbytesMloffyesSpec> {
        MloffW::new(self, 10)
    }
    #[doc = "Bit 30 - Destination Minor Loop Offset Enable"]
    #[inline(always)]
    pub fn dmloe(&mut self) -> DmloeW<MloffyesTcdNbytesMloffyesSpec> {
        DmloeW::new(self, 30)
    }
    #[doc = "Bit 31 - Source Minor Loop Offset Enable"]
    #[inline(always)]
    pub fn smloe(&mut self) -> SmloeW<MloffyesTcdNbytesMloffyesSpec> {
        SmloeW::new(self, 31)
    }
}
#[doc = "TCD Transfer Size with Minor Loop Offsets\n\nYou can [`read`](crate::Reg::read) this register and get [`mloffyes_tcd_nbytes_mloffyes::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mloffyes_tcd_nbytes_mloffyes::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MloffyesTcdNbytesMloffyesSpec;
impl crate::RegisterSpec for MloffyesTcdNbytesMloffyesSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mloffyes_tcd_nbytes_mloffyes::R`](R) reader structure"]
impl crate::Readable for MloffyesTcdNbytesMloffyesSpec {}
#[doc = "`write(|w| ..)` method takes [`mloffyes_tcd_nbytes_mloffyes::W`](W) writer structure"]
impl crate::Writable for MloffyesTcdNbytesMloffyesSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TCD_NBYTES_MLOFFYES to value 0"]
impl crate::Resettable for MloffyesTcdNbytesMloffyesSpec {}

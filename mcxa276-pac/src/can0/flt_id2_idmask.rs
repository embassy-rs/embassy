#[doc = "Register `FLT_ID2_IDMASK` reader"]
pub type R = crate::R<FltId2IdmaskSpec>;
#[doc = "Register `FLT_ID2_IDMASK` writer"]
pub type W = crate::W<FltId2IdmaskSpec>;
#[doc = "Field `FLT_ID2_IDMASK` reader - ID Filter 2 for Pretended Networking Filtering or ID Mask Bits for Pretended Networking ID Filtering"]
pub type FltId2IdmaskR = crate::FieldReader<u32>;
#[doc = "Field `FLT_ID2_IDMASK` writer - ID Filter 2 for Pretended Networking Filtering or ID Mask Bits for Pretended Networking ID Filtering"]
pub type FltId2IdmaskW<'a, REG> = crate::FieldWriter<'a, REG, 29, u32>;
#[doc = "Remote Transmission Request Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RtrMsk {
    #[doc = "0: The corresponding bit in the filter is \"don't care.\""]
    FrameTypeNo = 0,
    #[doc = "1: The corresponding bit in the filter is checked."]
    FrameTypeYes = 1,
}
impl From<RtrMsk> for bool {
    #[inline(always)]
    fn from(variant: RtrMsk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RTR_MSK` reader - Remote Transmission Request Mask"]
pub type RtrMskR = crate::BitReader<RtrMsk>;
impl RtrMskR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RtrMsk {
        match self.bits {
            false => RtrMsk::FrameTypeNo,
            true => RtrMsk::FrameTypeYes,
        }
    }
    #[doc = "The corresponding bit in the filter is \"don't care.\""]
    #[inline(always)]
    pub fn is_frame_type_no(&self) -> bool {
        *self == RtrMsk::FrameTypeNo
    }
    #[doc = "The corresponding bit in the filter is checked."]
    #[inline(always)]
    pub fn is_frame_type_yes(&self) -> bool {
        *self == RtrMsk::FrameTypeYes
    }
}
#[doc = "Field `RTR_MSK` writer - Remote Transmission Request Mask"]
pub type RtrMskW<'a, REG> = crate::BitWriter<'a, REG, RtrMsk>;
impl<'a, REG> RtrMskW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The corresponding bit in the filter is \"don't care.\""]
    #[inline(always)]
    pub fn frame_type_no(self) -> &'a mut crate::W<REG> {
        self.variant(RtrMsk::FrameTypeNo)
    }
    #[doc = "The corresponding bit in the filter is checked."]
    #[inline(always)]
    pub fn frame_type_yes(self) -> &'a mut crate::W<REG> {
        self.variant(RtrMsk::FrameTypeYes)
    }
}
#[doc = "ID Extended Mask\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdeMsk {
    #[doc = "0: The corresponding bit in the filter is \"don't care.\""]
    FrameFormatNo = 0,
    #[doc = "1: The corresponding bit in the filter is checked."]
    FrameFormatYes = 1,
}
impl From<IdeMsk> for bool {
    #[inline(always)]
    fn from(variant: IdeMsk) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `IDE_MSK` reader - ID Extended Mask"]
pub type IdeMskR = crate::BitReader<IdeMsk>;
impl IdeMskR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> IdeMsk {
        match self.bits {
            false => IdeMsk::FrameFormatNo,
            true => IdeMsk::FrameFormatYes,
        }
    }
    #[doc = "The corresponding bit in the filter is \"don't care.\""]
    #[inline(always)]
    pub fn is_frame_format_no(&self) -> bool {
        *self == IdeMsk::FrameFormatNo
    }
    #[doc = "The corresponding bit in the filter is checked."]
    #[inline(always)]
    pub fn is_frame_format_yes(&self) -> bool {
        *self == IdeMsk::FrameFormatYes
    }
}
#[doc = "Field `IDE_MSK` writer - ID Extended Mask"]
pub type IdeMskW<'a, REG> = crate::BitWriter<'a, REG, IdeMsk>;
impl<'a, REG> IdeMskW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "The corresponding bit in the filter is \"don't care.\""]
    #[inline(always)]
    pub fn frame_format_no(self) -> &'a mut crate::W<REG> {
        self.variant(IdeMsk::FrameFormatNo)
    }
    #[doc = "The corresponding bit in the filter is checked."]
    #[inline(always)]
    pub fn frame_format_yes(self) -> &'a mut crate::W<REG> {
        self.variant(IdeMsk::FrameFormatYes)
    }
}
impl R {
    #[doc = "Bits 0:28 - ID Filter 2 for Pretended Networking Filtering or ID Mask Bits for Pretended Networking ID Filtering"]
    #[inline(always)]
    pub fn flt_id2_idmask(&self) -> FltId2IdmaskR {
        FltId2IdmaskR::new(self.bits & 0x1fff_ffff)
    }
    #[doc = "Bit 29 - Remote Transmission Request Mask"]
    #[inline(always)]
    pub fn rtr_msk(&self) -> RtrMskR {
        RtrMskR::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - ID Extended Mask"]
    #[inline(always)]
    pub fn ide_msk(&self) -> IdeMskR {
        IdeMskR::new(((self.bits >> 30) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:28 - ID Filter 2 for Pretended Networking Filtering or ID Mask Bits for Pretended Networking ID Filtering"]
    #[inline(always)]
    pub fn flt_id2_idmask(&mut self) -> FltId2IdmaskW<FltId2IdmaskSpec> {
        FltId2IdmaskW::new(self, 0)
    }
    #[doc = "Bit 29 - Remote Transmission Request Mask"]
    #[inline(always)]
    pub fn rtr_msk(&mut self) -> RtrMskW<FltId2IdmaskSpec> {
        RtrMskW::new(self, 29)
    }
    #[doc = "Bit 30 - ID Extended Mask"]
    #[inline(always)]
    pub fn ide_msk(&mut self) -> IdeMskW<FltId2IdmaskSpec> {
        IdeMskW::new(self, 30)
    }
}
#[doc = "Pretended Networking ID Filter 2 or ID Mask\n\nYou can [`read`](crate::Reg::read) this register and get [`flt_id2_idmask::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flt_id2_idmask::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FltId2IdmaskSpec;
impl crate::RegisterSpec for FltId2IdmaskSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`flt_id2_idmask::R`](R) reader structure"]
impl crate::Readable for FltId2IdmaskSpec {}
#[doc = "`write(|w| ..)` method takes [`flt_id2_idmask::W`](W) writer structure"]
impl crate::Writable for FltId2IdmaskSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FLT_ID2_IDMASK to value 0"]
impl crate::Resettable for FltId2IdmaskSpec {}

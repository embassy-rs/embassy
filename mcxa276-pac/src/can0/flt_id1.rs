#[doc = "Register `FLT_ID1` reader"]
pub type R = crate::R<FltId1Spec>;
#[doc = "Register `FLT_ID1` writer"]
pub type W = crate::W<FltId1Spec>;
#[doc = "Field `FLT_ID1` reader - ID Filter 1 for Pretended Networking filtering"]
pub type FltId1R = crate::FieldReader<u32>;
#[doc = "Field `FLT_ID1` writer - ID Filter 1 for Pretended Networking filtering"]
pub type FltId1W<'a, REG> = crate::FieldWriter<'a, REG, 29, u32>;
#[doc = "Remote Transmission Request Filter\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FltRtr {
    #[doc = "0: Reject remote frame (accept data frame)"]
    Reject = 0,
    #[doc = "1: Accept remote frame"]
    Accept = 1,
}
impl From<FltRtr> for bool {
    #[inline(always)]
    fn from(variant: FltRtr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FLT_RTR` reader - Remote Transmission Request Filter"]
pub type FltRtrR = crate::BitReader<FltRtr>;
impl FltRtrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FltRtr {
        match self.bits {
            false => FltRtr::Reject,
            true => FltRtr::Accept,
        }
    }
    #[doc = "Reject remote frame (accept data frame)"]
    #[inline(always)]
    pub fn is_reject(&self) -> bool {
        *self == FltRtr::Reject
    }
    #[doc = "Accept remote frame"]
    #[inline(always)]
    pub fn is_accept(&self) -> bool {
        *self == FltRtr::Accept
    }
}
#[doc = "Field `FLT_RTR` writer - Remote Transmission Request Filter"]
pub type FltRtrW<'a, REG> = crate::BitWriter<'a, REG, FltRtr>;
impl<'a, REG> FltRtrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Reject remote frame (accept data frame)"]
    #[inline(always)]
    pub fn reject(self) -> &'a mut crate::W<REG> {
        self.variant(FltRtr::Reject)
    }
    #[doc = "Accept remote frame"]
    #[inline(always)]
    pub fn accept(self) -> &'a mut crate::W<REG> {
        self.variant(FltRtr::Accept)
    }
}
#[doc = "ID Extended Filter\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FltIde {
    #[doc = "0: Standard"]
    Standard = 0,
    #[doc = "1: Extended"]
    Extended = 1,
}
impl From<FltIde> for bool {
    #[inline(always)]
    fn from(variant: FltIde) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FLT_IDE` reader - ID Extended Filter"]
pub type FltIdeR = crate::BitReader<FltIde>;
impl FltIdeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FltIde {
        match self.bits {
            false => FltIde::Standard,
            true => FltIde::Extended,
        }
    }
    #[doc = "Standard"]
    #[inline(always)]
    pub fn is_standard(&self) -> bool {
        *self == FltIde::Standard
    }
    #[doc = "Extended"]
    #[inline(always)]
    pub fn is_extended(&self) -> bool {
        *self == FltIde::Extended
    }
}
#[doc = "Field `FLT_IDE` writer - ID Extended Filter"]
pub type FltIdeW<'a, REG> = crate::BitWriter<'a, REG, FltIde>;
impl<'a, REG> FltIdeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Standard"]
    #[inline(always)]
    pub fn standard(self) -> &'a mut crate::W<REG> {
        self.variant(FltIde::Standard)
    }
    #[doc = "Extended"]
    #[inline(always)]
    pub fn extended(self) -> &'a mut crate::W<REG> {
        self.variant(FltIde::Extended)
    }
}
impl R {
    #[doc = "Bits 0:28 - ID Filter 1 for Pretended Networking filtering"]
    #[inline(always)]
    pub fn flt_id1(&self) -> FltId1R {
        FltId1R::new(self.bits & 0x1fff_ffff)
    }
    #[doc = "Bit 29 - Remote Transmission Request Filter"]
    #[inline(always)]
    pub fn flt_rtr(&self) -> FltRtrR {
        FltRtrR::new(((self.bits >> 29) & 1) != 0)
    }
    #[doc = "Bit 30 - ID Extended Filter"]
    #[inline(always)]
    pub fn flt_ide(&self) -> FltIdeR {
        FltIdeR::new(((self.bits >> 30) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:28 - ID Filter 1 for Pretended Networking filtering"]
    #[inline(always)]
    pub fn flt_id1(&mut self) -> FltId1W<FltId1Spec> {
        FltId1W::new(self, 0)
    }
    #[doc = "Bit 29 - Remote Transmission Request Filter"]
    #[inline(always)]
    pub fn flt_rtr(&mut self) -> FltRtrW<FltId1Spec> {
        FltRtrW::new(self, 29)
    }
    #[doc = "Bit 30 - ID Extended Filter"]
    #[inline(always)]
    pub fn flt_ide(&mut self) -> FltIdeW<FltId1Spec> {
        FltIdeW::new(self, 30)
    }
}
#[doc = "Pretended Networking ID Filter 1\n\nYou can [`read`](crate::Reg::read) this register and get [`flt_id1::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flt_id1::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FltId1Spec;
impl crate::RegisterSpec for FltId1Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`flt_id1::R`](R) reader structure"]
impl crate::Readable for FltId1Spec {}
#[doc = "`write(|w| ..)` method takes [`flt_id1::W`](W) writer structure"]
impl crate::Writable for FltId1Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FLT_ID1 to value 0"]
impl crate::Resettable for FltId1Spec {}

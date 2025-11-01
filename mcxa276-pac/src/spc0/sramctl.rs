#[doc = "Register `SRAMCTL` reader"]
pub type R = crate::R<SramctlSpec>;
#[doc = "Register `SRAMCTL` writer"]
pub type W = crate::W<SramctlSpec>;
#[doc = "Voltage Select Margin\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Vsm {
    #[doc = "1: 1.0 V"]
    Vsm1 = 1,
    #[doc = "2: 1.1 V"]
    Vsm2 = 2,
}
impl From<Vsm> for u8 {
    #[inline(always)]
    fn from(variant: Vsm) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Vsm {
    type Ux = u8;
}
impl crate::IsEnum for Vsm {}
#[doc = "Field `VSM` reader - Voltage Select Margin"]
pub type VsmR = crate::FieldReader<Vsm>;
impl VsmR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Vsm> {
        match self.bits {
            1 => Some(Vsm::Vsm1),
            2 => Some(Vsm::Vsm2),
            _ => None,
        }
    }
    #[doc = "1.0 V"]
    #[inline(always)]
    pub fn is_vsm1(&self) -> bool {
        *self == Vsm::Vsm1
    }
    #[doc = "1.1 V"]
    #[inline(always)]
    pub fn is_vsm2(&self) -> bool {
        *self == Vsm::Vsm2
    }
}
#[doc = "Field `VSM` writer - Voltage Select Margin"]
pub type VsmW<'a, REG> = crate::FieldWriter<'a, REG, 2, Vsm>;
impl<'a, REG> VsmW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "1.0 V"]
    #[inline(always)]
    pub fn vsm1(self) -> &'a mut crate::W<REG> {
        self.variant(Vsm::Vsm1)
    }
    #[doc = "1.1 V"]
    #[inline(always)]
    pub fn vsm2(self) -> &'a mut crate::W<REG> {
        self.variant(Vsm::Vsm2)
    }
}
#[doc = "SRAM Voltage Update Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Req {
    #[doc = "0: Do not request"]
    ReqNo = 0,
    #[doc = "1: Request"]
    ReqYes = 1,
}
impl From<Req> for bool {
    #[inline(always)]
    fn from(variant: Req) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `REQ` reader - SRAM Voltage Update Request"]
pub type ReqR = crate::BitReader<Req>;
impl ReqR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Req {
        match self.bits {
            false => Req::ReqNo,
            true => Req::ReqYes,
        }
    }
    #[doc = "Do not request"]
    #[inline(always)]
    pub fn is_req_no(&self) -> bool {
        *self == Req::ReqNo
    }
    #[doc = "Request"]
    #[inline(always)]
    pub fn is_req_yes(&self) -> bool {
        *self == Req::ReqYes
    }
}
#[doc = "Field `REQ` writer - SRAM Voltage Update Request"]
pub type ReqW<'a, REG> = crate::BitWriter<'a, REG, Req>;
impl<'a, REG> ReqW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Do not request"]
    #[inline(always)]
    pub fn req_no(self) -> &'a mut crate::W<REG> {
        self.variant(Req::ReqNo)
    }
    #[doc = "Request"]
    #[inline(always)]
    pub fn req_yes(self) -> &'a mut crate::W<REG> {
        self.variant(Req::ReqYes)
    }
}
#[doc = "SRAM Voltage Update Request Acknowledge\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ack {
    #[doc = "0: Not acknowledged"]
    AckNo = 0,
    #[doc = "1: Acknowledged"]
    AckYes = 1,
}
impl From<Ack> for bool {
    #[inline(always)]
    fn from(variant: Ack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ACK` reader - SRAM Voltage Update Request Acknowledge"]
pub type AckR = crate::BitReader<Ack>;
impl AckR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ack {
        match self.bits {
            false => Ack::AckNo,
            true => Ack::AckYes,
        }
    }
    #[doc = "Not acknowledged"]
    #[inline(always)]
    pub fn is_ack_no(&self) -> bool {
        *self == Ack::AckNo
    }
    #[doc = "Acknowledged"]
    #[inline(always)]
    pub fn is_ack_yes(&self) -> bool {
        *self == Ack::AckYes
    }
}
impl R {
    #[doc = "Bits 0:1 - Voltage Select Margin"]
    #[inline(always)]
    pub fn vsm(&self) -> VsmR {
        VsmR::new((self.bits & 3) as u8)
    }
    #[doc = "Bit 30 - SRAM Voltage Update Request"]
    #[inline(always)]
    pub fn req(&self) -> ReqR {
        ReqR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - SRAM Voltage Update Request Acknowledge"]
    #[inline(always)]
    pub fn ack(&self) -> AckR {
        AckR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:1 - Voltage Select Margin"]
    #[inline(always)]
    pub fn vsm(&mut self) -> VsmW<SramctlSpec> {
        VsmW::new(self, 0)
    }
    #[doc = "Bit 30 - SRAM Voltage Update Request"]
    #[inline(always)]
    pub fn req(&mut self) -> ReqW<SramctlSpec> {
        ReqW::new(self, 30)
    }
}
#[doc = "SRAM Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sramctl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sramctl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SramctlSpec;
impl crate::RegisterSpec for SramctlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sramctl::R`](R) reader structure"]
impl crate::Readable for SramctlSpec {}
#[doc = "`write(|w| ..)` method takes [`sramctl::W`](W) writer structure"]
impl crate::Writable for SramctlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SRAMCTL to value 0x01"]
impl crate::Resettable for SramctlSpec {
    const RESET_VALUE: u32 = 0x01;
}

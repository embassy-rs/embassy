#[doc = "Register `ENDPT` reader"]
pub type R = crate::R<EndptSpec>;
#[doc = "Register `ENDPT` writer"]
pub type W = crate::W<EndptSpec>;
#[doc = "Field `EPHSHK` reader - Endpoint Handshaking Enable"]
pub type EphshkR = crate::BitReader;
#[doc = "Field `EPHSHK` writer - Endpoint Handshaking Enable"]
pub type EphshkW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `EPSTALL` reader - Endpoint Stalled"]
pub type EpstallR = crate::BitReader;
#[doc = "Field `EPSTALL` writer - Endpoint Stalled"]
pub type EpstallW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `EPTXEN` reader - Endpoint for TX transfers enable"]
pub type EptxenR = crate::BitReader;
#[doc = "Field `EPTXEN` writer - Endpoint for TX transfers enable"]
pub type EptxenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `EPRXEN` reader - Endpoint for RX transfers enable"]
pub type EprxenR = crate::BitReader;
#[doc = "Field `EPRXEN` writer - Endpoint for RX transfers enable"]
pub type EprxenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Control Transfer Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Epctldis {
    #[doc = "0: Enable"]
    Enable = 0,
    #[doc = "1: Disable"]
    Disable = 1,
}
impl From<Epctldis> for bool {
    #[inline(always)]
    fn from(variant: Epctldis) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EPCTLDIS` reader - Control Transfer Disable"]
pub type EpctldisR = crate::BitReader<Epctldis>;
impl EpctldisR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Epctldis {
        match self.bits {
            false => Epctldis::Enable,
            true => Epctldis::Disable,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Epctldis::Enable
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Epctldis::Disable
    }
}
#[doc = "Field `EPCTLDIS` writer - Control Transfer Disable"]
pub type EpctldisW<'a, REG> = crate::BitWriter<'a, REG, Epctldis>;
impl<'a, REG> EpctldisW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Epctldis::Enable)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Epctldis::Disable)
    }
}
#[doc = "Retry Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Retrydis {
    #[doc = "0: Retried NAK'ed transactions in hardware."]
    Retried = 0,
    #[doc = "1: Do not retry NAK'ed transactions. When a transaction is NAK'ed, the BDT PID field is updated with the NAK PID, and the TOKEN_DNE interrupt becomes 1."]
    DoNotRetried = 1,
}
impl From<Retrydis> for bool {
    #[inline(always)]
    fn from(variant: Retrydis) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RETRYDIS` reader - Retry Disable"]
pub type RetrydisR = crate::BitReader<Retrydis>;
impl RetrydisR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Retrydis {
        match self.bits {
            false => Retrydis::Retried,
            true => Retrydis::DoNotRetried,
        }
    }
    #[doc = "Retried NAK'ed transactions in hardware."]
    #[inline(always)]
    pub fn is_retried(&self) -> bool {
        *self == Retrydis::Retried
    }
    #[doc = "Do not retry NAK'ed transactions. When a transaction is NAK'ed, the BDT PID field is updated with the NAK PID, and the TOKEN_DNE interrupt becomes 1."]
    #[inline(always)]
    pub fn is_do_not_retried(&self) -> bool {
        *self == Retrydis::DoNotRetried
    }
}
#[doc = "Field `RETRYDIS` writer - Retry Disable"]
pub type RetrydisW<'a, REG> = crate::BitWriter<'a, REG, Retrydis>;
impl<'a, REG> RetrydisW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Retried NAK'ed transactions in hardware."]
    #[inline(always)]
    pub fn retried(self) -> &'a mut crate::W<REG> {
        self.variant(Retrydis::Retried)
    }
    #[doc = "Do not retry NAK'ed transactions. When a transaction is NAK'ed, the BDT PID field is updated with the NAK PID, and the TOKEN_DNE interrupt becomes 1."]
    #[inline(always)]
    pub fn do_not_retried(self) -> &'a mut crate::W<REG> {
        self.variant(Retrydis::DoNotRetried)
    }
}
#[doc = "Host Without A Hub\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hostwohub {
    #[doc = "0: Connected using a hub (USBFS generates PRE_PID as required)"]
    LsThruHub = 0,
    #[doc = "1: Connected directly to host without a hub, or was used to attach"]
    LsDirectConnect = 1,
}
impl From<Hostwohub> for bool {
    #[inline(always)]
    fn from(variant: Hostwohub) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HOSTWOHUB` reader - Host Without A Hub"]
pub type HostwohubR = crate::BitReader<Hostwohub>;
impl HostwohubR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hostwohub {
        match self.bits {
            false => Hostwohub::LsThruHub,
            true => Hostwohub::LsDirectConnect,
        }
    }
    #[doc = "Connected using a hub (USBFS generates PRE_PID as required)"]
    #[inline(always)]
    pub fn is_ls_thru_hub(&self) -> bool {
        *self == Hostwohub::LsThruHub
    }
    #[doc = "Connected directly to host without a hub, or was used to attach"]
    #[inline(always)]
    pub fn is_ls_direct_connect(&self) -> bool {
        *self == Hostwohub::LsDirectConnect
    }
}
#[doc = "Field `HOSTWOHUB` writer - Host Without A Hub"]
pub type HostwohubW<'a, REG> = crate::BitWriter<'a, REG, Hostwohub>;
impl<'a, REG> HostwohubW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Connected using a hub (USBFS generates PRE_PID as required)"]
    #[inline(always)]
    pub fn ls_thru_hub(self) -> &'a mut crate::W<REG> {
        self.variant(Hostwohub::LsThruHub)
    }
    #[doc = "Connected directly to host without a hub, or was used to attach"]
    #[inline(always)]
    pub fn ls_direct_connect(self) -> &'a mut crate::W<REG> {
        self.variant(Hostwohub::LsDirectConnect)
    }
}
impl R {
    #[doc = "Bit 0 - Endpoint Handshaking Enable"]
    #[inline(always)]
    pub fn ephshk(&self) -> EphshkR {
        EphshkR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Endpoint Stalled"]
    #[inline(always)]
    pub fn epstall(&self) -> EpstallR {
        EpstallR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Endpoint for TX transfers enable"]
    #[inline(always)]
    pub fn eptxen(&self) -> EptxenR {
        EptxenR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Endpoint for RX transfers enable"]
    #[inline(always)]
    pub fn eprxen(&self) -> EprxenR {
        EprxenR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Control Transfer Disable"]
    #[inline(always)]
    pub fn epctldis(&self) -> EpctldisR {
        EpctldisR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 6 - Retry Disable"]
    #[inline(always)]
    pub fn retrydis(&self) -> RetrydisR {
        RetrydisR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Host Without A Hub"]
    #[inline(always)]
    pub fn hostwohub(&self) -> HostwohubR {
        HostwohubR::new(((self.bits >> 7) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Endpoint Handshaking Enable"]
    #[inline(always)]
    pub fn ephshk(&mut self) -> EphshkW<EndptSpec> {
        EphshkW::new(self, 0)
    }
    #[doc = "Bit 1 - Endpoint Stalled"]
    #[inline(always)]
    pub fn epstall(&mut self) -> EpstallW<EndptSpec> {
        EpstallW::new(self, 1)
    }
    #[doc = "Bit 2 - Endpoint for TX transfers enable"]
    #[inline(always)]
    pub fn eptxen(&mut self) -> EptxenW<EndptSpec> {
        EptxenW::new(self, 2)
    }
    #[doc = "Bit 3 - Endpoint for RX transfers enable"]
    #[inline(always)]
    pub fn eprxen(&mut self) -> EprxenW<EndptSpec> {
        EprxenW::new(self, 3)
    }
    #[doc = "Bit 4 - Control Transfer Disable"]
    #[inline(always)]
    pub fn epctldis(&mut self) -> EpctldisW<EndptSpec> {
        EpctldisW::new(self, 4)
    }
    #[doc = "Bit 6 - Retry Disable"]
    #[inline(always)]
    pub fn retrydis(&mut self) -> RetrydisW<EndptSpec> {
        RetrydisW::new(self, 6)
    }
    #[doc = "Bit 7 - Host Without A Hub"]
    #[inline(always)]
    pub fn hostwohub(&mut self) -> HostwohubW<EndptSpec> {
        HostwohubW::new(self, 7)
    }
}
#[doc = "Endpoint Control\n\nYou can [`read`](crate::Reg::read) this register and get [`endpt::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`endpt::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct EndptSpec;
impl crate::RegisterSpec for EndptSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`endpt::R`](R) reader structure"]
impl crate::Readable for EndptSpec {}
#[doc = "`write(|w| ..)` method takes [`endpt::W`](W) writer structure"]
impl crate::Writable for EndptSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ENDPT to value 0"]
impl crate::Resettable for EndptSpec {}

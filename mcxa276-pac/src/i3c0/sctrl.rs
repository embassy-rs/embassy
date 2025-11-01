#[doc = "Register `SCTRL` reader"]
pub type R = crate::R<SctrlSpec>;
#[doc = "Register `SCTRL` writer"]
pub type W = crate::W<SctrlSpec>;
#[doc = "Event\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Event {
    #[doc = "0: NORMAL_MODE"]
    NormalMode = 0,
    #[doc = "1: IBI"]
    Ibi = 1,
    #[doc = "2: CONTROLLER_REQUEST"]
    MasterRequest = 2,
    #[doc = "3: HOT_JOIN_REQUEST"]
    HotJoinRequest = 3,
}
impl From<Event> for u8 {
    #[inline(always)]
    fn from(variant: Event) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Event {
    type Ux = u8;
}
impl crate::IsEnum for Event {}
#[doc = "Field `EVENT` reader - Event"]
pub type EventR = crate::FieldReader<Event>;
impl EventR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Event {
        match self.bits {
            0 => Event::NormalMode,
            1 => Event::Ibi,
            2 => Event::MasterRequest,
            3 => Event::HotJoinRequest,
            _ => unreachable!(),
        }
    }
    #[doc = "NORMAL_MODE"]
    #[inline(always)]
    pub fn is_normal_mode(&self) -> bool {
        *self == Event::NormalMode
    }
    #[doc = "IBI"]
    #[inline(always)]
    pub fn is_ibi(&self) -> bool {
        *self == Event::Ibi
    }
    #[doc = "CONTROLLER_REQUEST"]
    #[inline(always)]
    pub fn is_master_request(&self) -> bool {
        *self == Event::MasterRequest
    }
    #[doc = "HOT_JOIN_REQUEST"]
    #[inline(always)]
    pub fn is_hot_join_request(&self) -> bool {
        *self == Event::HotJoinRequest
    }
}
#[doc = "Field `EVENT` writer - Event"]
pub type EventW<'a, REG> = crate::FieldWriter<'a, REG, 2, Event, crate::Safe>;
impl<'a, REG> EventW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "NORMAL_MODE"]
    #[inline(always)]
    pub fn normal_mode(self) -> &'a mut crate::W<REG> {
        self.variant(Event::NormalMode)
    }
    #[doc = "IBI"]
    #[inline(always)]
    pub fn ibi(self) -> &'a mut crate::W<REG> {
        self.variant(Event::Ibi)
    }
    #[doc = "CONTROLLER_REQUEST"]
    #[inline(always)]
    pub fn master_request(self) -> &'a mut crate::W<REG> {
        self.variant(Event::MasterRequest)
    }
    #[doc = "HOT_JOIN_REQUEST"]
    #[inline(always)]
    pub fn hot_join_request(self) -> &'a mut crate::W<REG> {
        self.variant(Event::HotJoinRequest)
    }
}
#[doc = "Extended Data\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Extdata {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Extdata> for bool {
    #[inline(always)]
    fn from(variant: Extdata) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EXTDATA` reader - Extended Data"]
pub type ExtdataR = crate::BitReader<Extdata>;
impl ExtdataR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Extdata {
        match self.bits {
            false => Extdata::Disable,
            true => Extdata::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Extdata::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Extdata::Enable
    }
}
#[doc = "Field `EXTDATA` writer - Extended Data"]
pub type ExtdataW<'a, REG> = crate::BitWriter<'a, REG, Extdata>;
impl<'a, REG> ExtdataW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Extdata::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Extdata::Enable)
    }
}
#[doc = "Field `IBIDATA` reader - In-Band Interrupt Data"]
pub type IbidataR = crate::FieldReader;
#[doc = "Field `IBIDATA` writer - In-Band Interrupt Data"]
pub type IbidataW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Field `PENDINT` reader - Pending Interrupt"]
pub type PendintR = crate::FieldReader;
#[doc = "Field `PENDINT` writer - Pending Interrupt"]
pub type PendintW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `ACTSTATE` reader - Activity State of Target"]
pub type ActstateR = crate::FieldReader;
#[doc = "Field `ACTSTATE` writer - Activity State of Target"]
pub type ActstateW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `VENDINFO` reader - Vendor Information"]
pub type VendinfoR = crate::FieldReader;
#[doc = "Field `VENDINFO` writer - Vendor Information"]
pub type VendinfoW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bits 0:1 - Event"]
    #[inline(always)]
    pub fn event(&self) -> EventR {
        EventR::new((self.bits & 3) as u8)
    }
    #[doc = "Bit 3 - Extended Data"]
    #[inline(always)]
    pub fn extdata(&self) -> ExtdataR {
        ExtdataR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bits 8:15 - In-Band Interrupt Data"]
    #[inline(always)]
    pub fn ibidata(&self) -> IbidataR {
        IbidataR::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:19 - Pending Interrupt"]
    #[inline(always)]
    pub fn pendint(&self) -> PendintR {
        PendintR::new(((self.bits >> 16) & 0x0f) as u8)
    }
    #[doc = "Bits 20:21 - Activity State of Target"]
    #[inline(always)]
    pub fn actstate(&self) -> ActstateR {
        ActstateR::new(((self.bits >> 20) & 3) as u8)
    }
    #[doc = "Bits 24:31 - Vendor Information"]
    #[inline(always)]
    pub fn vendinfo(&self) -> VendinfoR {
        VendinfoR::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - Event"]
    #[inline(always)]
    pub fn event(&mut self) -> EventW<SctrlSpec> {
        EventW::new(self, 0)
    }
    #[doc = "Bit 3 - Extended Data"]
    #[inline(always)]
    pub fn extdata(&mut self) -> ExtdataW<SctrlSpec> {
        ExtdataW::new(self, 3)
    }
    #[doc = "Bits 8:15 - In-Band Interrupt Data"]
    #[inline(always)]
    pub fn ibidata(&mut self) -> IbidataW<SctrlSpec> {
        IbidataW::new(self, 8)
    }
    #[doc = "Bits 16:19 - Pending Interrupt"]
    #[inline(always)]
    pub fn pendint(&mut self) -> PendintW<SctrlSpec> {
        PendintW::new(self, 16)
    }
    #[doc = "Bits 20:21 - Activity State of Target"]
    #[inline(always)]
    pub fn actstate(&mut self) -> ActstateW<SctrlSpec> {
        ActstateW::new(self, 20)
    }
    #[doc = "Bits 24:31 - Vendor Information"]
    #[inline(always)]
    pub fn vendinfo(&mut self) -> VendinfoW<SctrlSpec> {
        VendinfoW::new(self, 24)
    }
}
#[doc = "Target Control\n\nYou can [`read`](crate::Reg::read) this register and get [`sctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SctrlSpec;
impl crate::RegisterSpec for SctrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sctrl::R`](R) reader structure"]
impl crate::Readable for SctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`sctrl::W`](W) writer structure"]
impl crate::Writable for SctrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SCTRL to value 0"]
impl crate::Resettable for SctrlSpec {}

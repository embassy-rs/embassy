#[doc = "Register `MIER` reader"]
pub type R = crate::R<MierSpec>;
#[doc = "Register `MIER` writer"]
pub type W = crate::W<MierSpec>;
#[doc = "Transmit Data Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Tdie> for bool {
    #[inline(always)]
    fn from(variant: Tdie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TDIE` reader - Transmit Data Interrupt Enable"]
pub type TdieR = crate::BitReader<Tdie>;
impl TdieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tdie {
        match self.bits {
            false => Tdie::Disabled,
            true => Tdie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Tdie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Tdie::Enabled
    }
}
#[doc = "Field `TDIE` writer - Transmit Data Interrupt Enable"]
pub type TdieW<'a, REG> = crate::BitWriter<'a, REG, Tdie>;
impl<'a, REG> TdieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tdie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Tdie::Enabled)
    }
}
#[doc = "Receive Data Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Rdie> for bool {
    #[inline(always)]
    fn from(variant: Rdie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RDIE` reader - Receive Data Interrupt Enable"]
pub type RdieR = crate::BitReader<Rdie>;
impl RdieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rdie {
        match self.bits {
            false => Rdie::Disabled,
            true => Rdie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Rdie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Rdie::Enabled
    }
}
#[doc = "Field `RDIE` writer - Receive Data Interrupt Enable"]
pub type RdieW<'a, REG> = crate::BitWriter<'a, REG, Rdie>;
impl<'a, REG> RdieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rdie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Rdie::Enabled)
    }
}
#[doc = "End Packet Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Epie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Epie> for bool {
    #[inline(always)]
    fn from(variant: Epie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `EPIE` reader - End Packet Interrupt Enable"]
pub type EpieR = crate::BitReader<Epie>;
impl EpieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Epie {
        match self.bits {
            false => Epie::Disabled,
            true => Epie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Epie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Epie::Enabled
    }
}
#[doc = "Field `EPIE` writer - End Packet Interrupt Enable"]
pub type EpieW<'a, REG> = crate::BitWriter<'a, REG, Epie>;
impl<'a, REG> EpieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Epie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Epie::Enabled)
    }
}
#[doc = "Stop Detect Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sdie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Sdie> for bool {
    #[inline(always)]
    fn from(variant: Sdie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SDIE` reader - Stop Detect Interrupt Enable"]
pub type SdieR = crate::BitReader<Sdie>;
impl SdieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Sdie {
        match self.bits {
            false => Sdie::Disabled,
            true => Sdie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Sdie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Sdie::Enabled
    }
}
#[doc = "Field `SDIE` writer - Stop Detect Interrupt Enable"]
pub type SdieW<'a, REG> = crate::BitWriter<'a, REG, Sdie>;
impl<'a, REG> SdieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sdie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Sdie::Enabled)
    }
}
#[doc = "NACK Detect Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ndie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Ndie> for bool {
    #[inline(always)]
    fn from(variant: Ndie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NDIE` reader - NACK Detect Interrupt Enable"]
pub type NdieR = crate::BitReader<Ndie>;
impl NdieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Ndie {
        match self.bits {
            false => Ndie::Disabled,
            true => Ndie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Ndie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Ndie::Enabled
    }
}
#[doc = "Field `NDIE` writer - NACK Detect Interrupt Enable"]
pub type NdieW<'a, REG> = crate::BitWriter<'a, REG, Ndie>;
impl<'a, REG> NdieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ndie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Ndie::Enabled)
    }
}
#[doc = "Arbitration Lost Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Alie> for bool {
    #[inline(always)]
    fn from(variant: Alie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ALIE` reader - Arbitration Lost Interrupt Enable"]
pub type AlieR = crate::BitReader<Alie>;
impl AlieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Alie {
        match self.bits {
            false => Alie::Disabled,
            true => Alie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Alie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Alie::Enabled
    }
}
#[doc = "Field `ALIE` writer - Arbitration Lost Interrupt Enable"]
pub type AlieW<'a, REG> = crate::BitWriter<'a, REG, Alie>;
impl<'a, REG> AlieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Alie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Alie::Enabled)
    }
}
#[doc = "FIFO Error Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Feie> for bool {
    #[inline(always)]
    fn from(variant: Feie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FEIE` reader - FIFO Error Interrupt Enable"]
pub type FeieR = crate::BitReader<Feie>;
impl FeieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Feie {
        match self.bits {
            false => Feie::Disabled,
            true => Feie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Feie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Feie::Enabled
    }
}
#[doc = "Field `FEIE` writer - FIFO Error Interrupt Enable"]
pub type FeieW<'a, REG> = crate::BitWriter<'a, REG, Feie>;
impl<'a, REG> FeieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Feie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Feie::Enabled)
    }
}
#[doc = "Pin Low Timeout Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pltie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Pltie> for bool {
    #[inline(always)]
    fn from(variant: Pltie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PLTIE` reader - Pin Low Timeout Interrupt Enable"]
pub type PltieR = crate::BitReader<Pltie>;
impl PltieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pltie {
        match self.bits {
            false => Pltie::Disabled,
            true => Pltie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Pltie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Pltie::Enabled
    }
}
#[doc = "Field `PLTIE` writer - Pin Low Timeout Interrupt Enable"]
pub type PltieW<'a, REG> = crate::BitWriter<'a, REG, Pltie>;
impl<'a, REG> PltieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Pltie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Pltie::Enabled)
    }
}
#[doc = "Data Match Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dmie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Dmie> for bool {
    #[inline(always)]
    fn from(variant: Dmie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DMIE` reader - Data Match Interrupt Enable"]
pub type DmieR = crate::BitReader<Dmie>;
impl DmieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Dmie {
        match self.bits {
            false => Dmie::Disabled,
            true => Dmie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Dmie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Dmie::Enabled
    }
}
#[doc = "Field `DMIE` writer - Data Match Interrupt Enable"]
pub type DmieW<'a, REG> = crate::BitWriter<'a, REG, Dmie>;
impl<'a, REG> DmieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dmie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Dmie::Enabled)
    }
}
#[doc = "Start Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stie {
    #[doc = "0: Disable"]
    Disabled = 0,
    #[doc = "1: Enable"]
    Enabled = 1,
}
impl From<Stie> for bool {
    #[inline(always)]
    fn from(variant: Stie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `STIE` reader - Start Interrupt Enable"]
pub type StieR = crate::BitReader<Stie>;
impl StieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Stie {
        match self.bits {
            false => Stie::Disabled,
            true => Stie::Enabled,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Stie::Disabled
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Stie::Enabled
    }
}
#[doc = "Field `STIE` writer - Start Interrupt Enable"]
pub type StieW<'a, REG> = crate::BitWriter<'a, REG, Stie>;
impl<'a, REG> StieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Stie::Disabled)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Stie::Enabled)
    }
}
impl R {
    #[doc = "Bit 0 - Transmit Data Interrupt Enable"]
    #[inline(always)]
    pub fn tdie(&self) -> TdieR {
        TdieR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Receive Data Interrupt Enable"]
    #[inline(always)]
    pub fn rdie(&self) -> RdieR {
        RdieR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 8 - End Packet Interrupt Enable"]
    #[inline(always)]
    pub fn epie(&self) -> EpieR {
        EpieR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Stop Detect Interrupt Enable"]
    #[inline(always)]
    pub fn sdie(&self) -> SdieR {
        SdieR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - NACK Detect Interrupt Enable"]
    #[inline(always)]
    pub fn ndie(&self) -> NdieR {
        NdieR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Arbitration Lost Interrupt Enable"]
    #[inline(always)]
    pub fn alie(&self) -> AlieR {
        AlieR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - FIFO Error Interrupt Enable"]
    #[inline(always)]
    pub fn feie(&self) -> FeieR {
        FeieR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Pin Low Timeout Interrupt Enable"]
    #[inline(always)]
    pub fn pltie(&self) -> PltieR {
        PltieR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 14 - Data Match Interrupt Enable"]
    #[inline(always)]
    pub fn dmie(&self) -> DmieR {
        DmieR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Start Interrupt Enable"]
    #[inline(always)]
    pub fn stie(&self) -> StieR {
        StieR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Transmit Data Interrupt Enable"]
    #[inline(always)]
    pub fn tdie(&mut self) -> TdieW<MierSpec> {
        TdieW::new(self, 0)
    }
    #[doc = "Bit 1 - Receive Data Interrupt Enable"]
    #[inline(always)]
    pub fn rdie(&mut self) -> RdieW<MierSpec> {
        RdieW::new(self, 1)
    }
    #[doc = "Bit 8 - End Packet Interrupt Enable"]
    #[inline(always)]
    pub fn epie(&mut self) -> EpieW<MierSpec> {
        EpieW::new(self, 8)
    }
    #[doc = "Bit 9 - Stop Detect Interrupt Enable"]
    #[inline(always)]
    pub fn sdie(&mut self) -> SdieW<MierSpec> {
        SdieW::new(self, 9)
    }
    #[doc = "Bit 10 - NACK Detect Interrupt Enable"]
    #[inline(always)]
    pub fn ndie(&mut self) -> NdieW<MierSpec> {
        NdieW::new(self, 10)
    }
    #[doc = "Bit 11 - Arbitration Lost Interrupt Enable"]
    #[inline(always)]
    pub fn alie(&mut self) -> AlieW<MierSpec> {
        AlieW::new(self, 11)
    }
    #[doc = "Bit 12 - FIFO Error Interrupt Enable"]
    #[inline(always)]
    pub fn feie(&mut self) -> FeieW<MierSpec> {
        FeieW::new(self, 12)
    }
    #[doc = "Bit 13 - Pin Low Timeout Interrupt Enable"]
    #[inline(always)]
    pub fn pltie(&mut self) -> PltieW<MierSpec> {
        PltieW::new(self, 13)
    }
    #[doc = "Bit 14 - Data Match Interrupt Enable"]
    #[inline(always)]
    pub fn dmie(&mut self) -> DmieW<MierSpec> {
        DmieW::new(self, 14)
    }
    #[doc = "Bit 15 - Start Interrupt Enable"]
    #[inline(always)]
    pub fn stie(&mut self) -> StieW<MierSpec> {
        StieW::new(self, 15)
    }
}
#[doc = "Controller Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`mier::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`mier::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct MierSpec;
impl crate::RegisterSpec for MierSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`mier::R`](R) reader structure"]
impl crate::Readable for MierSpec {}
#[doc = "`write(|w| ..)` method takes [`mier::W`](W) writer structure"]
impl crate::Writable for MierSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets MIER to value 0"]
impl crate::Resettable for MierSpec {}

#[doc = "Register `IER` reader"]
pub type R = crate::R<IerSpec>;
#[doc = "Register `IER` writer"]
pub type W = crate::W<IerSpec>;
#[doc = "Transmit Data Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
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
            false => Tdie::Disable,
            true => Tdie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tdie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tdie::Enable
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
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tdie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tdie::Enable)
    }
}
#[doc = "Receive Data Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rdie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
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
            false => Rdie::Disable,
            true => Rdie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Rdie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Rdie::Enable
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
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Rdie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Rdie::Enable)
    }
}
#[doc = "Word Complete Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wcie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Wcie> for bool {
    #[inline(always)]
    fn from(variant: Wcie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `WCIE` reader - Word Complete Interrupt Enable"]
pub type WcieR = crate::BitReader<Wcie>;
impl WcieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Wcie {
        match self.bits {
            false => Wcie::Disable,
            true => Wcie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Wcie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Wcie::Enable
    }
}
#[doc = "Field `WCIE` writer - Word Complete Interrupt Enable"]
pub type WcieW<'a, REG> = crate::BitWriter<'a, REG, Wcie>;
impl<'a, REG> WcieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Wcie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Wcie::Enable)
    }
}
#[doc = "Frame Complete Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fcie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Fcie> for bool {
    #[inline(always)]
    fn from(variant: Fcie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FCIE` reader - Frame Complete Interrupt Enable"]
pub type FcieR = crate::BitReader<Fcie>;
impl FcieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fcie {
        match self.bits {
            false => Fcie::Disable,
            true => Fcie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Fcie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Fcie::Enable
    }
}
#[doc = "Field `FCIE` writer - Frame Complete Interrupt Enable"]
pub type FcieW<'a, REG> = crate::BitWriter<'a, REG, Fcie>;
impl<'a, REG> FcieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Fcie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Fcie::Enable)
    }
}
#[doc = "Transfer Complete Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tcie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Tcie> for bool {
    #[inline(always)]
    fn from(variant: Tcie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TCIE` reader - Transfer Complete Interrupt Enable"]
pub type TcieR = crate::BitReader<Tcie>;
impl TcieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tcie {
        match self.bits {
            false => Tcie::Disable,
            true => Tcie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tcie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tcie::Enable
    }
}
#[doc = "Field `TCIE` writer - Transfer Complete Interrupt Enable"]
pub type TcieW<'a, REG> = crate::BitWriter<'a, REG, Tcie>;
impl<'a, REG> TcieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tcie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tcie::Enable)
    }
}
#[doc = "Transmit Error Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Teie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Teie> for bool {
    #[inline(always)]
    fn from(variant: Teie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TEIE` reader - Transmit Error Interrupt Enable"]
pub type TeieR = crate::BitReader<Teie>;
impl TeieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Teie {
        match self.bits {
            false => Teie::Disable,
            true => Teie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Teie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Teie::Enable
    }
}
#[doc = "Field `TEIE` writer - Transmit Error Interrupt Enable"]
pub type TeieW<'a, REG> = crate::BitWriter<'a, REG, Teie>;
impl<'a, REG> TeieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Teie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Teie::Enable)
    }
}
#[doc = "Receive Error Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Reie> for bool {
    #[inline(always)]
    fn from(variant: Reie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `REIE` reader - Receive Error Interrupt Enable"]
pub type ReieR = crate::BitReader<Reie>;
impl ReieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Reie {
        match self.bits {
            false => Reie::Disable,
            true => Reie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Reie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Reie::Enable
    }
}
#[doc = "Field `REIE` writer - Receive Error Interrupt Enable"]
pub type ReieW<'a, REG> = crate::BitWriter<'a, REG, Reie>;
impl<'a, REG> ReieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Reie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Reie::Enable)
    }
}
#[doc = "Data Match Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dmie {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
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
            false => Dmie::Disable,
            true => Dmie::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Dmie::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Dmie::Enable
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
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Dmie::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Dmie::Enable)
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
    #[doc = "Bit 8 - Word Complete Interrupt Enable"]
    #[inline(always)]
    pub fn wcie(&self) -> WcieR {
        WcieR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - Frame Complete Interrupt Enable"]
    #[inline(always)]
    pub fn fcie(&self) -> FcieR {
        FcieR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - Transfer Complete Interrupt Enable"]
    #[inline(always)]
    pub fn tcie(&self) -> TcieR {
        TcieR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - Transmit Error Interrupt Enable"]
    #[inline(always)]
    pub fn teie(&self) -> TeieR {
        TeieR::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bit 12 - Receive Error Interrupt Enable"]
    #[inline(always)]
    pub fn reie(&self) -> ReieR {
        ReieR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - Data Match Interrupt Enable"]
    #[inline(always)]
    pub fn dmie(&self) -> DmieR {
        DmieR::new(((self.bits >> 13) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Transmit Data Interrupt Enable"]
    #[inline(always)]
    pub fn tdie(&mut self) -> TdieW<IerSpec> {
        TdieW::new(self, 0)
    }
    #[doc = "Bit 1 - Receive Data Interrupt Enable"]
    #[inline(always)]
    pub fn rdie(&mut self) -> RdieW<IerSpec> {
        RdieW::new(self, 1)
    }
    #[doc = "Bit 8 - Word Complete Interrupt Enable"]
    #[inline(always)]
    pub fn wcie(&mut self) -> WcieW<IerSpec> {
        WcieW::new(self, 8)
    }
    #[doc = "Bit 9 - Frame Complete Interrupt Enable"]
    #[inline(always)]
    pub fn fcie(&mut self) -> FcieW<IerSpec> {
        FcieW::new(self, 9)
    }
    #[doc = "Bit 10 - Transfer Complete Interrupt Enable"]
    #[inline(always)]
    pub fn tcie(&mut self) -> TcieW<IerSpec> {
        TcieW::new(self, 10)
    }
    #[doc = "Bit 11 - Transmit Error Interrupt Enable"]
    #[inline(always)]
    pub fn teie(&mut self) -> TeieW<IerSpec> {
        TeieW::new(self, 11)
    }
    #[doc = "Bit 12 - Receive Error Interrupt Enable"]
    #[inline(always)]
    pub fn reie(&mut self) -> ReieW<IerSpec> {
        ReieW::new(self, 12)
    }
    #[doc = "Bit 13 - Data Match Interrupt Enable"]
    #[inline(always)]
    pub fn dmie(&mut self) -> DmieW<IerSpec> {
        DmieW::new(self, 13)
    }
}
#[doc = "Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ier::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ier::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IerSpec;
impl crate::RegisterSpec for IerSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ier::R`](R) reader structure"]
impl crate::Readable for IerSpec {}
#[doc = "`write(|w| ..)` method takes [`ier::W`](W) writer structure"]
impl crate::Writable for IerSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets IER to value 0"]
impl crate::Resettable for IerSpec {}

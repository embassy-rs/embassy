#[doc = "Register `IER` reader"]
pub type R = crate::R<IerSpec>;
#[doc = "Register `IER` writer"]
pub type W = crate::W<IerSpec>;
#[doc = "Comparator Flag Rising Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CfrIe {
    #[doc = "0: Disables the comparator flag rising interrupt."]
    Disable = 0,
    #[doc = "1: Enables the comparator flag rising interrupt when CFR is set."]
    Enable = 1,
}
impl From<CfrIe> for bool {
    #[inline(always)]
    fn from(variant: CfrIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CFR_IE` reader - Comparator Flag Rising Interrupt Enable"]
pub type CfrIeR = crate::BitReader<CfrIe>;
impl CfrIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CfrIe {
        match self.bits {
            false => CfrIe::Disable,
            true => CfrIe::Enable,
        }
    }
    #[doc = "Disables the comparator flag rising interrupt."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == CfrIe::Disable
    }
    #[doc = "Enables the comparator flag rising interrupt when CFR is set."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == CfrIe::Enable
    }
}
#[doc = "Field `CFR_IE` writer - Comparator Flag Rising Interrupt Enable"]
pub type CfrIeW<'a, REG> = crate::BitWriter<'a, REG, CfrIe>;
impl<'a, REG> CfrIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables the comparator flag rising interrupt."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(CfrIe::Disable)
    }
    #[doc = "Enables the comparator flag rising interrupt when CFR is set."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(CfrIe::Enable)
    }
}
#[doc = "Comparator Flag Falling Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CffIe {
    #[doc = "0: Disables the comparator flag falling interrupt."]
    Disable = 0,
    #[doc = "1: Enables the comparator flag falling interrupt when CFF is set."]
    Enable = 1,
}
impl From<CffIe> for bool {
    #[inline(always)]
    fn from(variant: CffIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CFF_IE` reader - Comparator Flag Falling Interrupt Enable"]
pub type CffIeR = crate::BitReader<CffIe>;
impl CffIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CffIe {
        match self.bits {
            false => CffIe::Disable,
            true => CffIe::Enable,
        }
    }
    #[doc = "Disables the comparator flag falling interrupt."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == CffIe::Disable
    }
    #[doc = "Enables the comparator flag falling interrupt when CFF is set."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == CffIe::Enable
    }
}
#[doc = "Field `CFF_IE` writer - Comparator Flag Falling Interrupt Enable"]
pub type CffIeW<'a, REG> = crate::BitWriter<'a, REG, CffIe>;
impl<'a, REG> CffIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables the comparator flag falling interrupt."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(CffIe::Disable)
    }
    #[doc = "Enables the comparator flag falling interrupt when CFF is set."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(CffIe::Enable)
    }
}
#[doc = "Round-Robin Flag Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RrfIe {
    #[doc = "0: Disables the round-robin flag interrupt."]
    Disable = 0,
    #[doc = "1: Enables the round-robin flag interrupt when the comparison result changes for a given channel."]
    Enable = 1,
}
impl From<RrfIe> for bool {
    #[inline(always)]
    fn from(variant: RrfIe) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RRF_IE` reader - Round-Robin Flag Interrupt Enable"]
pub type RrfIeR = crate::BitReader<RrfIe>;
impl RrfIeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> RrfIe {
        match self.bits {
            false => RrfIe::Disable,
            true => RrfIe::Enable,
        }
    }
    #[doc = "Disables the round-robin flag interrupt."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == RrfIe::Disable
    }
    #[doc = "Enables the round-robin flag interrupt when the comparison result changes for a given channel."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == RrfIe::Enable
    }
}
#[doc = "Field `RRF_IE` writer - Round-Robin Flag Interrupt Enable"]
pub type RrfIeW<'a, REG> = crate::BitWriter<'a, REG, RrfIe>;
impl<'a, REG> RrfIeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables the round-robin flag interrupt."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(RrfIe::Disable)
    }
    #[doc = "Enables the round-robin flag interrupt when the comparison result changes for a given channel."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(RrfIe::Enable)
    }
}
impl R {
    #[doc = "Bit 0 - Comparator Flag Rising Interrupt Enable"]
    #[inline(always)]
    pub fn cfr_ie(&self) -> CfrIeR {
        CfrIeR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Comparator Flag Falling Interrupt Enable"]
    #[inline(always)]
    pub fn cff_ie(&self) -> CffIeR {
        CffIeR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Round-Robin Flag Interrupt Enable"]
    #[inline(always)]
    pub fn rrf_ie(&self) -> RrfIeR {
        RrfIeR::new(((self.bits >> 2) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Comparator Flag Rising Interrupt Enable"]
    #[inline(always)]
    pub fn cfr_ie(&mut self) -> CfrIeW<IerSpec> {
        CfrIeW::new(self, 0)
    }
    #[doc = "Bit 1 - Comparator Flag Falling Interrupt Enable"]
    #[inline(always)]
    pub fn cff_ie(&mut self) -> CffIeW<IerSpec> {
        CffIeW::new(self, 1)
    }
    #[doc = "Bit 2 - Round-Robin Flag Interrupt Enable"]
    #[inline(always)]
    pub fn rrf_ie(&mut self) -> RrfIeW<IerSpec> {
        RrfIeW::new(self, 2)
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

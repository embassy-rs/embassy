#[doc = "Register `SCONFIG` reader"]
pub type R = crate::R<SconfigSpec>;
#[doc = "Register `SCONFIG` writer"]
pub type W = crate::W<SconfigSpec>;
#[doc = "Target Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slvena {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Slvena> for bool {
    #[inline(always)]
    fn from(variant: Slvena) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SLVENA` reader - Target Enable"]
pub type SlvenaR = crate::BitReader<Slvena>;
impl SlvenaR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Slvena {
        match self.bits {
            false => Slvena::Disable,
            true => Slvena::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Slvena::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Slvena::Enable
    }
}
#[doc = "Field `SLVENA` writer - Target Enable"]
pub type SlvenaW<'a, REG> = crate::BitWriter<'a, REG, Slvena>;
impl<'a, REG> SlvenaW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Slvena::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Slvena::Enable)
    }
}
#[doc = "Not Acknowledge\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nack {
    #[doc = "0: Always disable NACK mode"]
    Disable = 0,
    #[doc = "1: Always enable NACK mode (works normally)"]
    Enable = 1,
}
impl From<Nack> for bool {
    #[inline(always)]
    fn from(variant: Nack) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NACK` reader - Not Acknowledge"]
pub type NackR = crate::BitReader<Nack>;
impl NackR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nack {
        match self.bits {
            false => Nack::Disable,
            true => Nack::Enable,
        }
    }
    #[doc = "Always disable NACK mode"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Nack::Disable
    }
    #[doc = "Always enable NACK mode (works normally)"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Nack::Enable
    }
}
#[doc = "Field `NACK` writer - Not Acknowledge"]
pub type NackW<'a, REG> = crate::BitWriter<'a, REG, Nack>;
impl<'a, REG> NackW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Always disable NACK mode"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Nack::Disable)
    }
    #[doc = "Always enable NACK mode (works normally)"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Nack::Enable)
    }
}
#[doc = "Match Start or Stop\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Matchss {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Matchss> for bool {
    #[inline(always)]
    fn from(variant: Matchss) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `MATCHSS` reader - Match Start or Stop"]
pub type MatchssR = crate::BitReader<Matchss>;
impl MatchssR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Matchss {
        match self.bits {
            false => Matchss::Disable,
            true => Matchss::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Matchss::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Matchss::Enable
    }
}
#[doc = "Field `MATCHSS` writer - Match Start or Stop"]
pub type MatchssW<'a, REG> = crate::BitWriter<'a, REG, Matchss>;
impl<'a, REG> MatchssW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Matchss::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Matchss::Enable)
    }
}
#[doc = "Ignore TE0 or TE1 Errors\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum S0ignore {
    #[doc = "0: Do not ignore TE0 or TE1 errors"]
    Disable = 0,
    #[doc = "1: Ignore TE0 or TE1 errors"]
    Enable = 1,
}
impl From<S0ignore> for bool {
    #[inline(always)]
    fn from(variant: S0ignore) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `S0IGNORE` reader - Ignore TE0 or TE1 Errors"]
pub type S0ignoreR = crate::BitReader<S0ignore>;
impl S0ignoreR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> S0ignore {
        match self.bits {
            false => S0ignore::Disable,
            true => S0ignore::Enable,
        }
    }
    #[doc = "Do not ignore TE0 or TE1 errors"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == S0ignore::Disable
    }
    #[doc = "Ignore TE0 or TE1 errors"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == S0ignore::Enable
    }
}
#[doc = "Field `S0IGNORE` writer - Ignore TE0 or TE1 Errors"]
pub type S0ignoreW<'a, REG> = crate::BitWriter<'a, REG, S0ignore>;
impl<'a, REG> S0ignoreW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Do not ignore TE0 or TE1 errors"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(S0ignore::Disable)
    }
    #[doc = "Ignore TE0 or TE1 errors"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(S0ignore::Enable)
    }
}
#[doc = "HDR OK\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hdrok {
    #[doc = "0: Disable HDR OK"]
    Disable = 0,
    #[doc = "1: Enable HDR OK"]
    Enable = 1,
}
impl From<Hdrok> for bool {
    #[inline(always)]
    fn from(variant: Hdrok) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `HDROK` reader - HDR OK"]
pub type HdrokR = crate::BitReader<Hdrok>;
impl HdrokR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hdrok {
        match self.bits {
            false => Hdrok::Disable,
            true => Hdrok::Enable,
        }
    }
    #[doc = "Disable HDR OK"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Hdrok::Disable
    }
    #[doc = "Enable HDR OK"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Hdrok::Enable
    }
}
#[doc = "Field `HDROK` writer - HDR OK"]
pub type HdrokW<'a, REG> = crate::BitWriter<'a, REG, Hdrok>;
impl<'a, REG> HdrokW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable HDR OK"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Hdrok::Disable)
    }
    #[doc = "Enable HDR OK"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Hdrok::Enable)
    }
}
#[doc = "Offline\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Offline {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Offline> for bool {
    #[inline(always)]
    fn from(variant: Offline) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `OFFLINE` reader - Offline"]
pub type OfflineR = crate::BitReader<Offline>;
impl OfflineR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Offline {
        match self.bits {
            false => Offline::Disable,
            true => Offline::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Offline::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Offline::Enable
    }
}
#[doc = "Field `OFFLINE` writer - Offline"]
pub type OfflineW<'a, REG> = crate::BitWriter<'a, REG, Offline>;
impl<'a, REG> OfflineW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Offline::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Offline::Enable)
    }
}
#[doc = "Field `BAMATCH` reader - Bus Available Match"]
pub type BamatchR = crate::FieldReader;
#[doc = "Field `BAMATCH` writer - Bus Available Match"]
pub type BamatchW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Field `SADDR` reader - Static Address"]
pub type SaddrR = crate::FieldReader;
#[doc = "Field `SADDR` writer - Static Address"]
pub type SaddrW<'a, REG> = crate::FieldWriter<'a, REG, 7>;
impl R {
    #[doc = "Bit 0 - Target Enable"]
    #[inline(always)]
    pub fn slvena(&self) -> SlvenaR {
        SlvenaR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Not Acknowledge"]
    #[inline(always)]
    pub fn nack(&self) -> NackR {
        NackR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Match Start or Stop"]
    #[inline(always)]
    pub fn matchss(&self) -> MatchssR {
        MatchssR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Ignore TE0 or TE1 Errors"]
    #[inline(always)]
    pub fn s0ignore(&self) -> S0ignoreR {
        S0ignoreR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - HDR OK"]
    #[inline(always)]
    pub fn hdrok(&self) -> HdrokR {
        HdrokR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 9 - Offline"]
    #[inline(always)]
    pub fn offline(&self) -> OfflineR {
        OfflineR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bits 16:21 - Bus Available Match"]
    #[inline(always)]
    pub fn bamatch(&self) -> BamatchR {
        BamatchR::new(((self.bits >> 16) & 0x3f) as u8)
    }
    #[doc = "Bits 25:31 - Static Address"]
    #[inline(always)]
    pub fn saddr(&self) -> SaddrR {
        SaddrR::new(((self.bits >> 25) & 0x7f) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Target Enable"]
    #[inline(always)]
    pub fn slvena(&mut self) -> SlvenaW<SconfigSpec> {
        SlvenaW::new(self, 0)
    }
    #[doc = "Bit 1 - Not Acknowledge"]
    #[inline(always)]
    pub fn nack(&mut self) -> NackW<SconfigSpec> {
        NackW::new(self, 1)
    }
    #[doc = "Bit 2 - Match Start or Stop"]
    #[inline(always)]
    pub fn matchss(&mut self) -> MatchssW<SconfigSpec> {
        MatchssW::new(self, 2)
    }
    #[doc = "Bit 3 - Ignore TE0 or TE1 Errors"]
    #[inline(always)]
    pub fn s0ignore(&mut self) -> S0ignoreW<SconfigSpec> {
        S0ignoreW::new(self, 3)
    }
    #[doc = "Bit 4 - HDR OK"]
    #[inline(always)]
    pub fn hdrok(&mut self) -> HdrokW<SconfigSpec> {
        HdrokW::new(self, 4)
    }
    #[doc = "Bit 9 - Offline"]
    #[inline(always)]
    pub fn offline(&mut self) -> OfflineW<SconfigSpec> {
        OfflineW::new(self, 9)
    }
    #[doc = "Bits 16:21 - Bus Available Match"]
    #[inline(always)]
    pub fn bamatch(&mut self) -> BamatchW<SconfigSpec> {
        BamatchW::new(self, 16)
    }
    #[doc = "Bits 25:31 - Static Address"]
    #[inline(always)]
    pub fn saddr(&mut self) -> SaddrW<SconfigSpec> {
        SaddrW::new(self, 25)
    }
}
#[doc = "Target Configuration\n\nYou can [`read`](crate::Reg::read) this register and get [`sconfig::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sconfig::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SconfigSpec;
impl crate::RegisterSpec for SconfigSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sconfig::R`](R) reader structure"]
impl crate::Readable for SconfigSpec {}
#[doc = "`write(|w| ..)` method takes [`sconfig::W`](W) writer structure"]
impl crate::Writable for SconfigSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SCONFIG to value 0x0017_0000"]
impl crate::Resettable for SconfigSpec {
    const RESET_VALUE: u32 = 0x0017_0000;
}

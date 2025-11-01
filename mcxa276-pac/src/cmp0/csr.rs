#[doc = "Register `CSR` reader"]
pub type R = crate::R<CsrSpec>;
#[doc = "Register `CSR` writer"]
pub type W = crate::W<CsrSpec>;
#[doc = "Analog Comparator Flag Rising\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cfr {
    #[doc = "0: Not detected"]
    NotDetected = 0,
    #[doc = "1: Detected"]
    Detected = 1,
}
impl From<Cfr> for bool {
    #[inline(always)]
    fn from(variant: Cfr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CFR` reader - Analog Comparator Flag Rising"]
pub type CfrR = crate::BitReader<Cfr>;
impl CfrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cfr {
        match self.bits {
            false => Cfr::NotDetected,
            true => Cfr::Detected,
        }
    }
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn is_not_detected(&self) -> bool {
        *self == Cfr::NotDetected
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn is_detected(&self) -> bool {
        *self == Cfr::Detected
    }
}
#[doc = "Field `CFR` writer - Analog Comparator Flag Rising"]
pub type CfrW<'a, REG> = crate::BitWriter1C<'a, REG, Cfr>;
impl<'a, REG> CfrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn not_detected(self) -> &'a mut crate::W<REG> {
        self.variant(Cfr::NotDetected)
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn detected(self) -> &'a mut crate::W<REG> {
        self.variant(Cfr::Detected)
    }
}
#[doc = "Analog Comparator Flag Falling\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cff {
    #[doc = "0: Not detected"]
    NotDetected = 0,
    #[doc = "1: Detected"]
    Detected = 1,
}
impl From<Cff> for bool {
    #[inline(always)]
    fn from(variant: Cff) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CFF` reader - Analog Comparator Flag Falling"]
pub type CffR = crate::BitReader<Cff>;
impl CffR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Cff {
        match self.bits {
            false => Cff::NotDetected,
            true => Cff::Detected,
        }
    }
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn is_not_detected(&self) -> bool {
        *self == Cff::NotDetected
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn is_detected(&self) -> bool {
        *self == Cff::Detected
    }
}
#[doc = "Field `CFF` writer - Analog Comparator Flag Falling"]
pub type CffW<'a, REG> = crate::BitWriter1C<'a, REG, Cff>;
impl<'a, REG> CffW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn not_detected(self) -> &'a mut crate::W<REG> {
        self.variant(Cff::NotDetected)
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn detected(self) -> &'a mut crate::W<REG> {
        self.variant(Cff::Detected)
    }
}
#[doc = "Round-Robin Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rrf {
    #[doc = "0: Not detected"]
    NotDetected = 0,
    #[doc = "1: Detected"]
    Detected = 1,
}
impl From<Rrf> for bool {
    #[inline(always)]
    fn from(variant: Rrf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `RRF` reader - Round-Robin Flag"]
pub type RrfR = crate::BitReader<Rrf>;
impl RrfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Rrf {
        match self.bits {
            false => Rrf::NotDetected,
            true => Rrf::Detected,
        }
    }
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn is_not_detected(&self) -> bool {
        *self == Rrf::NotDetected
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn is_detected(&self) -> bool {
        *self == Rrf::Detected
    }
}
#[doc = "Field `RRF` writer - Round-Robin Flag"]
pub type RrfW<'a, REG> = crate::BitWriter1C<'a, REG, Rrf>;
impl<'a, REG> RrfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn not_detected(self) -> &'a mut crate::W<REG> {
        self.variant(Rrf::NotDetected)
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn detected(self) -> &'a mut crate::W<REG> {
        self.variant(Rrf::Detected)
    }
}
#[doc = "Field `COUT` reader - Analog Comparator Output"]
pub type CoutR = crate::BitReader;
impl R {
    #[doc = "Bit 0 - Analog Comparator Flag Rising"]
    #[inline(always)]
    pub fn cfr(&self) -> CfrR {
        CfrR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Analog Comparator Flag Falling"]
    #[inline(always)]
    pub fn cff(&self) -> CffR {
        CffR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Round-Robin Flag"]
    #[inline(always)]
    pub fn rrf(&self) -> RrfR {
        RrfR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 8 - Analog Comparator Output"]
    #[inline(always)]
    pub fn cout(&self) -> CoutR {
        CoutR::new(((self.bits >> 8) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Analog Comparator Flag Rising"]
    #[inline(always)]
    pub fn cfr(&mut self) -> CfrW<CsrSpec> {
        CfrW::new(self, 0)
    }
    #[doc = "Bit 1 - Analog Comparator Flag Falling"]
    #[inline(always)]
    pub fn cff(&mut self) -> CffW<CsrSpec> {
        CffW::new(self, 1)
    }
    #[doc = "Bit 2 - Round-Robin Flag"]
    #[inline(always)]
    pub fn rrf(&mut self) -> RrfW<CsrSpec> {
        RrfW::new(self, 2)
    }
}
#[doc = "Comparator Status\n\nYou can [`read`](crate::Reg::read) this register and get [`csr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`csr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct CsrSpec;
impl crate::RegisterSpec for CsrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`csr::R`](R) reader structure"]
impl crate::Readable for CsrSpec {}
#[doc = "`write(|w| ..)` method takes [`csr::W`](W) writer structure"]
impl crate::Writable for CsrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x07;
}
#[doc = "`reset()` method sets CSR to value 0"]
impl crate::Resettable for CsrSpec {}

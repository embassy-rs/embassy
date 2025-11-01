#[doc = "Register `ETDC` reader"]
pub type R = crate::R<EtdcSpec>;
#[doc = "Register `ETDC` writer"]
pub type W = crate::W<EtdcSpec>;
#[doc = "Field `ETDCVAL` reader - Enhanced Transceiver Delay Compensation Value"]
pub type EtdcvalR = crate::FieldReader;
#[doc = "Transceiver Delay Compensation Fail\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Etdcfail {
    #[doc = "0: In range"]
    InRange = 0,
    #[doc = "1: Out of range"]
    OutOfRange = 1,
}
impl From<Etdcfail> for bool {
    #[inline(always)]
    fn from(variant: Etdcfail) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ETDCFAIL` reader - Transceiver Delay Compensation Fail"]
pub type EtdcfailR = crate::BitReader<Etdcfail>;
impl EtdcfailR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Etdcfail {
        match self.bits {
            false => Etdcfail::InRange,
            true => Etdcfail::OutOfRange,
        }
    }
    #[doc = "In range"]
    #[inline(always)]
    pub fn is_in_range(&self) -> bool {
        *self == Etdcfail::InRange
    }
    #[doc = "Out of range"]
    #[inline(always)]
    pub fn is_out_of_range(&self) -> bool {
        *self == Etdcfail::OutOfRange
    }
}
#[doc = "Field `ETDCFAIL` writer - Transceiver Delay Compensation Fail"]
pub type EtdcfailW<'a, REG> = crate::BitWriter1C<'a, REG, Etdcfail>;
impl<'a, REG> EtdcfailW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "In range"]
    #[inline(always)]
    pub fn in_range(self) -> &'a mut crate::W<REG> {
        self.variant(Etdcfail::InRange)
    }
    #[doc = "Out of range"]
    #[inline(always)]
    pub fn out_of_range(self) -> &'a mut crate::W<REG> {
        self.variant(Etdcfail::OutOfRange)
    }
}
#[doc = "Field `ETDCOFF` reader - Enhanced Transceiver Delay Compensation Offset"]
pub type EtdcoffR = crate::FieldReader;
#[doc = "Field `ETDCOFF` writer - Enhanced Transceiver Delay Compensation Offset"]
pub type EtdcoffW<'a, REG> = crate::FieldWriter<'a, REG, 7>;
#[doc = "Transceiver Delay Measurement Disable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdmdis {
    #[doc = "0: Enable"]
    Enable = 0,
    #[doc = "1: Disable"]
    Disable = 1,
}
impl From<Tdmdis> for bool {
    #[inline(always)]
    fn from(variant: Tdmdis) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TDMDIS` reader - Transceiver Delay Measurement Disable"]
pub type TdmdisR = crate::BitReader<Tdmdis>;
impl TdmdisR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tdmdis {
        match self.bits {
            false => Tdmdis::Enable,
            true => Tdmdis::Disable,
        }
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tdmdis::Enable
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tdmdis::Disable
    }
}
#[doc = "Field `TDMDIS` writer - Transceiver Delay Measurement Disable"]
pub type TdmdisW<'a, REG> = crate::BitWriter<'a, REG, Tdmdis>;
impl<'a, REG> TdmdisW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tdmdis::Enable)
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tdmdis::Disable)
    }
}
#[doc = "Transceiver Delay Compensation Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Etdcen {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Etdcen> for bool {
    #[inline(always)]
    fn from(variant: Etdcen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ETDCEN` reader - Transceiver Delay Compensation Enable"]
pub type EtdcenR = crate::BitReader<Etdcen>;
impl EtdcenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Etdcen {
        match self.bits {
            false => Etdcen::Disable,
            true => Etdcen::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Etdcen::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Etdcen::Enable
    }
}
#[doc = "Field `ETDCEN` writer - Transceiver Delay Compensation Enable"]
pub type EtdcenW<'a, REG> = crate::BitWriter<'a, REG, Etdcen>;
impl<'a, REG> EtdcenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Etdcen::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Etdcen::Enable)
    }
}
impl R {
    #[doc = "Bits 0:7 - Enhanced Transceiver Delay Compensation Value"]
    #[inline(always)]
    pub fn etdcval(&self) -> EtdcvalR {
        EtdcvalR::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bit 15 - Transceiver Delay Compensation Fail"]
    #[inline(always)]
    pub fn etdcfail(&self) -> EtdcfailR {
        EtdcfailR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bits 16:22 - Enhanced Transceiver Delay Compensation Offset"]
    #[inline(always)]
    pub fn etdcoff(&self) -> EtdcoffR {
        EtdcoffR::new(((self.bits >> 16) & 0x7f) as u8)
    }
    #[doc = "Bit 30 - Transceiver Delay Measurement Disable"]
    #[inline(always)]
    pub fn tdmdis(&self) -> TdmdisR {
        TdmdisR::new(((self.bits >> 30) & 1) != 0)
    }
    #[doc = "Bit 31 - Transceiver Delay Compensation Enable"]
    #[inline(always)]
    pub fn etdcen(&self) -> EtdcenR {
        EtdcenR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 15 - Transceiver Delay Compensation Fail"]
    #[inline(always)]
    pub fn etdcfail(&mut self) -> EtdcfailW<EtdcSpec> {
        EtdcfailW::new(self, 15)
    }
    #[doc = "Bits 16:22 - Enhanced Transceiver Delay Compensation Offset"]
    #[inline(always)]
    pub fn etdcoff(&mut self) -> EtdcoffW<EtdcSpec> {
        EtdcoffW::new(self, 16)
    }
    #[doc = "Bit 30 - Transceiver Delay Measurement Disable"]
    #[inline(always)]
    pub fn tdmdis(&mut self) -> TdmdisW<EtdcSpec> {
        TdmdisW::new(self, 30)
    }
    #[doc = "Bit 31 - Transceiver Delay Compensation Enable"]
    #[inline(always)]
    pub fn etdcen(&mut self) -> EtdcenW<EtdcSpec> {
        EtdcenW::new(self, 31)
    }
}
#[doc = "Enhanced Transceiver Delay Compensation\n\nYou can [`read`](crate::Reg::read) this register and get [`etdc::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`etdc::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct EtdcSpec;
impl crate::RegisterSpec for EtdcSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`etdc::R`](R) reader structure"]
impl crate::Readable for EtdcSpec {}
#[doc = "`write(|w| ..)` method takes [`etdc::W`](W) writer structure"]
impl crate::Writable for EtdcSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x8000;
}
#[doc = "`reset()` method sets ETDC to value 0"]
impl crate::Resettable for EtdcSpec {}

#[doc = "Register `FDCTRL` reader"]
pub type R = crate::R<FdctrlSpec>;
#[doc = "Register `FDCTRL` writer"]
pub type W = crate::W<FdctrlSpec>;
#[doc = "Field `TDCVAL` reader - Transceiver Delay Compensation Value"]
pub type TdcvalR = crate::FieldReader;
#[doc = "Field `TDCOFF` reader - Transceiver Delay Compensation Offset"]
pub type TdcoffR = crate::FieldReader;
#[doc = "Field `TDCOFF` writer - Transceiver Delay Compensation Offset"]
pub type TdcoffW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Transceiver Delay Compensation Fail\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdcfail {
    #[doc = "0: In range"]
    InRange = 0,
    #[doc = "1: Out of range"]
    OutOfRange = 1,
}
impl From<Tdcfail> for bool {
    #[inline(always)]
    fn from(variant: Tdcfail) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TDCFAIL` reader - Transceiver Delay Compensation Fail"]
pub type TdcfailR = crate::BitReader<Tdcfail>;
impl TdcfailR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tdcfail {
        match self.bits {
            false => Tdcfail::InRange,
            true => Tdcfail::OutOfRange,
        }
    }
    #[doc = "In range"]
    #[inline(always)]
    pub fn is_in_range(&self) -> bool {
        *self == Tdcfail::InRange
    }
    #[doc = "Out of range"]
    #[inline(always)]
    pub fn is_out_of_range(&self) -> bool {
        *self == Tdcfail::OutOfRange
    }
}
#[doc = "Field `TDCFAIL` writer - Transceiver Delay Compensation Fail"]
pub type TdcfailW<'a, REG> = crate::BitWriter1C<'a, REG, Tdcfail>;
impl<'a, REG> TdcfailW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "In range"]
    #[inline(always)]
    pub fn in_range(self) -> &'a mut crate::W<REG> {
        self.variant(Tdcfail::InRange)
    }
    #[doc = "Out of range"]
    #[inline(always)]
    pub fn out_of_range(self) -> &'a mut crate::W<REG> {
        self.variant(Tdcfail::OutOfRange)
    }
}
#[doc = "Transceiver Delay Compensation Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdcen {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<Tdcen> for bool {
    #[inline(always)]
    fn from(variant: Tdcen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TDCEN` reader - Transceiver Delay Compensation Enable"]
pub type TdcenR = crate::BitReader<Tdcen>;
impl TdcenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tdcen {
        match self.bits {
            false => Tdcen::Disable,
            true => Tdcen::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Tdcen::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Tdcen::Enable
    }
}
#[doc = "Field `TDCEN` writer - Transceiver Delay Compensation Enable"]
pub type TdcenW<'a, REG> = crate::BitWriter<'a, REG, Tdcen>;
impl<'a, REG> TdcenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Tdcen::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Tdcen::Enable)
    }
}
#[doc = "Message Buffer Data Size for Region 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Mbdsr0 {
    #[doc = "0: 8 bytes"]
    R0_8Bytes = 0,
    #[doc = "1: 16 bytes"]
    R0_16Bytes = 1,
    #[doc = "2: 32 bytes"]
    R0_32Bytes = 2,
    #[doc = "3: 64 bytes"]
    R0_64Bytes = 3,
}
impl From<Mbdsr0> for u8 {
    #[inline(always)]
    fn from(variant: Mbdsr0) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Mbdsr0 {
    type Ux = u8;
}
impl crate::IsEnum for Mbdsr0 {}
#[doc = "Field `MBDSR0` reader - Message Buffer Data Size for Region 0"]
pub type Mbdsr0R = crate::FieldReader<Mbdsr0>;
impl Mbdsr0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Mbdsr0 {
        match self.bits {
            0 => Mbdsr0::R0_8Bytes,
            1 => Mbdsr0::R0_16Bytes,
            2 => Mbdsr0::R0_32Bytes,
            3 => Mbdsr0::R0_64Bytes,
            _ => unreachable!(),
        }
    }
    #[doc = "8 bytes"]
    #[inline(always)]
    pub fn is_r0_8_bytes(&self) -> bool {
        *self == Mbdsr0::R0_8Bytes
    }
    #[doc = "16 bytes"]
    #[inline(always)]
    pub fn is_r0_16_bytes(&self) -> bool {
        *self == Mbdsr0::R0_16Bytes
    }
    #[doc = "32 bytes"]
    #[inline(always)]
    pub fn is_r0_32_bytes(&self) -> bool {
        *self == Mbdsr0::R0_32Bytes
    }
    #[doc = "64 bytes"]
    #[inline(always)]
    pub fn is_r0_64_bytes(&self) -> bool {
        *self == Mbdsr0::R0_64Bytes
    }
}
#[doc = "Field `MBDSR0` writer - Message Buffer Data Size for Region 0"]
pub type Mbdsr0W<'a, REG> = crate::FieldWriter<'a, REG, 2, Mbdsr0, crate::Safe>;
impl<'a, REG> Mbdsr0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "8 bytes"]
    #[inline(always)]
    pub fn r0_8_bytes(self) -> &'a mut crate::W<REG> {
        self.variant(Mbdsr0::R0_8Bytes)
    }
    #[doc = "16 bytes"]
    #[inline(always)]
    pub fn r0_16_bytes(self) -> &'a mut crate::W<REG> {
        self.variant(Mbdsr0::R0_16Bytes)
    }
    #[doc = "32 bytes"]
    #[inline(always)]
    pub fn r0_32_bytes(self) -> &'a mut crate::W<REG> {
        self.variant(Mbdsr0::R0_32Bytes)
    }
    #[doc = "64 bytes"]
    #[inline(always)]
    pub fn r0_64_bytes(self) -> &'a mut crate::W<REG> {
        self.variant(Mbdsr0::R0_64Bytes)
    }
}
#[doc = "Bit Rate Switch Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fdrate {
    #[doc = "0: Disable"]
    Nominal = 0,
    #[doc = "1: Enable"]
    BitRateSwitching = 1,
}
impl From<Fdrate> for bool {
    #[inline(always)]
    fn from(variant: Fdrate) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FDRATE` reader - Bit Rate Switch Enable"]
pub type FdrateR = crate::BitReader<Fdrate>;
impl FdrateR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fdrate {
        match self.bits {
            false => Fdrate::Nominal,
            true => Fdrate::BitRateSwitching,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_nominal(&self) -> bool {
        *self == Fdrate::Nominal
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_bit_rate_switching(&self) -> bool {
        *self == Fdrate::BitRateSwitching
    }
}
#[doc = "Field `FDRATE` writer - Bit Rate Switch Enable"]
pub type FdrateW<'a, REG> = crate::BitWriter<'a, REG, Fdrate>;
impl<'a, REG> FdrateW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn nominal(self) -> &'a mut crate::W<REG> {
        self.variant(Fdrate::Nominal)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn bit_rate_switching(self) -> &'a mut crate::W<REG> {
        self.variant(Fdrate::BitRateSwitching)
    }
}
impl R {
    #[doc = "Bits 0:5 - Transceiver Delay Compensation Value"]
    #[inline(always)]
    pub fn tdcval(&self) -> TdcvalR {
        TdcvalR::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bits 8:12 - Transceiver Delay Compensation Offset"]
    #[inline(always)]
    pub fn tdcoff(&self) -> TdcoffR {
        TdcoffR::new(((self.bits >> 8) & 0x1f) as u8)
    }
    #[doc = "Bit 14 - Transceiver Delay Compensation Fail"]
    #[inline(always)]
    pub fn tdcfail(&self) -> TdcfailR {
        TdcfailR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - Transceiver Delay Compensation Enable"]
    #[inline(always)]
    pub fn tdcen(&self) -> TdcenR {
        TdcenR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bits 16:17 - Message Buffer Data Size for Region 0"]
    #[inline(always)]
    pub fn mbdsr0(&self) -> Mbdsr0R {
        Mbdsr0R::new(((self.bits >> 16) & 3) as u8)
    }
    #[doc = "Bit 31 - Bit Rate Switch Enable"]
    #[inline(always)]
    pub fn fdrate(&self) -> FdrateR {
        FdrateR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 8:12 - Transceiver Delay Compensation Offset"]
    #[inline(always)]
    pub fn tdcoff(&mut self) -> TdcoffW<FdctrlSpec> {
        TdcoffW::new(self, 8)
    }
    #[doc = "Bit 14 - Transceiver Delay Compensation Fail"]
    #[inline(always)]
    pub fn tdcfail(&mut self) -> TdcfailW<FdctrlSpec> {
        TdcfailW::new(self, 14)
    }
    #[doc = "Bit 15 - Transceiver Delay Compensation Enable"]
    #[inline(always)]
    pub fn tdcen(&mut self) -> TdcenW<FdctrlSpec> {
        TdcenW::new(self, 15)
    }
    #[doc = "Bits 16:17 - Message Buffer Data Size for Region 0"]
    #[inline(always)]
    pub fn mbdsr0(&mut self) -> Mbdsr0W<FdctrlSpec> {
        Mbdsr0W::new(self, 16)
    }
    #[doc = "Bit 31 - Bit Rate Switch Enable"]
    #[inline(always)]
    pub fn fdrate(&mut self) -> FdrateW<FdctrlSpec> {
        FdrateW::new(self, 31)
    }
}
#[doc = "CAN FD Control\n\nYou can [`read`](crate::Reg::read) this register and get [`fdctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`fdctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FdctrlSpec;
impl crate::RegisterSpec for FdctrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`fdctrl::R`](R) reader structure"]
impl crate::Readable for FdctrlSpec {}
#[doc = "`write(|w| ..)` method takes [`fdctrl::W`](W) writer structure"]
impl crate::Writable for FdctrlSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x4000;
}
#[doc = "`reset()` method sets FDCTRL to value 0x8000_0100"]
impl crate::Resettable for FdctrlSpec {
    const RESET_VALUE: u32 = 0x8000_0100;
}

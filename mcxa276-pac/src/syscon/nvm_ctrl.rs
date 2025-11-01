#[doc = "Register `NVM_CTRL` reader"]
pub type R = crate::R<NvmCtrlSpec>;
#[doc = "Register `NVM_CTRL` writer"]
pub type W = crate::W<NvmCtrlSpec>;
#[doc = "Flash speculation control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisFlashSpec {
    #[doc = "0: Enables flash speculation"]
    Enable = 0,
    #[doc = "1: Disables flash speculation"]
    Disable = 1,
}
impl From<DisFlashSpec> for bool {
    #[inline(always)]
    fn from(variant: DisFlashSpec) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DIS_FLASH_SPEC` reader - Flash speculation control"]
pub type DisFlashSpecR = crate::BitReader<DisFlashSpec>;
impl DisFlashSpecR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> DisFlashSpec {
        match self.bits {
            false => DisFlashSpec::Enable,
            true => DisFlashSpec::Disable,
        }
    }
    #[doc = "Enables flash speculation"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == DisFlashSpec::Enable
    }
    #[doc = "Disables flash speculation"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == DisFlashSpec::Disable
    }
}
#[doc = "Field `DIS_FLASH_SPEC` writer - Flash speculation control"]
pub type DisFlashSpecW<'a, REG> = crate::BitWriter<'a, REG, DisFlashSpec>;
impl<'a, REG> DisFlashSpecW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enables flash speculation"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(DisFlashSpec::Enable)
    }
    #[doc = "Disables flash speculation"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(DisFlashSpec::Disable)
    }
}
#[doc = "Flash data speculation control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisDataSpec {
    #[doc = "0: Enables data speculation"]
    Enable = 0,
    #[doc = "1: Disables data speculation"]
    Disable = 1,
}
impl From<DisDataSpec> for bool {
    #[inline(always)]
    fn from(variant: DisDataSpec) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DIS_DATA_SPEC` reader - Flash data speculation control"]
pub type DisDataSpecR = crate::BitReader<DisDataSpec>;
impl DisDataSpecR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> DisDataSpec {
        match self.bits {
            false => DisDataSpec::Enable,
            true => DisDataSpec::Disable,
        }
    }
    #[doc = "Enables data speculation"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == DisDataSpec::Enable
    }
    #[doc = "Disables data speculation"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == DisDataSpec::Disable
    }
}
#[doc = "Field `DIS_DATA_SPEC` writer - Flash data speculation control"]
pub type DisDataSpecW<'a, REG> = crate::BitWriter<'a, REG, DisDataSpec>;
impl<'a, REG> DisDataSpecW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enables data speculation"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(DisDataSpec::Enable)
    }
    #[doc = "Disables data speculation"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(DisDataSpec::Disable)
    }
}
#[doc = "FLASH stall on busy control\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlashStallEn {
    #[doc = "0: No stall on FLASH busy"]
    Enable = 0,
    #[doc = "1: Stall on FLASH busy"]
    Disable = 1,
}
impl From<FlashStallEn> for bool {
    #[inline(always)]
    fn from(variant: FlashStallEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FLASH_STALL_EN` reader - FLASH stall on busy control"]
pub type FlashStallEnR = crate::BitReader<FlashStallEn>;
impl FlashStallEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> FlashStallEn {
        match self.bits {
            false => FlashStallEn::Enable,
            true => FlashStallEn::Disable,
        }
    }
    #[doc = "No stall on FLASH busy"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == FlashStallEn::Enable
    }
    #[doc = "Stall on FLASH busy"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == FlashStallEn::Disable
    }
}
#[doc = "Field `FLASH_STALL_EN` writer - FLASH stall on busy control"]
pub type FlashStallEnW<'a, REG> = crate::BitWriter<'a, REG, FlashStallEn>;
impl<'a, REG> FlashStallEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No stall on FLASH busy"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(FlashStallEn::Enable)
    }
    #[doc = "Stall on FLASH busy"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(FlashStallEn::Disable)
    }
}
#[doc = "Bus error on data multi-bit ECC error control Set this field to 0 if you want to enable flash speculative\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisMbeccErrInst {
    #[doc = "0: Enables bus error on multi-bit ECC error for instruction"]
    Enable = 0,
    #[doc = "1: Disables bus error on multi-bit ECC error for instruction"]
    Disable = 1,
}
impl From<DisMbeccErrInst> for bool {
    #[inline(always)]
    fn from(variant: DisMbeccErrInst) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DIS_MBECC_ERR_INST` reader - Bus error on data multi-bit ECC error control Set this field to 0 if you want to enable flash speculative"]
pub type DisMbeccErrInstR = crate::BitReader<DisMbeccErrInst>;
impl DisMbeccErrInstR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> DisMbeccErrInst {
        match self.bits {
            false => DisMbeccErrInst::Enable,
            true => DisMbeccErrInst::Disable,
        }
    }
    #[doc = "Enables bus error on multi-bit ECC error for instruction"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == DisMbeccErrInst::Enable
    }
    #[doc = "Disables bus error on multi-bit ECC error for instruction"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == DisMbeccErrInst::Disable
    }
}
#[doc = "Field `DIS_MBECC_ERR_INST` writer - Bus error on data multi-bit ECC error control Set this field to 0 if you want to enable flash speculative"]
pub type DisMbeccErrInstW<'a, REG> = crate::BitWriter<'a, REG, DisMbeccErrInst>;
impl<'a, REG> DisMbeccErrInstW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enables bus error on multi-bit ECC error for instruction"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(DisMbeccErrInst::Enable)
    }
    #[doc = "Disables bus error on multi-bit ECC error for instruction"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(DisMbeccErrInst::Disable)
    }
}
#[doc = "Bus error on data multi-bit ECC error control Set this field to 0 if you want to enable flash speculative\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisMbeccErrData {
    #[doc = "0: Enables bus error on multi-bit ECC error for data"]
    Enable = 0,
    #[doc = "1: Disables bus error on multi-bit ECC error for data"]
    Disable = 1,
}
impl From<DisMbeccErrData> for bool {
    #[inline(always)]
    fn from(variant: DisMbeccErrData) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DIS_MBECC_ERR_DATA` reader - Bus error on data multi-bit ECC error control Set this field to 0 if you want to enable flash speculative"]
pub type DisMbeccErrDataR = crate::BitReader<DisMbeccErrData>;
impl DisMbeccErrDataR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> DisMbeccErrData {
        match self.bits {
            false => DisMbeccErrData::Enable,
            true => DisMbeccErrData::Disable,
        }
    }
    #[doc = "Enables bus error on multi-bit ECC error for data"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == DisMbeccErrData::Enable
    }
    #[doc = "Disables bus error on multi-bit ECC error for data"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == DisMbeccErrData::Disable
    }
}
#[doc = "Field `DIS_MBECC_ERR_DATA` writer - Bus error on data multi-bit ECC error control Set this field to 0 if you want to enable flash speculative"]
pub type DisMbeccErrDataW<'a, REG> = crate::BitWriter<'a, REG, DisMbeccErrData>;
impl<'a, REG> DisMbeccErrDataW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Enables bus error on multi-bit ECC error for data"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(DisMbeccErrData::Enable)
    }
    #[doc = "Disables bus error on multi-bit ECC error for data"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(DisMbeccErrData::Disable)
    }
}
impl R {
    #[doc = "Bit 0 - Flash speculation control"]
    #[inline(always)]
    pub fn dis_flash_spec(&self) -> DisFlashSpecR {
        DisFlashSpecR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Flash data speculation control"]
    #[inline(always)]
    pub fn dis_data_spec(&self) -> DisDataSpecR {
        DisDataSpecR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 10 - FLASH stall on busy control"]
    #[inline(always)]
    pub fn flash_stall_en(&self) -> FlashStallEnR {
        FlashStallEnR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 16 - Bus error on data multi-bit ECC error control Set this field to 0 if you want to enable flash speculative"]
    #[inline(always)]
    pub fn dis_mbecc_err_inst(&self) -> DisMbeccErrInstR {
        DisMbeccErrInstR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Bus error on data multi-bit ECC error control Set this field to 0 if you want to enable flash speculative"]
    #[inline(always)]
    pub fn dis_mbecc_err_data(&self) -> DisMbeccErrDataR {
        DisMbeccErrDataR::new(((self.bits >> 17) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Flash speculation control"]
    #[inline(always)]
    pub fn dis_flash_spec(&mut self) -> DisFlashSpecW<NvmCtrlSpec> {
        DisFlashSpecW::new(self, 0)
    }
    #[doc = "Bit 1 - Flash data speculation control"]
    #[inline(always)]
    pub fn dis_data_spec(&mut self) -> DisDataSpecW<NvmCtrlSpec> {
        DisDataSpecW::new(self, 1)
    }
    #[doc = "Bit 10 - FLASH stall on busy control"]
    #[inline(always)]
    pub fn flash_stall_en(&mut self) -> FlashStallEnW<NvmCtrlSpec> {
        FlashStallEnW::new(self, 10)
    }
    #[doc = "Bit 16 - Bus error on data multi-bit ECC error control Set this field to 0 if you want to enable flash speculative"]
    #[inline(always)]
    pub fn dis_mbecc_err_inst(&mut self) -> DisMbeccErrInstW<NvmCtrlSpec> {
        DisMbeccErrInstW::new(self, 16)
    }
    #[doc = "Bit 17 - Bus error on data multi-bit ECC error control Set this field to 0 if you want to enable flash speculative"]
    #[inline(always)]
    pub fn dis_mbecc_err_data(&mut self) -> DisMbeccErrDataW<NvmCtrlSpec> {
        DisMbeccErrDataW::new(self, 17)
    }
}
#[doc = "NVM Control\n\nYou can [`read`](crate::Reg::read) this register and get [`nvm_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`nvm_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct NvmCtrlSpec;
impl crate::RegisterSpec for NvmCtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`nvm_ctrl::R`](R) reader structure"]
impl crate::Readable for NvmCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`nvm_ctrl::W`](W) writer structure"]
impl crate::Writable for NvmCtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets NVM_CTRL to value 0x0002_0400"]
impl crate::Resettable for NvmCtrlSpec {
    const RESET_VALUE: u32 = 0x0002_0400;
}

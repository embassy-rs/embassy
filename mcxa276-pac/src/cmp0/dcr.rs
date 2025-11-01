#[doc = "Register `DCR` reader"]
pub type R = crate::R<DcrSpec>;
#[doc = "Register `DCR` writer"]
pub type W = crate::W<DcrSpec>;
#[doc = "DAC Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DacEn {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<DacEn> for bool {
    #[inline(always)]
    fn from(variant: DacEn) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DAC_EN` reader - DAC Enable"]
pub type DacEnR = crate::BitReader<DacEn>;
impl DacEnR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> DacEn {
        match self.bits {
            false => DacEn::Disable,
            true => DacEn::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == DacEn::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == DacEn::Enable
    }
}
#[doc = "Field `DAC_EN` writer - DAC Enable"]
pub type DacEnW<'a, REG> = crate::BitWriter<'a, REG, DacEn>;
impl<'a, REG> DacEnW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(DacEn::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(DacEn::Enable)
    }
}
#[doc = "DAC High Power Mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DacHpmd {
    #[doc = "0: Disable"]
    Disable = 0,
    #[doc = "1: Enable"]
    Enable = 1,
}
impl From<DacHpmd> for bool {
    #[inline(always)]
    fn from(variant: DacHpmd) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `DAC_HPMD` reader - DAC High Power Mode"]
pub type DacHpmdR = crate::BitReader<DacHpmd>;
impl DacHpmdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> DacHpmd {
        match self.bits {
            false => DacHpmd::Disable,
            true => DacHpmd::Enable,
        }
    }
    #[doc = "Disable"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == DacHpmd::Disable
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == DacHpmd::Enable
    }
}
#[doc = "Field `DAC_HPMD` writer - DAC High Power Mode"]
pub type DacHpmdW<'a, REG> = crate::BitWriter<'a, REG, DacHpmd>;
impl<'a, REG> DacHpmdW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(DacHpmd::Disable)
    }
    #[doc = "Enable"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(DacHpmd::Enable)
    }
}
#[doc = "DAC Reference High Voltage Source Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vrsel {
    #[doc = "0: VREFH0"]
    Vref0 = 0,
    #[doc = "1: VREFH1"]
    Vref1 = 1,
}
impl From<Vrsel> for bool {
    #[inline(always)]
    fn from(variant: Vrsel) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `VRSEL` reader - DAC Reference High Voltage Source Select"]
pub type VrselR = crate::BitReader<Vrsel>;
impl VrselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Vrsel {
        match self.bits {
            false => Vrsel::Vref0,
            true => Vrsel::Vref1,
        }
    }
    #[doc = "VREFH0"]
    #[inline(always)]
    pub fn is_vref0(&self) -> bool {
        *self == Vrsel::Vref0
    }
    #[doc = "VREFH1"]
    #[inline(always)]
    pub fn is_vref1(&self) -> bool {
        *self == Vrsel::Vref1
    }
}
#[doc = "Field `VRSEL` writer - DAC Reference High Voltage Source Select"]
pub type VrselW<'a, REG> = crate::BitWriter<'a, REG, Vrsel>;
impl<'a, REG> VrselW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "VREFH0"]
    #[inline(always)]
    pub fn vref0(self) -> &'a mut crate::W<REG> {
        self.variant(Vrsel::Vref0)
    }
    #[doc = "VREFH1"]
    #[inline(always)]
    pub fn vref1(self) -> &'a mut crate::W<REG> {
        self.variant(Vrsel::Vref1)
    }
}
#[doc = "Field `DAC_DATA` reader - DAC Output Voltage Select"]
pub type DacDataR = crate::FieldReader;
#[doc = "Field `DAC_DATA` writer - DAC Output Voltage Select"]
pub type DacDataW<'a, REG> = crate::FieldWriter<'a, REG, 8>;
impl R {
    #[doc = "Bit 0 - DAC Enable"]
    #[inline(always)]
    pub fn dac_en(&self) -> DacEnR {
        DacEnR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - DAC High Power Mode"]
    #[inline(always)]
    pub fn dac_hpmd(&self) -> DacHpmdR {
        DacHpmdR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 8 - DAC Reference High Voltage Source Select"]
    #[inline(always)]
    pub fn vrsel(&self) -> VrselR {
        VrselR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bits 16:23 - DAC Output Voltage Select"]
    #[inline(always)]
    pub fn dac_data(&self) -> DacDataR {
        DacDataR::new(((self.bits >> 16) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - DAC Enable"]
    #[inline(always)]
    pub fn dac_en(&mut self) -> DacEnW<DcrSpec> {
        DacEnW::new(self, 0)
    }
    #[doc = "Bit 1 - DAC High Power Mode"]
    #[inline(always)]
    pub fn dac_hpmd(&mut self) -> DacHpmdW<DcrSpec> {
        DacHpmdW::new(self, 1)
    }
    #[doc = "Bit 8 - DAC Reference High Voltage Source Select"]
    #[inline(always)]
    pub fn vrsel(&mut self) -> VrselW<DcrSpec> {
        VrselW::new(self, 8)
    }
    #[doc = "Bits 16:23 - DAC Output Voltage Select"]
    #[inline(always)]
    pub fn dac_data(&mut self) -> DacDataW<DcrSpec> {
        DacDataW::new(self, 16)
    }
}
#[doc = "DAC Control\n\nYou can [`read`](crate::Reg::read) this register and get [`dcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`dcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct DcrSpec;
impl crate::RegisterSpec for DcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`dcr::R`](R) reader structure"]
impl crate::Readable for DcrSpec {}
#[doc = "`write(|w| ..)` method takes [`dcr::W`](W) writer structure"]
impl crate::Writable for DcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets DCR to value 0"]
impl crate::Resettable for DcrSpec {}

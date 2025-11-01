#[doc = "Register `CCR2` reader"]
pub type R = crate::R<Ccr2Spec>;
#[doc = "Register `CCR2` writer"]
pub type W = crate::W<Ccr2Spec>;
#[doc = "CMP High Power Mode Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmpHpmd {
    #[doc = "0: Low power (speed) comparison mode"]
    Low = 0,
    #[doc = "1: High power (speed) comparison mode"]
    High = 1,
}
impl From<CmpHpmd> for bool {
    #[inline(always)]
    fn from(variant: CmpHpmd) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP_HPMD` reader - CMP High Power Mode Select"]
pub type CmpHpmdR = crate::BitReader<CmpHpmd>;
impl CmpHpmdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CmpHpmd {
        match self.bits {
            false => CmpHpmd::Low,
            true => CmpHpmd::High,
        }
    }
    #[doc = "Low power (speed) comparison mode"]
    #[inline(always)]
    pub fn is_low(&self) -> bool {
        *self == CmpHpmd::Low
    }
    #[doc = "High power (speed) comparison mode"]
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        *self == CmpHpmd::High
    }
}
#[doc = "Field `CMP_HPMD` writer - CMP High Power Mode Select"]
pub type CmpHpmdW<'a, REG> = crate::BitWriter<'a, REG, CmpHpmd>;
impl<'a, REG> CmpHpmdW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Low power (speed) comparison mode"]
    #[inline(always)]
    pub fn low(self) -> &'a mut crate::W<REG> {
        self.variant(CmpHpmd::Low)
    }
    #[doc = "High power (speed) comparison mode"]
    #[inline(always)]
    pub fn high(self) -> &'a mut crate::W<REG> {
        self.variant(CmpHpmd::High)
    }
}
#[doc = "CMP Nano Power Mode Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmpNpmd {
    #[doc = "0: Disables CMP Nano power mode. CCR2\\[CMP_HPMD\\] determines the mode for the comparator."]
    NoNano = 0,
    #[doc = "1: Enables CMP Nano power mode."]
    Nano = 1,
}
impl From<CmpNpmd> for bool {
    #[inline(always)]
    fn from(variant: CmpNpmd) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `CMP_NPMD` reader - CMP Nano Power Mode Select"]
pub type CmpNpmdR = crate::BitReader<CmpNpmd>;
impl CmpNpmdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> CmpNpmd {
        match self.bits {
            false => CmpNpmd::NoNano,
            true => CmpNpmd::Nano,
        }
    }
    #[doc = "Disables CMP Nano power mode. CCR2\\[CMP_HPMD\\] determines the mode for the comparator."]
    #[inline(always)]
    pub fn is_no_nano(&self) -> bool {
        *self == CmpNpmd::NoNano
    }
    #[doc = "Enables CMP Nano power mode."]
    #[inline(always)]
    pub fn is_nano(&self) -> bool {
        *self == CmpNpmd::Nano
    }
}
#[doc = "Field `CMP_NPMD` writer - CMP Nano Power Mode Select"]
pub type CmpNpmdW<'a, REG> = crate::BitWriter<'a, REG, CmpNpmd>;
impl<'a, REG> CmpNpmdW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables CMP Nano power mode. CCR2\\[CMP_HPMD\\] determines the mode for the comparator."]
    #[inline(always)]
    pub fn no_nano(self) -> &'a mut crate::W<REG> {
        self.variant(CmpNpmd::NoNano)
    }
    #[doc = "Enables CMP Nano power mode."]
    #[inline(always)]
    pub fn nano(self) -> &'a mut crate::W<REG> {
        self.variant(CmpNpmd::Nano)
    }
}
#[doc = "Comparator Hysteresis Control\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Hystctr {
    #[doc = "0: Level 0: Analog comparator hysteresis 0 mV."]
    Level0 = 0,
    #[doc = "1: Level 1: Analog comparator hysteresis 10 mV."]
    Level1 = 1,
    #[doc = "2: Level 2: Analog comparator hysteresis 20 mV."]
    Level2 = 2,
    #[doc = "3: Level 3: Analog comparator hysteresis 30 mV."]
    Level3 = 3,
}
impl From<Hystctr> for u8 {
    #[inline(always)]
    fn from(variant: Hystctr) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Hystctr {
    type Ux = u8;
}
impl crate::IsEnum for Hystctr {}
#[doc = "Field `HYSTCTR` reader - Comparator Hysteresis Control"]
pub type HystctrR = crate::FieldReader<Hystctr>;
impl HystctrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Hystctr {
        match self.bits {
            0 => Hystctr::Level0,
            1 => Hystctr::Level1,
            2 => Hystctr::Level2,
            3 => Hystctr::Level3,
            _ => unreachable!(),
        }
    }
    #[doc = "Level 0: Analog comparator hysteresis 0 mV."]
    #[inline(always)]
    pub fn is_level_0(&self) -> bool {
        *self == Hystctr::Level0
    }
    #[doc = "Level 1: Analog comparator hysteresis 10 mV."]
    #[inline(always)]
    pub fn is_level_1(&self) -> bool {
        *self == Hystctr::Level1
    }
    #[doc = "Level 2: Analog comparator hysteresis 20 mV."]
    #[inline(always)]
    pub fn is_level_2(&self) -> bool {
        *self == Hystctr::Level2
    }
    #[doc = "Level 3: Analog comparator hysteresis 30 mV."]
    #[inline(always)]
    pub fn is_level_3(&self) -> bool {
        *self == Hystctr::Level3
    }
}
#[doc = "Field `HYSTCTR` writer - Comparator Hysteresis Control"]
pub type HystctrW<'a, REG> = crate::FieldWriter<'a, REG, 2, Hystctr, crate::Safe>;
impl<'a, REG> HystctrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Level 0: Analog comparator hysteresis 0 mV."]
    #[inline(always)]
    pub fn level_0(self) -> &'a mut crate::W<REG> {
        self.variant(Hystctr::Level0)
    }
    #[doc = "Level 1: Analog comparator hysteresis 10 mV."]
    #[inline(always)]
    pub fn level_1(self) -> &'a mut crate::W<REG> {
        self.variant(Hystctr::Level1)
    }
    #[doc = "Level 2: Analog comparator hysteresis 20 mV."]
    #[inline(always)]
    pub fn level_2(self) -> &'a mut crate::W<REG> {
        self.variant(Hystctr::Level2)
    }
    #[doc = "Level 3: Analog comparator hysteresis 30 mV."]
    #[inline(always)]
    pub fn level_3(self) -> &'a mut crate::W<REG> {
        self.variant(Hystctr::Level3)
    }
}
#[doc = "Plus Input MUX Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Psel {
    #[doc = "0: Input 0p"]
    Input0 = 0,
    #[doc = "1: Input 1p"]
    Input1 = 1,
    #[doc = "2: Input 2p"]
    Input2 = 2,
    #[doc = "3: Input 3p"]
    Input3 = 3,
    #[doc = "4: Input 4p"]
    Input4 = 4,
    #[doc = "5: Input 5p"]
    Input5 = 5,
    #[doc = "7: Internal DAC output"]
    Input7 = 7,
}
impl From<Psel> for u8 {
    #[inline(always)]
    fn from(variant: Psel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Psel {
    type Ux = u8;
}
impl crate::IsEnum for Psel {}
#[doc = "Field `PSEL` reader - Plus Input MUX Select"]
pub type PselR = crate::FieldReader<Psel>;
impl PselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Psel> {
        match self.bits {
            0 => Some(Psel::Input0),
            1 => Some(Psel::Input1),
            2 => Some(Psel::Input2),
            3 => Some(Psel::Input3),
            4 => Some(Psel::Input4),
            5 => Some(Psel::Input5),
            7 => Some(Psel::Input7),
            _ => None,
        }
    }
    #[doc = "Input 0p"]
    #[inline(always)]
    pub fn is_input_0(&self) -> bool {
        *self == Psel::Input0
    }
    #[doc = "Input 1p"]
    #[inline(always)]
    pub fn is_input_1(&self) -> bool {
        *self == Psel::Input1
    }
    #[doc = "Input 2p"]
    #[inline(always)]
    pub fn is_input_2(&self) -> bool {
        *self == Psel::Input2
    }
    #[doc = "Input 3p"]
    #[inline(always)]
    pub fn is_input_3(&self) -> bool {
        *self == Psel::Input3
    }
    #[doc = "Input 4p"]
    #[inline(always)]
    pub fn is_input_4(&self) -> bool {
        *self == Psel::Input4
    }
    #[doc = "Input 5p"]
    #[inline(always)]
    pub fn is_input_5(&self) -> bool {
        *self == Psel::Input5
    }
    #[doc = "Internal DAC output"]
    #[inline(always)]
    pub fn is_input_7(&self) -> bool {
        *self == Psel::Input7
    }
}
#[doc = "Field `PSEL` writer - Plus Input MUX Select"]
pub type PselW<'a, REG> = crate::FieldWriter<'a, REG, 3, Psel>;
impl<'a, REG> PselW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Input 0p"]
    #[inline(always)]
    pub fn input_0(self) -> &'a mut crate::W<REG> {
        self.variant(Psel::Input0)
    }
    #[doc = "Input 1p"]
    #[inline(always)]
    pub fn input_1(self) -> &'a mut crate::W<REG> {
        self.variant(Psel::Input1)
    }
    #[doc = "Input 2p"]
    #[inline(always)]
    pub fn input_2(self) -> &'a mut crate::W<REG> {
        self.variant(Psel::Input2)
    }
    #[doc = "Input 3p"]
    #[inline(always)]
    pub fn input_3(self) -> &'a mut crate::W<REG> {
        self.variant(Psel::Input3)
    }
    #[doc = "Input 4p"]
    #[inline(always)]
    pub fn input_4(self) -> &'a mut crate::W<REG> {
        self.variant(Psel::Input4)
    }
    #[doc = "Input 5p"]
    #[inline(always)]
    pub fn input_5(self) -> &'a mut crate::W<REG> {
        self.variant(Psel::Input5)
    }
    #[doc = "Internal DAC output"]
    #[inline(always)]
    pub fn input_7(self) -> &'a mut crate::W<REG> {
        self.variant(Psel::Input7)
    }
}
#[doc = "Minus Input MUX Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Msel {
    #[doc = "0: Input 0m"]
    Input0 = 0,
    #[doc = "1: Input 1m"]
    Input1 = 1,
    #[doc = "2: Input 2m"]
    Input2 = 2,
    #[doc = "3: Input 3m"]
    Input3 = 3,
    #[doc = "4: Input 4m"]
    Input4 = 4,
    #[doc = "5: Input 5m"]
    Input5 = 5,
    #[doc = "7: Internal DAC output"]
    Input7 = 7,
}
impl From<Msel> for u8 {
    #[inline(always)]
    fn from(variant: Msel) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Msel {
    type Ux = u8;
}
impl crate::IsEnum for Msel {}
#[doc = "Field `MSEL` reader - Minus Input MUX Select"]
pub type MselR = crate::FieldReader<Msel>;
impl MselR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Msel> {
        match self.bits {
            0 => Some(Msel::Input0),
            1 => Some(Msel::Input1),
            2 => Some(Msel::Input2),
            3 => Some(Msel::Input3),
            4 => Some(Msel::Input4),
            5 => Some(Msel::Input5),
            7 => Some(Msel::Input7),
            _ => None,
        }
    }
    #[doc = "Input 0m"]
    #[inline(always)]
    pub fn is_input_0(&self) -> bool {
        *self == Msel::Input0
    }
    #[doc = "Input 1m"]
    #[inline(always)]
    pub fn is_input_1(&self) -> bool {
        *self == Msel::Input1
    }
    #[doc = "Input 2m"]
    #[inline(always)]
    pub fn is_input_2(&self) -> bool {
        *self == Msel::Input2
    }
    #[doc = "Input 3m"]
    #[inline(always)]
    pub fn is_input_3(&self) -> bool {
        *self == Msel::Input3
    }
    #[doc = "Input 4m"]
    #[inline(always)]
    pub fn is_input_4(&self) -> bool {
        *self == Msel::Input4
    }
    #[doc = "Input 5m"]
    #[inline(always)]
    pub fn is_input_5(&self) -> bool {
        *self == Msel::Input5
    }
    #[doc = "Internal DAC output"]
    #[inline(always)]
    pub fn is_input_7(&self) -> bool {
        *self == Msel::Input7
    }
}
#[doc = "Field `MSEL` writer - Minus Input MUX Select"]
pub type MselW<'a, REG> = crate::FieldWriter<'a, REG, 3, Msel>;
impl<'a, REG> MselW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Input 0m"]
    #[inline(always)]
    pub fn input_0(self) -> &'a mut crate::W<REG> {
        self.variant(Msel::Input0)
    }
    #[doc = "Input 1m"]
    #[inline(always)]
    pub fn input_1(self) -> &'a mut crate::W<REG> {
        self.variant(Msel::Input1)
    }
    #[doc = "Input 2m"]
    #[inline(always)]
    pub fn input_2(self) -> &'a mut crate::W<REG> {
        self.variant(Msel::Input2)
    }
    #[doc = "Input 3m"]
    #[inline(always)]
    pub fn input_3(self) -> &'a mut crate::W<REG> {
        self.variant(Msel::Input3)
    }
    #[doc = "Input 4m"]
    #[inline(always)]
    pub fn input_4(self) -> &'a mut crate::W<REG> {
        self.variant(Msel::Input4)
    }
    #[doc = "Input 5m"]
    #[inline(always)]
    pub fn input_5(self) -> &'a mut crate::W<REG> {
        self.variant(Msel::Input5)
    }
    #[doc = "Internal DAC output"]
    #[inline(always)]
    pub fn input_7(self) -> &'a mut crate::W<REG> {
        self.variant(Msel::Input7)
    }
}
impl R {
    #[doc = "Bit 0 - CMP High Power Mode Select"]
    #[inline(always)]
    pub fn cmp_hpmd(&self) -> CmpHpmdR {
        CmpHpmdR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - CMP Nano Power Mode Select"]
    #[inline(always)]
    pub fn cmp_npmd(&self) -> CmpNpmdR {
        CmpNpmdR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 4:5 - Comparator Hysteresis Control"]
    #[inline(always)]
    pub fn hystctr(&self) -> HystctrR {
        HystctrR::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 16:18 - Plus Input MUX Select"]
    #[inline(always)]
    pub fn psel(&self) -> PselR {
        PselR::new(((self.bits >> 16) & 7) as u8)
    }
    #[doc = "Bits 20:22 - Minus Input MUX Select"]
    #[inline(always)]
    pub fn msel(&self) -> MselR {
        MselR::new(((self.bits >> 20) & 7) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - CMP High Power Mode Select"]
    #[inline(always)]
    pub fn cmp_hpmd(&mut self) -> CmpHpmdW<Ccr2Spec> {
        CmpHpmdW::new(self, 0)
    }
    #[doc = "Bit 1 - CMP Nano Power Mode Select"]
    #[inline(always)]
    pub fn cmp_npmd(&mut self) -> CmpNpmdW<Ccr2Spec> {
        CmpNpmdW::new(self, 1)
    }
    #[doc = "Bits 4:5 - Comparator Hysteresis Control"]
    #[inline(always)]
    pub fn hystctr(&mut self) -> HystctrW<Ccr2Spec> {
        HystctrW::new(self, 4)
    }
    #[doc = "Bits 16:18 - Plus Input MUX Select"]
    #[inline(always)]
    pub fn psel(&mut self) -> PselW<Ccr2Spec> {
        PselW::new(self, 16)
    }
    #[doc = "Bits 20:22 - Minus Input MUX Select"]
    #[inline(always)]
    pub fn msel(&mut self) -> MselW<Ccr2Spec> {
        MselW::new(self, 20)
    }
}
#[doc = "Comparator Control Register 2\n\nYou can [`read`](crate::Reg::read) this register and get [`ccr2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ccr2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct Ccr2Spec;
impl crate::RegisterSpec for Ccr2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ccr2::R`](R) reader structure"]
impl crate::Readable for Ccr2Spec {}
#[doc = "`write(|w| ..)` method takes [`ccr2::W`](W) writer structure"]
impl crate::Writable for Ccr2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets CCR2 to value 0"]
impl crate::Resettable for Ccr2Spec {}

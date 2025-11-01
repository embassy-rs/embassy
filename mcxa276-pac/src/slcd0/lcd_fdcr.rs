#[doc = "Register `LCD_FDCR` reader"]
pub type R = crate::R<LcdFdcrSpec>;
#[doc = "Register `LCD_FDCR` writer"]
pub type W = crate::W<LcdFdcrSpec>;
#[doc = "Field `FDPINID` reader - Fault Detect Pin ID"]
pub type FdpinidR = crate::FieldReader;
#[doc = "Field `FDPINID` writer - Fault Detect Pin ID"]
pub type FdpinidW<'a, REG> = crate::FieldWriter<'a, REG, 6>;
#[doc = "Fault Detect Back Plane Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fdbpen {
    #[doc = "0: Type of the selected pin under fault detect test is front plane."]
    Front = 0,
    #[doc = "1: Type of the selected pin under fault detect test is back plane."]
    Back = 1,
}
impl From<Fdbpen> for bool {
    #[inline(always)]
    fn from(variant: Fdbpen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FDBPEN` reader - Fault Detect Back Plane Enable"]
pub type FdbpenR = crate::BitReader<Fdbpen>;
impl FdbpenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fdbpen {
        match self.bits {
            false => Fdbpen::Front,
            true => Fdbpen::Back,
        }
    }
    #[doc = "Type of the selected pin under fault detect test is front plane."]
    #[inline(always)]
    pub fn is_front(&self) -> bool {
        *self == Fdbpen::Front
    }
    #[doc = "Type of the selected pin under fault detect test is back plane."]
    #[inline(always)]
    pub fn is_back(&self) -> bool {
        *self == Fdbpen::Back
    }
}
#[doc = "Field `FDBPEN` writer - Fault Detect Back Plane Enable"]
pub type FdbpenW<'a, REG> = crate::BitWriter<'a, REG, Fdbpen>;
impl<'a, REG> FdbpenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Type of the selected pin under fault detect test is front plane."]
    #[inline(always)]
    pub fn front(self) -> &'a mut crate::W<REG> {
        self.variant(Fdbpen::Front)
    }
    #[doc = "Type of the selected pin under fault detect test is back plane."]
    #[inline(always)]
    pub fn back(self) -> &'a mut crate::W<REG> {
        self.variant(Fdbpen::Back)
    }
}
#[doc = "Fault Detect Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fden {
    #[doc = "0: Disable fault detection."]
    Disable = 0,
    #[doc = "1: Enable fault detection."]
    Enable = 1,
}
impl From<Fden> for bool {
    #[inline(always)]
    fn from(variant: Fden) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FDEN` reader - Fault Detect Enable"]
pub type FdenR = crate::BitReader<Fden>;
impl FdenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fden {
        match self.bits {
            false => Fden::Disable,
            true => Fden::Enable,
        }
    }
    #[doc = "Disable fault detection."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Fden::Disable
    }
    #[doc = "Enable fault detection."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Fden::Enable
    }
}
#[doc = "Field `FDEN` writer - Fault Detect Enable"]
pub type FdenW<'a, REG> = crate::BitWriter<'a, REG, Fden>;
impl<'a, REG> FdenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable fault detection."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Fden::Disable)
    }
    #[doc = "Enable fault detection."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Fden::Enable)
    }
}
#[doc = "Fault Detect Sample Window Width\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Fdsww {
    #[doc = "0: Sample window width is 4 sample clock cycles."]
    Sample4Cycles = 0,
    #[doc = "1: Sample window width is 8 sample clock cycles."]
    Sample8Cycles = 1,
    #[doc = "2: Sample window width is 16 sample clock cycles."]
    Sample16Cycles = 2,
    #[doc = "3: Sample window width is 32 sample clock cycles."]
    Sample32Cycles = 3,
    #[doc = "4: Sample window width is 64 sample clock cycles."]
    Sample64Cycles = 4,
    #[doc = "5: Sample window width is 128 sample clock cycles."]
    Sample128Cycles = 5,
    #[doc = "6: Sample window width is 256 sample clock cycles."]
    Sample256Cycles = 6,
    #[doc = "7: Sample window width is 512 sample clock cycles."]
    Sample512Cycles = 7,
}
impl From<Fdsww> for u8 {
    #[inline(always)]
    fn from(variant: Fdsww) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Fdsww {
    type Ux = u8;
}
impl crate::IsEnum for Fdsww {}
#[doc = "Field `FDSWW` reader - Fault Detect Sample Window Width"]
pub type FdswwR = crate::FieldReader<Fdsww>;
impl FdswwR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fdsww {
        match self.bits {
            0 => Fdsww::Sample4Cycles,
            1 => Fdsww::Sample8Cycles,
            2 => Fdsww::Sample16Cycles,
            3 => Fdsww::Sample32Cycles,
            4 => Fdsww::Sample64Cycles,
            5 => Fdsww::Sample128Cycles,
            6 => Fdsww::Sample256Cycles,
            7 => Fdsww::Sample512Cycles,
            _ => unreachable!(),
        }
    }
    #[doc = "Sample window width is 4 sample clock cycles."]
    #[inline(always)]
    pub fn is_sample_4_cycles(&self) -> bool {
        *self == Fdsww::Sample4Cycles
    }
    #[doc = "Sample window width is 8 sample clock cycles."]
    #[inline(always)]
    pub fn is_sample_8_cycles(&self) -> bool {
        *self == Fdsww::Sample8Cycles
    }
    #[doc = "Sample window width is 16 sample clock cycles."]
    #[inline(always)]
    pub fn is_sample_16_cycles(&self) -> bool {
        *self == Fdsww::Sample16Cycles
    }
    #[doc = "Sample window width is 32 sample clock cycles."]
    #[inline(always)]
    pub fn is_sample_32_cycles(&self) -> bool {
        *self == Fdsww::Sample32Cycles
    }
    #[doc = "Sample window width is 64 sample clock cycles."]
    #[inline(always)]
    pub fn is_sample_64_cycles(&self) -> bool {
        *self == Fdsww::Sample64Cycles
    }
    #[doc = "Sample window width is 128 sample clock cycles."]
    #[inline(always)]
    pub fn is_sample_128_cycles(&self) -> bool {
        *self == Fdsww::Sample128Cycles
    }
    #[doc = "Sample window width is 256 sample clock cycles."]
    #[inline(always)]
    pub fn is_sample_256_cycles(&self) -> bool {
        *self == Fdsww::Sample256Cycles
    }
    #[doc = "Sample window width is 512 sample clock cycles."]
    #[inline(always)]
    pub fn is_sample_512_cycles(&self) -> bool {
        *self == Fdsww::Sample512Cycles
    }
}
#[doc = "Field `FDSWW` writer - Fault Detect Sample Window Width"]
pub type FdswwW<'a, REG> = crate::FieldWriter<'a, REG, 3, Fdsww, crate::Safe>;
impl<'a, REG> FdswwW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Sample window width is 4 sample clock cycles."]
    #[inline(always)]
    pub fn sample_4_cycles(self) -> &'a mut crate::W<REG> {
        self.variant(Fdsww::Sample4Cycles)
    }
    #[doc = "Sample window width is 8 sample clock cycles."]
    #[inline(always)]
    pub fn sample_8_cycles(self) -> &'a mut crate::W<REG> {
        self.variant(Fdsww::Sample8Cycles)
    }
    #[doc = "Sample window width is 16 sample clock cycles."]
    #[inline(always)]
    pub fn sample_16_cycles(self) -> &'a mut crate::W<REG> {
        self.variant(Fdsww::Sample16Cycles)
    }
    #[doc = "Sample window width is 32 sample clock cycles."]
    #[inline(always)]
    pub fn sample_32_cycles(self) -> &'a mut crate::W<REG> {
        self.variant(Fdsww::Sample32Cycles)
    }
    #[doc = "Sample window width is 64 sample clock cycles."]
    #[inline(always)]
    pub fn sample_64_cycles(self) -> &'a mut crate::W<REG> {
        self.variant(Fdsww::Sample64Cycles)
    }
    #[doc = "Sample window width is 128 sample clock cycles."]
    #[inline(always)]
    pub fn sample_128_cycles(self) -> &'a mut crate::W<REG> {
        self.variant(Fdsww::Sample128Cycles)
    }
    #[doc = "Sample window width is 256 sample clock cycles."]
    #[inline(always)]
    pub fn sample_256_cycles(self) -> &'a mut crate::W<REG> {
        self.variant(Fdsww::Sample256Cycles)
    }
    #[doc = "Sample window width is 512 sample clock cycles."]
    #[inline(always)]
    pub fn sample_512_cycles(self) -> &'a mut crate::W<REG> {
        self.variant(Fdsww::Sample512Cycles)
    }
}
#[doc = "Fault Detect Clock Prescaler\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Fdprs {
    #[doc = "0: 1/1 bus clock."]
    Div1Bus = 0,
    #[doc = "1: 1/2 bus clock."]
    Div2Bus = 1,
    #[doc = "2: 1/4 bus clock."]
    Div4Bus = 2,
    #[doc = "3: 1/8 bus clock."]
    Div8Bus = 3,
    #[doc = "4: 1/16 bus clock."]
    Div16Bus = 4,
    #[doc = "5: 1/32 bus clock."]
    Div32Bus = 5,
    #[doc = "6: 1/64 bus clock."]
    Div64Bus = 6,
    #[doc = "7: 1/128 bus clock."]
    Div128Bus = 7,
}
impl From<Fdprs> for u8 {
    #[inline(always)]
    fn from(variant: Fdprs) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Fdprs {
    type Ux = u8;
}
impl crate::IsEnum for Fdprs {}
#[doc = "Field `FDPRS` reader - Fault Detect Clock Prescaler"]
pub type FdprsR = crate::FieldReader<Fdprs>;
impl FdprsR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fdprs {
        match self.bits {
            0 => Fdprs::Div1Bus,
            1 => Fdprs::Div2Bus,
            2 => Fdprs::Div4Bus,
            3 => Fdprs::Div8Bus,
            4 => Fdprs::Div16Bus,
            5 => Fdprs::Div32Bus,
            6 => Fdprs::Div64Bus,
            7 => Fdprs::Div128Bus,
            _ => unreachable!(),
        }
    }
    #[doc = "1/1 bus clock."]
    #[inline(always)]
    pub fn is_div_1_bus(&self) -> bool {
        *self == Fdprs::Div1Bus
    }
    #[doc = "1/2 bus clock."]
    #[inline(always)]
    pub fn is_div_2_bus(&self) -> bool {
        *self == Fdprs::Div2Bus
    }
    #[doc = "1/4 bus clock."]
    #[inline(always)]
    pub fn is_div_4_bus(&self) -> bool {
        *self == Fdprs::Div4Bus
    }
    #[doc = "1/8 bus clock."]
    #[inline(always)]
    pub fn is_div_8_bus(&self) -> bool {
        *self == Fdprs::Div8Bus
    }
    #[doc = "1/16 bus clock."]
    #[inline(always)]
    pub fn is_div_16_bus(&self) -> bool {
        *self == Fdprs::Div16Bus
    }
    #[doc = "1/32 bus clock."]
    #[inline(always)]
    pub fn is_div_32_bus(&self) -> bool {
        *self == Fdprs::Div32Bus
    }
    #[doc = "1/64 bus clock."]
    #[inline(always)]
    pub fn is_div_64_bus(&self) -> bool {
        *self == Fdprs::Div64Bus
    }
    #[doc = "1/128 bus clock."]
    #[inline(always)]
    pub fn is_div_128_bus(&self) -> bool {
        *self == Fdprs::Div128Bus
    }
}
#[doc = "Field `FDPRS` writer - Fault Detect Clock Prescaler"]
pub type FdprsW<'a, REG> = crate::FieldWriter<'a, REG, 3, Fdprs, crate::Safe>;
impl<'a, REG> FdprsW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "1/1 bus clock."]
    #[inline(always)]
    pub fn div_1_bus(self) -> &'a mut crate::W<REG> {
        self.variant(Fdprs::Div1Bus)
    }
    #[doc = "1/2 bus clock."]
    #[inline(always)]
    pub fn div_2_bus(self) -> &'a mut crate::W<REG> {
        self.variant(Fdprs::Div2Bus)
    }
    #[doc = "1/4 bus clock."]
    #[inline(always)]
    pub fn div_4_bus(self) -> &'a mut crate::W<REG> {
        self.variant(Fdprs::Div4Bus)
    }
    #[doc = "1/8 bus clock."]
    #[inline(always)]
    pub fn div_8_bus(self) -> &'a mut crate::W<REG> {
        self.variant(Fdprs::Div8Bus)
    }
    #[doc = "1/16 bus clock."]
    #[inline(always)]
    pub fn div_16_bus(self) -> &'a mut crate::W<REG> {
        self.variant(Fdprs::Div16Bus)
    }
    #[doc = "1/32 bus clock."]
    #[inline(always)]
    pub fn div_32_bus(self) -> &'a mut crate::W<REG> {
        self.variant(Fdprs::Div32Bus)
    }
    #[doc = "1/64 bus clock."]
    #[inline(always)]
    pub fn div_64_bus(self) -> &'a mut crate::W<REG> {
        self.variant(Fdprs::Div64Bus)
    }
    #[doc = "1/128 bus clock."]
    #[inline(always)]
    pub fn div_128_bus(self) -> &'a mut crate::W<REG> {
        self.variant(Fdprs::Div128Bus)
    }
}
impl R {
    #[doc = "Bits 0:5 - Fault Detect Pin ID"]
    #[inline(always)]
    pub fn fdpinid(&self) -> FdpinidR {
        FdpinidR::new((self.bits & 0x3f) as u8)
    }
    #[doc = "Bit 6 - Fault Detect Back Plane Enable"]
    #[inline(always)]
    pub fn fdbpen(&self) -> FdbpenR {
        FdbpenR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Fault Detect Enable"]
    #[inline(always)]
    pub fn fden(&self) -> FdenR {
        FdenR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bits 9:11 - Fault Detect Sample Window Width"]
    #[inline(always)]
    pub fn fdsww(&self) -> FdswwR {
        FdswwR::new(((self.bits >> 9) & 7) as u8)
    }
    #[doc = "Bits 12:14 - Fault Detect Clock Prescaler"]
    #[inline(always)]
    pub fn fdprs(&self) -> FdprsR {
        FdprsR::new(((self.bits >> 12) & 7) as u8)
    }
}
impl W {
    #[doc = "Bits 0:5 - Fault Detect Pin ID"]
    #[inline(always)]
    pub fn fdpinid(&mut self) -> FdpinidW<LcdFdcrSpec> {
        FdpinidW::new(self, 0)
    }
    #[doc = "Bit 6 - Fault Detect Back Plane Enable"]
    #[inline(always)]
    pub fn fdbpen(&mut self) -> FdbpenW<LcdFdcrSpec> {
        FdbpenW::new(self, 6)
    }
    #[doc = "Bit 7 - Fault Detect Enable"]
    #[inline(always)]
    pub fn fden(&mut self) -> FdenW<LcdFdcrSpec> {
        FdenW::new(self, 7)
    }
    #[doc = "Bits 9:11 - Fault Detect Sample Window Width"]
    #[inline(always)]
    pub fn fdsww(&mut self) -> FdswwW<LcdFdcrSpec> {
        FdswwW::new(self, 9)
    }
    #[doc = "Bits 12:14 - Fault Detect Clock Prescaler"]
    #[inline(always)]
    pub fn fdprs(&mut self) -> FdprsW<LcdFdcrSpec> {
        FdprsW::new(self, 12)
    }
}
#[doc = "LCD Fault Detect Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_fdcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_fdcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LcdFdcrSpec;
impl crate::RegisterSpec for LcdFdcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`lcd_fdcr::R`](R) reader structure"]
impl crate::Readable for LcdFdcrSpec {}
#[doc = "`write(|w| ..)` method takes [`lcd_fdcr::W`](W) writer structure"]
impl crate::Writable for LcdFdcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LCD_FDCR to value 0"]
impl crate::Resettable for LcdFdcrSpec {}

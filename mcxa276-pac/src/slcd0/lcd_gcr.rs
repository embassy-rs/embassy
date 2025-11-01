#[doc = "Register `LCD_GCR` reader"]
pub type R = crate::R<LcdGcrSpec>;
#[doc = "Register `LCD_GCR` writer"]
pub type W = crate::W<LcdGcrSpec>;
#[doc = "LCD duty select\n\nValue on reset: 3"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Duty {
    #[doc = "0: Use 1 BP (1/1 duty cycle)."]
    Use1Bp = 0,
    #[doc = "1: Use 2 BP (1/2 duty cycle)."]
    Use2Bp = 1,
    #[doc = "2: Use 3 BP (1/3 duty cycle)."]
    Use3Bp = 2,
    #[doc = "3: Use 4 BP (1/4 duty cycle).(Default)"]
    Use4Bp = 3,
}
impl From<Duty> for u8 {
    #[inline(always)]
    fn from(variant: Duty) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Duty {
    type Ux = u8;
}
impl crate::IsEnum for Duty {}
#[doc = "Field `DUTY` reader - LCD duty select"]
pub type DutyR = crate::FieldReader<Duty>;
impl DutyR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Duty {
        match self.bits {
            0 => Duty::Use1Bp,
            1 => Duty::Use2Bp,
            2 => Duty::Use3Bp,
            3 => Duty::Use4Bp,
            _ => unreachable!(),
        }
    }
    #[doc = "Use 1 BP (1/1 duty cycle)."]
    #[inline(always)]
    pub fn is_use_1_bp(&self) -> bool {
        *self == Duty::Use1Bp
    }
    #[doc = "Use 2 BP (1/2 duty cycle)."]
    #[inline(always)]
    pub fn is_use_2_bp(&self) -> bool {
        *self == Duty::Use2Bp
    }
    #[doc = "Use 3 BP (1/3 duty cycle)."]
    #[inline(always)]
    pub fn is_use_3_bp(&self) -> bool {
        *self == Duty::Use3Bp
    }
    #[doc = "Use 4 BP (1/4 duty cycle).(Default)"]
    #[inline(always)]
    pub fn is_use_4_bp(&self) -> bool {
        *self == Duty::Use4Bp
    }
}
#[doc = "Field `DUTY` writer - LCD duty select"]
pub type DutyW<'a, REG> = crate::FieldWriter<'a, REG, 2, Duty, crate::Safe>;
impl<'a, REG> DutyW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "Use 1 BP (1/1 duty cycle)."]
    #[inline(always)]
    pub fn use_1_bp(self) -> &'a mut crate::W<REG> {
        self.variant(Duty::Use1Bp)
    }
    #[doc = "Use 2 BP (1/2 duty cycle)."]
    #[inline(always)]
    pub fn use_2_bp(self) -> &'a mut crate::W<REG> {
        self.variant(Duty::Use2Bp)
    }
    #[doc = "Use 3 BP (1/3 duty cycle)."]
    #[inline(always)]
    pub fn use_3_bp(self) -> &'a mut crate::W<REG> {
        self.variant(Duty::Use3Bp)
    }
    #[doc = "Use 4 BP (1/4 duty cycle).(Default)"]
    #[inline(always)]
    pub fn use_4_bp(self) -> &'a mut crate::W<REG> {
        self.variant(Duty::Use4Bp)
    }
}
#[doc = "Field `LCLK` reader - LCD Clock Prescaler"]
pub type LclkR = crate::FieldReader;
#[doc = "Field `LCLK` writer - LCD Clock Prescaler"]
pub type LclkW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "LCD Low Power Waveform\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lcdlp {
    #[doc = "0: LCD driver drives standard waveforms."]
    High = 0,
    #[doc = "1: LCD driver drives low-power waveforms."]
    Low = 1,
}
impl From<Lcdlp> for bool {
    #[inline(always)]
    fn from(variant: Lcdlp) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LCDLP` reader - LCD Low Power Waveform"]
pub type LcdlpR = crate::BitReader<Lcdlp>;
impl LcdlpR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lcdlp {
        match self.bits {
            false => Lcdlp::High,
            true => Lcdlp::Low,
        }
    }
    #[doc = "LCD driver drives standard waveforms."]
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        *self == Lcdlp::High
    }
    #[doc = "LCD driver drives low-power waveforms."]
    #[inline(always)]
    pub fn is_low(&self) -> bool {
        *self == Lcdlp::Low
    }
}
#[doc = "Field `LCDLP` writer - LCD Low Power Waveform"]
pub type LcdlpW<'a, REG> = crate::BitWriter<'a, REG, Lcdlp>;
impl<'a, REG> LcdlpW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "LCD driver drives standard waveforms."]
    #[inline(always)]
    pub fn high(self) -> &'a mut crate::W<REG> {
        self.variant(Lcdlp::High)
    }
    #[doc = "LCD driver drives low-power waveforms."]
    #[inline(always)]
    pub fn low(self) -> &'a mut crate::W<REG> {
        self.variant(Lcdlp::Low)
    }
}
#[doc = "Field `LCDEN` reader - LCD Driver Enable"]
pub type LcdenR = crate::BitReader;
#[doc = "Field `LCDEN` writer - LCD Driver Enable"]
pub type LcdenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "LCD Stop\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lcdstp {
    #[doc = "0: Allows the LCD driver to continue running during Stop mode."]
    Enable = 0,
    #[doc = "1: Disables the LCD driver when MCU enters Stop mode."]
    Disable = 1,
}
impl From<Lcdstp> for bool {
    #[inline(always)]
    fn from(variant: Lcdstp) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LCDSTP` reader - LCD Stop"]
pub type LcdstpR = crate::BitReader<Lcdstp>;
impl LcdstpR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lcdstp {
        match self.bits {
            false => Lcdstp::Enable,
            true => Lcdstp::Disable,
        }
    }
    #[doc = "Allows the LCD driver to continue running during Stop mode."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Lcdstp::Enable
    }
    #[doc = "Disables the LCD driver when MCU enters Stop mode."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Lcdstp::Disable
    }
}
#[doc = "Field `LCDSTP` writer - LCD Stop"]
pub type LcdstpW<'a, REG> = crate::BitWriter<'a, REG, Lcdstp>;
impl<'a, REG> LcdstpW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Allows the LCD driver to continue running during Stop mode."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Lcdstp::Enable)
    }
    #[doc = "Disables the LCD driver when MCU enters Stop mode."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Lcdstp::Disable)
    }
}
#[doc = "LCD Doze enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lcddoze {
    #[doc = "0: Allows the LCD driver to continue running during Doze mode."]
    Enable = 0,
    #[doc = "1: Disables the LCD driver when MCU enters Doze mode."]
    Disable = 1,
}
impl From<Lcddoze> for bool {
    #[inline(always)]
    fn from(variant: Lcddoze) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LCDDOZE` reader - LCD Doze enable"]
pub type LcddozeR = crate::BitReader<Lcddoze>;
impl LcddozeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lcddoze {
        match self.bits {
            false => Lcddoze::Enable,
            true => Lcddoze::Disable,
        }
    }
    #[doc = "Allows the LCD driver to continue running during Doze mode."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Lcddoze::Enable
    }
    #[doc = "Disables the LCD driver when MCU enters Doze mode."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Lcddoze::Disable
    }
}
#[doc = "Field `LCDDOZE` writer - LCD Doze enable"]
pub type LcddozeW<'a, REG> = crate::BitWriter<'a, REG, Lcddoze>;
impl<'a, REG> LcddozeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Allows the LCD driver to continue running during Doze mode."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Lcddoze::Enable)
    }
    #[doc = "Disables the LCD driver when MCU enters Doze mode."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Lcddoze::Disable)
    }
}
#[doc = "LCD Fault Detection Complete Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fdcien {
    #[doc = "0: No interrupt request is generated by this event."]
    Disable = 0,
    #[doc = "1: When a fault is detected and FDCF bit is set, this event causes an interrupt request."]
    Enable = 1,
}
impl From<Fdcien> for bool {
    #[inline(always)]
    fn from(variant: Fdcien) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `FDCIEN` reader - LCD Fault Detection Complete Interrupt Enable"]
pub type FdcienR = crate::BitReader<Fdcien>;
impl FdcienR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Fdcien {
        match self.bits {
            false => Fdcien::Disable,
            true => Fdcien::Enable,
        }
    }
    #[doc = "No interrupt request is generated by this event."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Fdcien::Disable
    }
    #[doc = "When a fault is detected and FDCF bit is set, this event causes an interrupt request."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Fdcien::Enable
    }
}
#[doc = "Field `FDCIEN` writer - LCD Fault Detection Complete Interrupt Enable"]
pub type FdcienW<'a, REG> = crate::BitWriter<'a, REG, Fdcien>;
impl<'a, REG> FdcienW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No interrupt request is generated by this event."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Fdcien::Disable)
    }
    #[doc = "When a fault is detected and FDCF bit is set, this event causes an interrupt request."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Fdcien::Enable)
    }
}
#[doc = "LCD Frame Frequency Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lcdien {
    #[doc = "0: No interrupt request is generated by this event."]
    Disable = 0,
    #[doc = "1: When LCDIF bit is set, this event causes an interrupt request."]
    Enable = 1,
}
impl From<Lcdien> for bool {
    #[inline(always)]
    fn from(variant: Lcdien) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LCDIEN` reader - LCD Frame Frequency Interrupt Enable"]
pub type LcdienR = crate::BitReader<Lcdien>;
impl LcdienR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lcdien {
        match self.bits {
            false => Lcdien::Disable,
            true => Lcdien::Enable,
        }
    }
    #[doc = "No interrupt request is generated by this event."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Lcdien::Disable
    }
    #[doc = "When LCDIF bit is set, this event causes an interrupt request."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Lcdien::Enable
    }
}
#[doc = "Field `LCDIEN` writer - LCD Frame Frequency Interrupt Enable"]
pub type LcdienW<'a, REG> = crate::BitWriter<'a, REG, Lcdien>;
impl<'a, REG> LcdienW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No interrupt request is generated by this event."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Lcdien::Disable)
    }
    #[doc = "When LCDIF bit is set, this event causes an interrupt request."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Lcdien::Enable)
    }
}
#[doc = "Sample & Hold Cycle Select\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shcycle {
    #[doc = "0: Sample & hold phase clock period is 64 LCD clock (16kHz) period / 32 LCD clock (8kHz) period."]
    Clk64 = 0,
    #[doc = "1: Sample & hold phase clock period is 128 LCD clk (16kHz) period / 64 LCD clock (8kHz) period."]
    Clk128 = 1,
}
impl From<Shcycle> for bool {
    #[inline(always)]
    fn from(variant: Shcycle) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SHCYCLE` reader - Sample & Hold Cycle Select"]
pub type ShcycleR = crate::BitReader<Shcycle>;
impl ShcycleR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Shcycle {
        match self.bits {
            false => Shcycle::Clk64,
            true => Shcycle::Clk128,
        }
    }
    #[doc = "Sample & hold phase clock period is 64 LCD clock (16kHz) period / 32 LCD clock (8kHz) period."]
    #[inline(always)]
    pub fn is_clk64(&self) -> bool {
        *self == Shcycle::Clk64
    }
    #[doc = "Sample & hold phase clock period is 128 LCD clk (16kHz) period / 64 LCD clock (8kHz) period."]
    #[inline(always)]
    pub fn is_clk128(&self) -> bool {
        *self == Shcycle::Clk128
    }
}
#[doc = "Field `SHCYCLE` writer - Sample & Hold Cycle Select"]
pub type ShcycleW<'a, REG> = crate::BitWriter<'a, REG, Shcycle>;
impl<'a, REG> ShcycleW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Sample & hold phase clock period is 64 LCD clock (16kHz) period / 32 LCD clock (8kHz) period."]
    #[inline(always)]
    pub fn clk64(self) -> &'a mut crate::W<REG> {
        self.variant(Shcycle::Clk64)
    }
    #[doc = "Sample & hold phase clock period is 128 LCD clk (16kHz) period / 64 LCD clock (8kHz) period."]
    #[inline(always)]
    pub fn clk128(self) -> &'a mut crate::W<REG> {
        self.variant(Shcycle::Clk128)
    }
}
#[doc = "Sample & Hold Mode Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shen {
    #[doc = "0: Sample & hold is disabled."]
    Disable = 0,
    #[doc = "1: Sample & hold is enabled."]
    Enable = 1,
}
impl From<Shen> for bool {
    #[inline(always)]
    fn from(variant: Shen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `SHEN` reader - Sample & Hold Mode Enable"]
pub type ShenR = crate::BitReader<Shen>;
impl ShenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Shen {
        match self.bits {
            false => Shen::Disable,
            true => Shen::Enable,
        }
    }
    #[doc = "Sample & hold is disabled."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Shen::Disable
    }
    #[doc = "Sample & hold is enabled."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Shen::Enable
    }
}
#[doc = "Field `SHEN` writer - Sample & Hold Mode Enable"]
pub type ShenW<'a, REG> = crate::BitWriter<'a, REG, Shen>;
impl<'a, REG> ShenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Sample & hold is disabled."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Shen::Disable)
    }
    #[doc = "Sample & hold is enabled."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Shen::Enable)
    }
}
#[doc = "Field `VLL1TRIM` reader - Level 1 Voltage Trim"]
pub type Vll1trimR = crate::FieldReader;
#[doc = "Field `VLL1TRIM` writer - Level 1 Voltage Trim"]
pub type Vll1trimW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `VLL2TRIM` reader - Level 2 Voltage Trim"]
pub type Vll2trimR = crate::FieldReader;
#[doc = "Field `VLL2TRIM` writer - Level 2 Voltage Trim"]
pub type Vll2trimW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:1 - LCD duty select"]
    #[inline(always)]
    pub fn duty(&self) -> DutyR {
        DutyR::new((self.bits & 3) as u8)
    }
    #[doc = "Bits 3:5 - LCD Clock Prescaler"]
    #[inline(always)]
    pub fn lclk(&self) -> LclkR {
        LclkR::new(((self.bits >> 3) & 7) as u8)
    }
    #[doc = "Bit 6 - LCD Low Power Waveform"]
    #[inline(always)]
    pub fn lcdlp(&self) -> LcdlpR {
        LcdlpR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - LCD Driver Enable"]
    #[inline(always)]
    pub fn lcden(&self) -> LcdenR {
        LcdenR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - LCD Stop"]
    #[inline(always)]
    pub fn lcdstp(&self) -> LcdstpR {
        LcdstpR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - LCD Doze enable"]
    #[inline(always)]
    pub fn lcddoze(&self) -> LcddozeR {
        LcddozeR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 14 - LCD Fault Detection Complete Interrupt Enable"]
    #[inline(always)]
    pub fn fdcien(&self) -> FdcienR {
        FdcienR::new(((self.bits >> 14) & 1) != 0)
    }
    #[doc = "Bit 15 - LCD Frame Frequency Interrupt Enable"]
    #[inline(always)]
    pub fn lcdien(&self) -> LcdienR {
        LcdienR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - Sample & Hold Cycle Select"]
    #[inline(always)]
    pub fn shcycle(&self) -> ShcycleR {
        ShcycleR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 23 - Sample & Hold Mode Enable"]
    #[inline(always)]
    pub fn shen(&self) -> ShenR {
        ShenR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bits 24:27 - Level 1 Voltage Trim"]
    #[inline(always)]
    pub fn vll1trim(&self) -> Vll1trimR {
        Vll1trimR::new(((self.bits >> 24) & 0x0f) as u8)
    }
    #[doc = "Bits 28:31 - Level 2 Voltage Trim"]
    #[inline(always)]
    pub fn vll2trim(&self) -> Vll2trimR {
        Vll2trimR::new(((self.bits >> 28) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:1 - LCD duty select"]
    #[inline(always)]
    pub fn duty(&mut self) -> DutyW<LcdGcrSpec> {
        DutyW::new(self, 0)
    }
    #[doc = "Bits 3:5 - LCD Clock Prescaler"]
    #[inline(always)]
    pub fn lclk(&mut self) -> LclkW<LcdGcrSpec> {
        LclkW::new(self, 3)
    }
    #[doc = "Bit 6 - LCD Low Power Waveform"]
    #[inline(always)]
    pub fn lcdlp(&mut self) -> LcdlpW<LcdGcrSpec> {
        LcdlpW::new(self, 6)
    }
    #[doc = "Bit 7 - LCD Driver Enable"]
    #[inline(always)]
    pub fn lcden(&mut self) -> LcdenW<LcdGcrSpec> {
        LcdenW::new(self, 7)
    }
    #[doc = "Bit 8 - LCD Stop"]
    #[inline(always)]
    pub fn lcdstp(&mut self) -> LcdstpW<LcdGcrSpec> {
        LcdstpW::new(self, 8)
    }
    #[doc = "Bit 9 - LCD Doze enable"]
    #[inline(always)]
    pub fn lcddoze(&mut self) -> LcddozeW<LcdGcrSpec> {
        LcddozeW::new(self, 9)
    }
    #[doc = "Bit 14 - LCD Fault Detection Complete Interrupt Enable"]
    #[inline(always)]
    pub fn fdcien(&mut self) -> FdcienW<LcdGcrSpec> {
        FdcienW::new(self, 14)
    }
    #[doc = "Bit 15 - LCD Frame Frequency Interrupt Enable"]
    #[inline(always)]
    pub fn lcdien(&mut self) -> LcdienW<LcdGcrSpec> {
        LcdienW::new(self, 15)
    }
    #[doc = "Bit 16 - Sample & Hold Cycle Select"]
    #[inline(always)]
    pub fn shcycle(&mut self) -> ShcycleW<LcdGcrSpec> {
        ShcycleW::new(self, 16)
    }
    #[doc = "Bit 23 - Sample & Hold Mode Enable"]
    #[inline(always)]
    pub fn shen(&mut self) -> ShenW<LcdGcrSpec> {
        ShenW::new(self, 23)
    }
    #[doc = "Bits 24:27 - Level 1 Voltage Trim"]
    #[inline(always)]
    pub fn vll1trim(&mut self) -> Vll1trimW<LcdGcrSpec> {
        Vll1trimW::new(self, 24)
    }
    #[doc = "Bits 28:31 - Level 2 Voltage Trim"]
    #[inline(always)]
    pub fn vll2trim(&mut self) -> Vll2trimW<LcdGcrSpec> {
        Vll2trimW::new(self, 28)
    }
}
#[doc = "LCD General Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_gcr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_gcr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LcdGcrSpec;
impl crate::RegisterSpec for LcdGcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`lcd_gcr::R`](R) reader structure"]
impl crate::Readable for LcdGcrSpec {}
#[doc = "`write(|w| ..)` method takes [`lcd_gcr::W`](W) writer structure"]
impl crate::Writable for LcdGcrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets LCD_GCR to value 0x03"]
impl crate::Resettable for LcdGcrSpec {
    const RESET_VALUE: u32 = 0x03;
}

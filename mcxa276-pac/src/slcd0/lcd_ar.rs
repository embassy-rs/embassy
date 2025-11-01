#[doc = "Register `LCD_AR` reader"]
pub type R = crate::R<LcdArSpec>;
#[doc = "Register `LCD_AR` writer"]
pub type W = crate::W<LcdArSpec>;
#[doc = "Field `BRATE` reader - Blink-rate configuration"]
pub type BrateR = crate::FieldReader;
#[doc = "Field `BRATE` writer - Blink-rate configuration"]
pub type BrateW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Blink mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bmode {
    #[doc = "0: Display blank during the blink period."]
    Blank = 0,
    #[doc = "1: Display alternate display during blink period."]
    Alternate = 1,
}
impl From<Bmode> for bool {
    #[inline(always)]
    fn from(variant: Bmode) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BMODE` reader - Blink mode"]
pub type BmodeR = crate::BitReader<Bmode>;
impl BmodeR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Bmode {
        match self.bits {
            false => Bmode::Blank,
            true => Bmode::Alternate,
        }
    }
    #[doc = "Display blank during the blink period."]
    #[inline(always)]
    pub fn is_blank(&self) -> bool {
        *self == Bmode::Blank
    }
    #[doc = "Display alternate display during blink period."]
    #[inline(always)]
    pub fn is_alternate(&self) -> bool {
        *self == Bmode::Alternate
    }
}
#[doc = "Field `BMODE` writer - Blink mode"]
pub type BmodeW<'a, REG> = crate::BitWriter<'a, REG, Bmode>;
impl<'a, REG> BmodeW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Display blank during the blink period."]
    #[inline(always)]
    pub fn blank(self) -> &'a mut crate::W<REG> {
        self.variant(Bmode::Blank)
    }
    #[doc = "Display alternate display during blink period."]
    #[inline(always)]
    pub fn alternate(self) -> &'a mut crate::W<REG> {
        self.variant(Bmode::Alternate)
    }
}
#[doc = "Blank display mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blank {
    #[doc = "0: Normal or alternate display mode."]
    NotBlank = 0,
    #[doc = "1: Blank display mode."]
    Blank = 1,
}
impl From<Blank> for bool {
    #[inline(always)]
    fn from(variant: Blank) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BLANK` reader - Blank display mode"]
pub type BlankR = crate::BitReader<Blank>;
impl BlankR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Blank {
        match self.bits {
            false => Blank::NotBlank,
            true => Blank::Blank,
        }
    }
    #[doc = "Normal or alternate display mode."]
    #[inline(always)]
    pub fn is_not_blank(&self) -> bool {
        *self == Blank::NotBlank
    }
    #[doc = "Blank display mode."]
    #[inline(always)]
    pub fn is_blank(&self) -> bool {
        *self == Blank::Blank
    }
}
#[doc = "Field `BLANK` writer - Blank display mode"]
pub type BlankW<'a, REG> = crate::BitWriter<'a, REG, Blank>;
impl<'a, REG> BlankW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Normal or alternate display mode."]
    #[inline(always)]
    pub fn not_blank(self) -> &'a mut crate::W<REG> {
        self.variant(Blank::NotBlank)
    }
    #[doc = "Blank display mode."]
    #[inline(always)]
    pub fn blank(self) -> &'a mut crate::W<REG> {
        self.variant(Blank::Blank)
    }
}
#[doc = "Alternate display mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alt {
    #[doc = "0: Normal display mode."]
    Normal = 0,
    #[doc = "1: Alternate display mode."]
    Alternate = 1,
}
impl From<Alt> for bool {
    #[inline(always)]
    fn from(variant: Alt) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ALT` reader - Alternate display mode"]
pub type AltR = crate::BitReader<Alt>;
impl AltR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Alt {
        match self.bits {
            false => Alt::Normal,
            true => Alt::Alternate,
        }
    }
    #[doc = "Normal display mode."]
    #[inline(always)]
    pub fn is_normal(&self) -> bool {
        *self == Alt::Normal
    }
    #[doc = "Alternate display mode."]
    #[inline(always)]
    pub fn is_alternate(&self) -> bool {
        *self == Alt::Alternate
    }
}
#[doc = "Field `ALT` writer - Alternate display mode"]
pub type AltW<'a, REG> = crate::BitWriter<'a, REG, Alt>;
impl<'a, REG> AltW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Normal display mode."]
    #[inline(always)]
    pub fn normal(self) -> &'a mut crate::W<REG> {
        self.variant(Alt::Normal)
    }
    #[doc = "Alternate display mode."]
    #[inline(always)]
    pub fn alternate(self) -> &'a mut crate::W<REG> {
        self.variant(Alt::Alternate)
    }
}
#[doc = "Blink command\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blink {
    #[doc = "0: Disables blinking."]
    Disable = 0,
    #[doc = "1: Starts blinking at blinking frequency specified by LCD blink rate calculation."]
    Enable = 1,
}
impl From<Blink> for bool {
    #[inline(always)]
    fn from(variant: Blink) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `BLINK` reader - Blink command"]
pub type BlinkR = crate::BitReader<Blink>;
impl BlinkR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Blink {
        match self.bits {
            false => Blink::Disable,
            true => Blink::Enable,
        }
    }
    #[doc = "Disables blinking."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Blink::Disable
    }
    #[doc = "Starts blinking at blinking frequency specified by LCD blink rate calculation."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Blink::Enable
    }
}
#[doc = "Field `BLINK` writer - Blink command"]
pub type BlinkW<'a, REG> = crate::BitWriter<'a, REG, Blink>;
impl<'a, REG> BlinkW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables blinking."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Blink::Disable)
    }
    #[doc = "Starts blinking at blinking frequency specified by LCD blink rate calculation."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Blink::Enable)
    }
}
#[doc = "LCD Frame Frequency Interrupt flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lcdif {
    #[doc = "0: Frame frequency interrupt condition has not occurred."]
    Disable = 0,
    #[doc = "1: Start of SLCD frame has occurred."]
    Enable = 1,
}
impl From<Lcdif> for bool {
    #[inline(always)]
    fn from(variant: Lcdif) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `LCDIF` reader - LCD Frame Frequency Interrupt flag"]
pub type LcdifR = crate::BitReader<Lcdif>;
impl LcdifR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Lcdif {
        match self.bits {
            false => Lcdif::Disable,
            true => Lcdif::Enable,
        }
    }
    #[doc = "Frame frequency interrupt condition has not occurred."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Lcdif::Disable
    }
    #[doc = "Start of SLCD frame has occurred."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Lcdif::Enable
    }
}
#[doc = "Field `LCDIF` writer - LCD Frame Frequency Interrupt flag"]
pub type LcdifW<'a, REG> = crate::BitWriter1C<'a, REG, Lcdif>;
impl<'a, REG> LcdifW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Frame frequency interrupt condition has not occurred."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Lcdif::Disable)
    }
    #[doc = "Start of SLCD frame has occurred."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Lcdif::Enable)
    }
}
impl R {
    #[doc = "Bits 0:2 - Blink-rate configuration"]
    #[inline(always)]
    pub fn brate(&self) -> BrateR {
        BrateR::new((self.bits & 7) as u8)
    }
    #[doc = "Bit 3 - Blink mode"]
    #[inline(always)]
    pub fn bmode(&self) -> BmodeR {
        BmodeR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 5 - Blank display mode"]
    #[inline(always)]
    pub fn blank(&self) -> BlankR {
        BlankR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Alternate display mode"]
    #[inline(always)]
    pub fn alt(&self) -> AltR {
        AltR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Blink command"]
    #[inline(always)]
    pub fn blink(&self) -> BlinkR {
        BlinkR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 15 - LCD Frame Frequency Interrupt flag"]
    #[inline(always)]
    pub fn lcdif(&self) -> LcdifR {
        LcdifR::new(((self.bits >> 15) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:2 - Blink-rate configuration"]
    #[inline(always)]
    pub fn brate(&mut self) -> BrateW<LcdArSpec> {
        BrateW::new(self, 0)
    }
    #[doc = "Bit 3 - Blink mode"]
    #[inline(always)]
    pub fn bmode(&mut self) -> BmodeW<LcdArSpec> {
        BmodeW::new(self, 3)
    }
    #[doc = "Bit 5 - Blank display mode"]
    #[inline(always)]
    pub fn blank(&mut self) -> BlankW<LcdArSpec> {
        BlankW::new(self, 5)
    }
    #[doc = "Bit 6 - Alternate display mode"]
    #[inline(always)]
    pub fn alt(&mut self) -> AltW<LcdArSpec> {
        AltW::new(self, 6)
    }
    #[doc = "Bit 7 - Blink command"]
    #[inline(always)]
    pub fn blink(&mut self) -> BlinkW<LcdArSpec> {
        BlinkW::new(self, 7)
    }
    #[doc = "Bit 15 - LCD Frame Frequency Interrupt flag"]
    #[inline(always)]
    pub fn lcdif(&mut self) -> LcdifW<LcdArSpec> {
        LcdifW::new(self, 15)
    }
}
#[doc = "LCD Auxiliary Register\n\nYou can [`read`](crate::Reg::read) this register and get [`lcd_ar::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`lcd_ar::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct LcdArSpec;
impl crate::RegisterSpec for LcdArSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`lcd_ar::R`](R) reader structure"]
impl crate::Readable for LcdArSpec {}
#[doc = "`write(|w| ..)` method takes [`lcd_ar::W`](W) writer structure"]
impl crate::Writable for LcdArSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x8000;
}
#[doc = "`reset()` method sets LCD_AR to value 0"]
impl crate::Resettable for LcdArSpec {}

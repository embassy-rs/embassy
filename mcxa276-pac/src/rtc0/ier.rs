#[doc = "Register `IER` reader"]
pub type R = crate::R<IerSpec>;
#[doc = "Register `IER` writer"]
pub type W = crate::W<IerSpec>;
#[doc = "Time Invalid Interrupt Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tiie {
    #[doc = "0: No interrupt is generated."]
    Tiie0 = 0,
    #[doc = "1: An interrupt is generated."]
    Tiie1 = 1,
}
impl From<Tiie> for bool {
    #[inline(always)]
    fn from(variant: Tiie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TIIE` reader - Time Invalid Interrupt Enable"]
pub type TiieR = crate::BitReader<Tiie>;
impl TiieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tiie {
        match self.bits {
            false => Tiie::Tiie0,
            true => Tiie::Tiie1,
        }
    }
    #[doc = "No interrupt is generated."]
    #[inline(always)]
    pub fn is_tiie_0(&self) -> bool {
        *self == Tiie::Tiie0
    }
    #[doc = "An interrupt is generated."]
    #[inline(always)]
    pub fn is_tiie_1(&self) -> bool {
        *self == Tiie::Tiie1
    }
}
#[doc = "Field `TIIE` writer - Time Invalid Interrupt Enable"]
pub type TiieW<'a, REG> = crate::BitWriter<'a, REG, Tiie>;
impl<'a, REG> TiieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No interrupt is generated."]
    #[inline(always)]
    pub fn tiie_0(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie::Tiie0)
    }
    #[doc = "An interrupt is generated."]
    #[inline(always)]
    pub fn tiie_1(self) -> &'a mut crate::W<REG> {
        self.variant(Tiie::Tiie1)
    }
}
#[doc = "Time Overflow Interrupt Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toie {
    #[doc = "0: No interrupt is generated."]
    Toie0 = 0,
    #[doc = "1: An interrupt is generated."]
    Toie1 = 1,
}
impl From<Toie> for bool {
    #[inline(always)]
    fn from(variant: Toie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TOIE` reader - Time Overflow Interrupt Enable"]
pub type ToieR = crate::BitReader<Toie>;
impl ToieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Toie {
        match self.bits {
            false => Toie::Toie0,
            true => Toie::Toie1,
        }
    }
    #[doc = "No interrupt is generated."]
    #[inline(always)]
    pub fn is_toie_0(&self) -> bool {
        *self == Toie::Toie0
    }
    #[doc = "An interrupt is generated."]
    #[inline(always)]
    pub fn is_toie_1(&self) -> bool {
        *self == Toie::Toie1
    }
}
#[doc = "Field `TOIE` writer - Time Overflow Interrupt Enable"]
pub type ToieW<'a, REG> = crate::BitWriter<'a, REG, Toie>;
impl<'a, REG> ToieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No interrupt is generated."]
    #[inline(always)]
    pub fn toie_0(self) -> &'a mut crate::W<REG> {
        self.variant(Toie::Toie0)
    }
    #[doc = "An interrupt is generated."]
    #[inline(always)]
    pub fn toie_1(self) -> &'a mut crate::W<REG> {
        self.variant(Toie::Toie1)
    }
}
#[doc = "Time Alarm Interrupt Enable\n\nValue on reset: 1"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Taie {
    #[doc = "0: No interrupt is generated."]
    Taie0 = 0,
    #[doc = "1: An interrupt is generated."]
    Taie1 = 1,
}
impl From<Taie> for bool {
    #[inline(always)]
    fn from(variant: Taie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TAIE` reader - Time Alarm Interrupt Enable"]
pub type TaieR = crate::BitReader<Taie>;
impl TaieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Taie {
        match self.bits {
            false => Taie::Taie0,
            true => Taie::Taie1,
        }
    }
    #[doc = "No interrupt is generated."]
    #[inline(always)]
    pub fn is_taie_0(&self) -> bool {
        *self == Taie::Taie0
    }
    #[doc = "An interrupt is generated."]
    #[inline(always)]
    pub fn is_taie_1(&self) -> bool {
        *self == Taie::Taie1
    }
}
#[doc = "Field `TAIE` writer - Time Alarm Interrupt Enable"]
pub type TaieW<'a, REG> = crate::BitWriter<'a, REG, Taie>;
impl<'a, REG> TaieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No interrupt is generated."]
    #[inline(always)]
    pub fn taie_0(self) -> &'a mut crate::W<REG> {
        self.variant(Taie::Taie0)
    }
    #[doc = "An interrupt is generated."]
    #[inline(always)]
    pub fn taie_1(self) -> &'a mut crate::W<REG> {
        self.variant(Taie::Taie1)
    }
}
#[doc = "Time Seconds Interrupt Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tsie {
    #[doc = "0: Disables"]
    Tsie0 = 0,
    #[doc = "1: Enables"]
    Tsie1 = 1,
}
impl From<Tsie> for bool {
    #[inline(always)]
    fn from(variant: Tsie) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `TSIE` reader - Time Seconds Interrupt Enable"]
pub type TsieR = crate::BitReader<Tsie>;
impl TsieR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tsie {
        match self.bits {
            false => Tsie::Tsie0,
            true => Tsie::Tsie1,
        }
    }
    #[doc = "Disables"]
    #[inline(always)]
    pub fn is_tsie_0(&self) -> bool {
        *self == Tsie::Tsie0
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn is_tsie_1(&self) -> bool {
        *self == Tsie::Tsie1
    }
}
#[doc = "Field `TSIE` writer - Time Seconds Interrupt Enable"]
pub type TsieW<'a, REG> = crate::BitWriter<'a, REG, Tsie>;
impl<'a, REG> TsieW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disables"]
    #[inline(always)]
    pub fn tsie_0(self) -> &'a mut crate::W<REG> {
        self.variant(Tsie::Tsie0)
    }
    #[doc = "Enables"]
    #[inline(always)]
    pub fn tsie_1(self) -> &'a mut crate::W<REG> {
        self.variant(Tsie::Tsie1)
    }
}
#[doc = "Timer Seconds Interrupt Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tsic {
    #[doc = "0: 1 Hz."]
    Tsic0 = 0,
    #[doc = "1: 2 Hz."]
    Tsic1 = 1,
    #[doc = "2: 4 Hz."]
    Tsic2 = 2,
    #[doc = "3: 8 Hz."]
    Tsic3 = 3,
    #[doc = "4: 16 Hz."]
    Tsic4 = 4,
    #[doc = "5: 32 Hz."]
    Tsic5 = 5,
    #[doc = "6: 64 Hz."]
    Tsic6 = 6,
    #[doc = "7: 128 Hz."]
    Tsic7 = 7,
}
impl From<Tsic> for u8 {
    #[inline(always)]
    fn from(variant: Tsic) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Tsic {
    type Ux = u8;
}
impl crate::IsEnum for Tsic {}
#[doc = "Field `TSIC` reader - Timer Seconds Interrupt Configuration"]
pub type TsicR = crate::FieldReader<Tsic>;
impl TsicR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Tsic {
        match self.bits {
            0 => Tsic::Tsic0,
            1 => Tsic::Tsic1,
            2 => Tsic::Tsic2,
            3 => Tsic::Tsic3,
            4 => Tsic::Tsic4,
            5 => Tsic::Tsic5,
            6 => Tsic::Tsic6,
            7 => Tsic::Tsic7,
            _ => unreachable!(),
        }
    }
    #[doc = "1 Hz."]
    #[inline(always)]
    pub fn is_tsic_0(&self) -> bool {
        *self == Tsic::Tsic0
    }
    #[doc = "2 Hz."]
    #[inline(always)]
    pub fn is_tsic_1(&self) -> bool {
        *self == Tsic::Tsic1
    }
    #[doc = "4 Hz."]
    #[inline(always)]
    pub fn is_tsic_2(&self) -> bool {
        *self == Tsic::Tsic2
    }
    #[doc = "8 Hz."]
    #[inline(always)]
    pub fn is_tsic_3(&self) -> bool {
        *self == Tsic::Tsic3
    }
    #[doc = "16 Hz."]
    #[inline(always)]
    pub fn is_tsic_4(&self) -> bool {
        *self == Tsic::Tsic4
    }
    #[doc = "32 Hz."]
    #[inline(always)]
    pub fn is_tsic_5(&self) -> bool {
        *self == Tsic::Tsic5
    }
    #[doc = "64 Hz."]
    #[inline(always)]
    pub fn is_tsic_6(&self) -> bool {
        *self == Tsic::Tsic6
    }
    #[doc = "128 Hz."]
    #[inline(always)]
    pub fn is_tsic_7(&self) -> bool {
        *self == Tsic::Tsic7
    }
}
#[doc = "Field `TSIC` writer - Timer Seconds Interrupt Configuration"]
pub type TsicW<'a, REG> = crate::FieldWriter<'a, REG, 3, Tsic, crate::Safe>;
impl<'a, REG> TsicW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "1 Hz."]
    #[inline(always)]
    pub fn tsic_0(self) -> &'a mut crate::W<REG> {
        self.variant(Tsic::Tsic0)
    }
    #[doc = "2 Hz."]
    #[inline(always)]
    pub fn tsic_1(self) -> &'a mut crate::W<REG> {
        self.variant(Tsic::Tsic1)
    }
    #[doc = "4 Hz."]
    #[inline(always)]
    pub fn tsic_2(self) -> &'a mut crate::W<REG> {
        self.variant(Tsic::Tsic2)
    }
    #[doc = "8 Hz."]
    #[inline(always)]
    pub fn tsic_3(self) -> &'a mut crate::W<REG> {
        self.variant(Tsic::Tsic3)
    }
    #[doc = "16 Hz."]
    #[inline(always)]
    pub fn tsic_4(self) -> &'a mut crate::W<REG> {
        self.variant(Tsic::Tsic4)
    }
    #[doc = "32 Hz."]
    #[inline(always)]
    pub fn tsic_5(self) -> &'a mut crate::W<REG> {
        self.variant(Tsic::Tsic5)
    }
    #[doc = "64 Hz."]
    #[inline(always)]
    pub fn tsic_6(self) -> &'a mut crate::W<REG> {
        self.variant(Tsic::Tsic6)
    }
    #[doc = "128 Hz."]
    #[inline(always)]
    pub fn tsic_7(self) -> &'a mut crate::W<REG> {
        self.variant(Tsic::Tsic7)
    }
}
impl R {
    #[doc = "Bit 0 - Time Invalid Interrupt Enable"]
    #[inline(always)]
    pub fn tiie(&self) -> TiieR {
        TiieR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Time Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn toie(&self) -> ToieR {
        ToieR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Time Alarm Interrupt Enable"]
    #[inline(always)]
    pub fn taie(&self) -> TaieR {
        TaieR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 4 - Time Seconds Interrupt Enable"]
    #[inline(always)]
    pub fn tsie(&self) -> TsieR {
        TsieR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bits 16:18 - Timer Seconds Interrupt Configuration"]
    #[inline(always)]
    pub fn tsic(&self) -> TsicR {
        TsicR::new(((self.bits >> 16) & 7) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Time Invalid Interrupt Enable"]
    #[inline(always)]
    pub fn tiie(&mut self) -> TiieW<IerSpec> {
        TiieW::new(self, 0)
    }
    #[doc = "Bit 1 - Time Overflow Interrupt Enable"]
    #[inline(always)]
    pub fn toie(&mut self) -> ToieW<IerSpec> {
        ToieW::new(self, 1)
    }
    #[doc = "Bit 2 - Time Alarm Interrupt Enable"]
    #[inline(always)]
    pub fn taie(&mut self) -> TaieW<IerSpec> {
        TaieW::new(self, 2)
    }
    #[doc = "Bit 4 - Time Seconds Interrupt Enable"]
    #[inline(always)]
    pub fn tsie(&mut self) -> TsieW<IerSpec> {
        TsieW::new(self, 4)
    }
    #[doc = "Bits 16:18 - Timer Seconds Interrupt Configuration"]
    #[inline(always)]
    pub fn tsic(&mut self) -> TsicW<IerSpec> {
        TsicW::new(self, 16)
    }
}
#[doc = "RTC Interrupt Enable\n\nYou can [`read`](crate::Reg::read) this register and get [`ier::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ier::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IerSpec;
impl crate::RegisterSpec for IerSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ier::R`](R) reader structure"]
impl crate::Readable for IerSpec {}
#[doc = "`write(|w| ..)` method takes [`ier::W`](W) writer structure"]
impl crate::Writable for IerSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets IER to value 0x07"]
impl crate::Resettable for IerSpec {
    const RESET_VALUE: u32 = 0x07;
}

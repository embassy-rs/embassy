#[doc = "Register `ICR[%s]` reader"]
pub type R = crate::R<IcrSpec>;
#[doc = "Register `ICR[%s]` writer"]
pub type W = crate::W<IcrSpec>;
#[doc = "Interrupt Configuration\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Irqc {
    #[doc = "0: ISF is disabled"]
    Irqc0 = 0,
    #[doc = "1: ISF and DMA request on rising edge"]
    Irqc1 = 1,
    #[doc = "2: ISF and DMA request on falling edge"]
    Irqc2 = 2,
    #[doc = "3: ISF and DMA request on either edge"]
    Irqc3 = 3,
    #[doc = "5: ISF sets on rising edge"]
    Irqc5 = 5,
    #[doc = "6: ISF sets on falling edge"]
    Irqc6 = 6,
    #[doc = "7: ISF sets on either edge"]
    Irqc7 = 7,
    #[doc = "8: ISF and interrupt when logic 0"]
    Irqc8 = 8,
    #[doc = "9: ISF and interrupt on rising edge"]
    Irqc9 = 9,
    #[doc = "10: ISF and interrupt on falling edge"]
    Irqc10 = 10,
    #[doc = "11: ISF and Interrupt on either edge"]
    Irqc11 = 11,
    #[doc = "12: ISF and interrupt when logic 1"]
    Irqc12 = 12,
    #[doc = "13: Enable active-high trigger output; ISF on rising edge (pin state is ORed with other enabled triggers to generate the output trigger for use by other peripherals)"]
    Irqc13 = 13,
    #[doc = "14: Enable active-low trigger output; ISF on falling edge (pin state is inverted and ORed with other enabled triggers to generate the output trigger for use by other peripherals)"]
    Irqc14 = 14,
}
impl From<Irqc> for u8 {
    #[inline(always)]
    fn from(variant: Irqc) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Irqc {
    type Ux = u8;
}
impl crate::IsEnum for Irqc {}
#[doc = "Field `IRQC` reader - Interrupt Configuration"]
pub type IrqcR = crate::FieldReader<Irqc>;
impl IrqcR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Irqc> {
        match self.bits {
            0 => Some(Irqc::Irqc0),
            1 => Some(Irqc::Irqc1),
            2 => Some(Irqc::Irqc2),
            3 => Some(Irqc::Irqc3),
            5 => Some(Irqc::Irqc5),
            6 => Some(Irqc::Irqc6),
            7 => Some(Irqc::Irqc7),
            8 => Some(Irqc::Irqc8),
            9 => Some(Irqc::Irqc9),
            10 => Some(Irqc::Irqc10),
            11 => Some(Irqc::Irqc11),
            12 => Some(Irqc::Irqc12),
            13 => Some(Irqc::Irqc13),
            14 => Some(Irqc::Irqc14),
            _ => None,
        }
    }
    #[doc = "ISF is disabled"]
    #[inline(always)]
    pub fn is_irqc0(&self) -> bool {
        *self == Irqc::Irqc0
    }
    #[doc = "ISF and DMA request on rising edge"]
    #[inline(always)]
    pub fn is_irqc1(&self) -> bool {
        *self == Irqc::Irqc1
    }
    #[doc = "ISF and DMA request on falling edge"]
    #[inline(always)]
    pub fn is_irqc2(&self) -> bool {
        *self == Irqc::Irqc2
    }
    #[doc = "ISF and DMA request on either edge"]
    #[inline(always)]
    pub fn is_irqc3(&self) -> bool {
        *self == Irqc::Irqc3
    }
    #[doc = "ISF sets on rising edge"]
    #[inline(always)]
    pub fn is_irqc5(&self) -> bool {
        *self == Irqc::Irqc5
    }
    #[doc = "ISF sets on falling edge"]
    #[inline(always)]
    pub fn is_irqc6(&self) -> bool {
        *self == Irqc::Irqc6
    }
    #[doc = "ISF sets on either edge"]
    #[inline(always)]
    pub fn is_irqc7(&self) -> bool {
        *self == Irqc::Irqc7
    }
    #[doc = "ISF and interrupt when logic 0"]
    #[inline(always)]
    pub fn is_irqc8(&self) -> bool {
        *self == Irqc::Irqc8
    }
    #[doc = "ISF and interrupt on rising edge"]
    #[inline(always)]
    pub fn is_irqc9(&self) -> bool {
        *self == Irqc::Irqc9
    }
    #[doc = "ISF and interrupt on falling edge"]
    #[inline(always)]
    pub fn is_irqc10(&self) -> bool {
        *self == Irqc::Irqc10
    }
    #[doc = "ISF and Interrupt on either edge"]
    #[inline(always)]
    pub fn is_irqc11(&self) -> bool {
        *self == Irqc::Irqc11
    }
    #[doc = "ISF and interrupt when logic 1"]
    #[inline(always)]
    pub fn is_irqc12(&self) -> bool {
        *self == Irqc::Irqc12
    }
    #[doc = "Enable active-high trigger output; ISF on rising edge (pin state is ORed with other enabled triggers to generate the output trigger for use by other peripherals)"]
    #[inline(always)]
    pub fn is_irqc13(&self) -> bool {
        *self == Irqc::Irqc13
    }
    #[doc = "Enable active-low trigger output; ISF on falling edge (pin state is inverted and ORed with other enabled triggers to generate the output trigger for use by other peripherals)"]
    #[inline(always)]
    pub fn is_irqc14(&self) -> bool {
        *self == Irqc::Irqc14
    }
}
#[doc = "Field `IRQC` writer - Interrupt Configuration"]
pub type IrqcW<'a, REG> = crate::FieldWriter<'a, REG, 4, Irqc>;
impl<'a, REG> IrqcW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "ISF is disabled"]
    #[inline(always)]
    pub fn irqc0(self) -> &'a mut crate::W<REG> {
        self.variant(Irqc::Irqc0)
    }
    #[doc = "ISF and DMA request on rising edge"]
    #[inline(always)]
    pub fn irqc1(self) -> &'a mut crate::W<REG> {
        self.variant(Irqc::Irqc1)
    }
    #[doc = "ISF and DMA request on falling edge"]
    #[inline(always)]
    pub fn irqc2(self) -> &'a mut crate::W<REG> {
        self.variant(Irqc::Irqc2)
    }
    #[doc = "ISF and DMA request on either edge"]
    #[inline(always)]
    pub fn irqc3(self) -> &'a mut crate::W<REG> {
        self.variant(Irqc::Irqc3)
    }
    #[doc = "ISF sets on rising edge"]
    #[inline(always)]
    pub fn irqc5(self) -> &'a mut crate::W<REG> {
        self.variant(Irqc::Irqc5)
    }
    #[doc = "ISF sets on falling edge"]
    #[inline(always)]
    pub fn irqc6(self) -> &'a mut crate::W<REG> {
        self.variant(Irqc::Irqc6)
    }
    #[doc = "ISF sets on either edge"]
    #[inline(always)]
    pub fn irqc7(self) -> &'a mut crate::W<REG> {
        self.variant(Irqc::Irqc7)
    }
    #[doc = "ISF and interrupt when logic 0"]
    #[inline(always)]
    pub fn irqc8(self) -> &'a mut crate::W<REG> {
        self.variant(Irqc::Irqc8)
    }
    #[doc = "ISF and interrupt on rising edge"]
    #[inline(always)]
    pub fn irqc9(self) -> &'a mut crate::W<REG> {
        self.variant(Irqc::Irqc9)
    }
    #[doc = "ISF and interrupt on falling edge"]
    #[inline(always)]
    pub fn irqc10(self) -> &'a mut crate::W<REG> {
        self.variant(Irqc::Irqc10)
    }
    #[doc = "ISF and Interrupt on either edge"]
    #[inline(always)]
    pub fn irqc11(self) -> &'a mut crate::W<REG> {
        self.variant(Irqc::Irqc11)
    }
    #[doc = "ISF and interrupt when logic 1"]
    #[inline(always)]
    pub fn irqc12(self) -> &'a mut crate::W<REG> {
        self.variant(Irqc::Irqc12)
    }
    #[doc = "Enable active-high trigger output; ISF on rising edge (pin state is ORed with other enabled triggers to generate the output trigger for use by other peripherals)"]
    #[inline(always)]
    pub fn irqc13(self) -> &'a mut crate::W<REG> {
        self.variant(Irqc::Irqc13)
    }
    #[doc = "Enable active-low trigger output; ISF on falling edge (pin state is inverted and ORed with other enabled triggers to generate the output trigger for use by other peripherals)"]
    #[inline(always)]
    pub fn irqc14(self) -> &'a mut crate::W<REG> {
        self.variant(Irqc::Irqc14)
    }
}
#[doc = "Interrupt Status Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Isf {
    #[doc = "0: Not detected"]
    Isf0 = 0,
    #[doc = "1: Detected"]
    Isf1 = 1,
}
impl From<Isf> for bool {
    #[inline(always)]
    fn from(variant: Isf) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ISF` reader - Interrupt Status Flag"]
pub type IsfR = crate::BitReader<Isf>;
impl IsfR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Isf {
        match self.bits {
            false => Isf::Isf0,
            true => Isf::Isf1,
        }
    }
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn is_isf0(&self) -> bool {
        *self == Isf::Isf0
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn is_isf1(&self) -> bool {
        *self == Isf::Isf1
    }
}
#[doc = "Field `ISF` writer - Interrupt Status Flag"]
pub type IsfW<'a, REG> = crate::BitWriter1C<'a, REG, Isf>;
impl<'a, REG> IsfW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not detected"]
    #[inline(always)]
    pub fn isf0(self) -> &'a mut crate::W<REG> {
        self.variant(Isf::Isf0)
    }
    #[doc = "Detected"]
    #[inline(always)]
    pub fn isf1(self) -> &'a mut crate::W<REG> {
        self.variant(Isf::Isf1)
    }
}
impl R {
    #[doc = "Bits 16:19 - Interrupt Configuration"]
    #[inline(always)]
    pub fn irqc(&self) -> IrqcR {
        IrqcR::new(((self.bits >> 16) & 0x0f) as u8)
    }
    #[doc = "Bit 24 - Interrupt Status Flag"]
    #[inline(always)]
    pub fn isf(&self) -> IsfR {
        IsfR::new(((self.bits >> 24) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 16:19 - Interrupt Configuration"]
    #[inline(always)]
    pub fn irqc(&mut self) -> IrqcW<IcrSpec> {
        IrqcW::new(self, 16)
    }
    #[doc = "Bit 24 - Interrupt Status Flag"]
    #[inline(always)]
    pub fn isf(&mut self) -> IsfW<IcrSpec> {
        IsfW::new(self, 24)
    }
}
#[doc = "Interrupt Control index\n\nYou can [`read`](crate::Reg::read) this register and get [`icr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`icr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IcrSpec;
impl crate::RegisterSpec for IcrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`icr::R`](R) reader structure"]
impl crate::Readable for IcrSpec {}
#[doc = "`write(|w| ..)` method takes [`icr::W`](W) writer structure"]
impl crate::Writable for IcrSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x0100_0000;
}
#[doc = "`reset()` method sets ICR[%s] to value 0"]
impl crate::Resettable for IcrSpec {}

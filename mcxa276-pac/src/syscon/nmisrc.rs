#[doc = "Register `NMISRC` reader"]
pub type R = crate::R<NmisrcSpec>;
#[doc = "Register `NMISRC` writer"]
pub type W = crate::W<NmisrcSpec>;
#[doc = "Field `IRQCPU0` reader - The IRQ number of the interrupt that acts as the Non-Maskable Interrupt (NMI) for CPU0, if enabled by NMIENCPU0."]
pub type Irqcpu0R = crate::FieldReader;
#[doc = "Field `IRQCPU0` writer - The IRQ number of the interrupt that acts as the Non-Maskable Interrupt (NMI) for CPU0, if enabled by NMIENCPU0."]
pub type Irqcpu0W<'a, REG> = crate::FieldWriter<'a, REG, 8>;
#[doc = "Enables the Non-Maskable Interrupt (NMI) source selected by IRQCPU0.\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Nmiencpu0 {
    #[doc = "0: Disable."]
    Disable = 0,
    #[doc = "1: Enable."]
    Enable = 1,
}
impl From<Nmiencpu0> for bool {
    #[inline(always)]
    fn from(variant: Nmiencpu0) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `NMIENCPU0` reader - Enables the Non-Maskable Interrupt (NMI) source selected by IRQCPU0."]
pub type Nmiencpu0R = crate::BitReader<Nmiencpu0>;
impl Nmiencpu0R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Nmiencpu0 {
        match self.bits {
            false => Nmiencpu0::Disable,
            true => Nmiencpu0::Enable,
        }
    }
    #[doc = "Disable."]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == Nmiencpu0::Disable
    }
    #[doc = "Enable."]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == Nmiencpu0::Enable
    }
}
#[doc = "Field `NMIENCPU0` writer - Enables the Non-Maskable Interrupt (NMI) source selected by IRQCPU0."]
pub type Nmiencpu0W<'a, REG> = crate::BitWriter<'a, REG, Nmiencpu0>;
impl<'a, REG> Nmiencpu0W<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Disable."]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(Nmiencpu0::Disable)
    }
    #[doc = "Enable."]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(Nmiencpu0::Enable)
    }
}
impl R {
    #[doc = "Bits 0:7 - The IRQ number of the interrupt that acts as the Non-Maskable Interrupt (NMI) for CPU0, if enabled by NMIENCPU0."]
    #[inline(always)]
    pub fn irqcpu0(&self) -> Irqcpu0R {
        Irqcpu0R::new((self.bits & 0xff) as u8)
    }
    #[doc = "Bit 31 - Enables the Non-Maskable Interrupt (NMI) source selected by IRQCPU0."]
    #[inline(always)]
    pub fn nmiencpu0(&self) -> Nmiencpu0R {
        Nmiencpu0R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:7 - The IRQ number of the interrupt that acts as the Non-Maskable Interrupt (NMI) for CPU0, if enabled by NMIENCPU0."]
    #[inline(always)]
    pub fn irqcpu0(&mut self) -> Irqcpu0W<NmisrcSpec> {
        Irqcpu0W::new(self, 0)
    }
    #[doc = "Bit 31 - Enables the Non-Maskable Interrupt (NMI) source selected by IRQCPU0."]
    #[inline(always)]
    pub fn nmiencpu0(&mut self) -> Nmiencpu0W<NmisrcSpec> {
        Nmiencpu0W::new(self, 31)
    }
}
#[doc = "NMI Source Select\n\nYou can [`read`](crate::Reg::read) this register and get [`nmisrc::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`nmisrc::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct NmisrcSpec;
impl crate::RegisterSpec for NmisrcSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`nmisrc::R`](R) reader structure"]
impl crate::Readable for NmisrcSpec {}
#[doc = "`write(|w| ..)` method takes [`nmisrc::W`](W) writer structure"]
impl crate::Writable for NmisrcSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets NMISRC to value 0"]
impl crate::Resettable for NmisrcSpec {}

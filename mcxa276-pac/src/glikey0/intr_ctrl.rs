#[doc = "Register `INTR_CTRL` reader"]
pub type R = crate::R<IntrCtrlSpec>;
#[doc = "Register `INTR_CTRL` writer"]
pub type W = crate::W<IntrCtrlSpec>;
#[doc = "Field `INT_EN` reader - Interrupt Enable. Writing a 1, Interrupt asserts on Interrupt output port"]
pub type IntEnR = crate::BitReader;
#[doc = "Field `INT_EN` writer - Interrupt Enable. Writing a 1, Interrupt asserts on Interrupt output port"]
pub type IntEnW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `INT_CLR` reader - Interrupt Clear. Writing a 1 to this register creates a single interrupt clear pulse. This register reads as 0"]
pub type IntClrR = crate::BitReader;
#[doc = "Field `INT_CLR` writer - Interrupt Clear. Writing a 1 to this register creates a single interrupt clear pulse. This register reads as 0"]
pub type IntClrW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Interrupt Set. Writing a 1 to this register asserts the interrupt. This register reads as 0\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntSet {
    #[doc = "0: No effect"]
    Disable = 0,
    #[doc = "1: Triggers interrupt"]
    Enable = 1,
}
impl From<IntSet> for bool {
    #[inline(always)]
    fn from(variant: IntSet) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT_SET` reader - Interrupt Set. Writing a 1 to this register asserts the interrupt. This register reads as 0"]
pub type IntSetR = crate::BitReader<IntSet>;
impl IntSetR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> IntSet {
        match self.bits {
            false => IntSet::Disable,
            true => IntSet::Enable,
        }
    }
    #[doc = "No effect"]
    #[inline(always)]
    pub fn is_disable(&self) -> bool {
        *self == IntSet::Disable
    }
    #[doc = "Triggers interrupt"]
    #[inline(always)]
    pub fn is_enable(&self) -> bool {
        *self == IntSet::Enable
    }
}
#[doc = "Field `INT_SET` writer - Interrupt Set. Writing a 1 to this register asserts the interrupt. This register reads as 0"]
pub type IntSetW<'a, REG> = crate::BitWriter<'a, REG, IntSet>;
impl<'a, REG> IntSetW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No effect"]
    #[inline(always)]
    pub fn disable(self) -> &'a mut crate::W<REG> {
        self.variant(IntSet::Disable)
    }
    #[doc = "Triggers interrupt"]
    #[inline(always)]
    pub fn enable(self) -> &'a mut crate::W<REG> {
        self.variant(IntSet::Enable)
    }
}
#[doc = "Field `RESERVED31` reader - Reserved for Future Use"]
pub type Reserved31R = crate::FieldReader<u32>;
impl R {
    #[doc = "Bit 0 - Interrupt Enable. Writing a 1, Interrupt asserts on Interrupt output port"]
    #[inline(always)]
    pub fn int_en(&self) -> IntEnR {
        IntEnR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Interrupt Clear. Writing a 1 to this register creates a single interrupt clear pulse. This register reads as 0"]
    #[inline(always)]
    pub fn int_clr(&self) -> IntClrR {
        IntClrR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Interrupt Set. Writing a 1 to this register asserts the interrupt. This register reads as 0"]
    #[inline(always)]
    pub fn int_set(&self) -> IntSetR {
        IntSetR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bits 3:31 - Reserved for Future Use"]
    #[inline(always)]
    pub fn reserved31(&self) -> Reserved31R {
        Reserved31R::new((self.bits >> 3) & 0x1fff_ffff)
    }
}
impl W {
    #[doc = "Bit 0 - Interrupt Enable. Writing a 1, Interrupt asserts on Interrupt output port"]
    #[inline(always)]
    pub fn int_en(&mut self) -> IntEnW<IntrCtrlSpec> {
        IntEnW::new(self, 0)
    }
    #[doc = "Bit 1 - Interrupt Clear. Writing a 1 to this register creates a single interrupt clear pulse. This register reads as 0"]
    #[inline(always)]
    pub fn int_clr(&mut self) -> IntClrW<IntrCtrlSpec> {
        IntClrW::new(self, 1)
    }
    #[doc = "Bit 2 - Interrupt Set. Writing a 1 to this register asserts the interrupt. This register reads as 0"]
    #[inline(always)]
    pub fn int_set(&mut self) -> IntSetW<IntrCtrlSpec> {
        IntSetW::new(self, 2)
    }
}
#[doc = "Interrupt Control\n\nYou can [`read`](crate::Reg::read) this register and get [`intr_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`intr_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct IntrCtrlSpec;
impl crate::RegisterSpec for IntrCtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`intr_ctrl::R`](R) reader structure"]
impl crate::Readable for IntrCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`intr_ctrl::W`](W) writer structure"]
impl crate::Writable for IntrCtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets INTR_CTRL to value 0"]
impl crate::Resettable for IntrCtrlSpec {}

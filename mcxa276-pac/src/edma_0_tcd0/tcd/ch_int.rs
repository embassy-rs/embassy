#[doc = "Register `CH_INT` reader"]
pub type R = crate::R<ChIntSpec>;
#[doc = "Register `CH_INT` writer"]
pub type W = crate::W<ChIntSpec>;
#[doc = "Interrupt Request\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Int {
    #[doc = "0: Interrupt request for corresponding channel cleared"]
    InterruptCleared = 0,
    #[doc = "1: Interrupt request for corresponding channel active"]
    InterruptActive = 1,
}
impl From<Int> for bool {
    #[inline(always)]
    fn from(variant: Int) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INT` reader - Interrupt Request"]
pub type IntR = crate::BitReader<Int>;
impl IntR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Int {
        match self.bits {
            false => Int::InterruptCleared,
            true => Int::InterruptActive,
        }
    }
    #[doc = "Interrupt request for corresponding channel cleared"]
    #[inline(always)]
    pub fn is_interrupt_cleared(&self) -> bool {
        *self == Int::InterruptCleared
    }
    #[doc = "Interrupt request for corresponding channel active"]
    #[inline(always)]
    pub fn is_interrupt_active(&self) -> bool {
        *self == Int::InterruptActive
    }
}
#[doc = "Field `INT` writer - Interrupt Request"]
pub type IntW<'a, REG> = crate::BitWriter1C<'a, REG, Int>;
impl<'a, REG> IntW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Interrupt request for corresponding channel cleared"]
    #[inline(always)]
    pub fn interrupt_cleared(self) -> &'a mut crate::W<REG> {
        self.variant(Int::InterruptCleared)
    }
    #[doc = "Interrupt request for corresponding channel active"]
    #[inline(always)]
    pub fn interrupt_active(self) -> &'a mut crate::W<REG> {
        self.variant(Int::InterruptActive)
    }
}
impl R {
    #[doc = "Bit 0 - Interrupt Request"]
    #[inline(always)]
    pub fn int(&self) -> IntR {
        IntR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Interrupt Request"]
    #[inline(always)]
    pub fn int(&mut self) -> IntW<ChIntSpec> {
        IntW::new(self, 0)
    }
}
#[doc = "Channel Interrupt Status\n\nYou can [`read`](crate::Reg::read) this register and get [`ch_int::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`ch_int::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ChIntSpec;
impl crate::RegisterSpec for ChIntSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`ch_int::R`](R) reader structure"]
impl crate::Readable for ChIntSpec {}
#[doc = "`write(|w| ..)` method takes [`ch_int::W`](W) writer structure"]
impl crate::Writable for ChIntSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x01;
}
#[doc = "`reset()` method sets CH_INT to value 0"]
impl crate::Resettable for ChIntSpec {}

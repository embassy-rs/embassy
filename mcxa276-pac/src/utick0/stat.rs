#[doc = "Register `STAT` reader"]
pub type R = crate::R<StatSpec>;
#[doc = "Register `STAT` writer"]
pub type W = crate::W<StatSpec>;
#[doc = "Interrupt Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intr {
    #[doc = "0: Not pending"]
    Nopendinginterrupt = 0,
    #[doc = "1: Pending"]
    Pendinginterrupt = 1,
}
impl From<Intr> for bool {
    #[inline(always)]
    fn from(variant: Intr) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `INTR` reader - Interrupt Flag"]
pub type IntrR = crate::BitReader<Intr>;
impl IntrR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Intr {
        match self.bits {
            false => Intr::Nopendinginterrupt,
            true => Intr::Pendinginterrupt,
        }
    }
    #[doc = "Not pending"]
    #[inline(always)]
    pub fn is_nopendinginterrupt(&self) -> bool {
        *self == Intr::Nopendinginterrupt
    }
    #[doc = "Pending"]
    #[inline(always)]
    pub fn is_pendinginterrupt(&self) -> bool {
        *self == Intr::Pendinginterrupt
    }
}
#[doc = "Field `INTR` writer - Interrupt Flag"]
pub type IntrW<'a, REG> = crate::BitWriter1C<'a, REG, Intr>;
impl<'a, REG> IntrW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Not pending"]
    #[inline(always)]
    pub fn nopendinginterrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Intr::Nopendinginterrupt)
    }
    #[doc = "Pending"]
    #[inline(always)]
    pub fn pendinginterrupt(self) -> &'a mut crate::W<REG> {
        self.variant(Intr::Pendinginterrupt)
    }
}
#[doc = "Timer Active Flag\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Active {
    #[doc = "0: Inactive (stopped)"]
    Timerisnotactive = 0,
    #[doc = "1: Active"]
    Timerisactive = 1,
}
impl From<Active> for bool {
    #[inline(always)]
    fn from(variant: Active) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ACTIVE` reader - Timer Active Flag"]
pub type ActiveR = crate::BitReader<Active>;
impl ActiveR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Active {
        match self.bits {
            false => Active::Timerisnotactive,
            true => Active::Timerisactive,
        }
    }
    #[doc = "Inactive (stopped)"]
    #[inline(always)]
    pub fn is_timerisnotactive(&self) -> bool {
        *self == Active::Timerisnotactive
    }
    #[doc = "Active"]
    #[inline(always)]
    pub fn is_timerisactive(&self) -> bool {
        *self == Active::Timerisactive
    }
}
impl R {
    #[doc = "Bit 0 - Interrupt Flag"]
    #[inline(always)]
    pub fn intr(&self) -> IntrR {
        IntrR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Timer Active Flag"]
    #[inline(always)]
    pub fn active(&self) -> ActiveR {
        ActiveR::new(((self.bits >> 1) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Interrupt Flag"]
    #[inline(always)]
    pub fn intr(&mut self) -> IntrW<StatSpec> {
        IntrW::new(self, 0)
    }
}
#[doc = "Status\n\nYou can [`read`](crate::Reg::read) this register and get [`stat::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`stat::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct StatSpec;
impl crate::RegisterSpec for StatSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`stat::R`](R) reader structure"]
impl crate::Readable for StatSpec {}
#[doc = "`write(|w| ..)` method takes [`stat::W`](W) writer structure"]
impl crate::Writable for StatSpec {
    type Safety = crate::Unsafe;
    const ONE_TO_MODIFY_FIELDS_BITMAP: u32 = 0x01;
}
#[doc = "`reset()` method sets STAT to value 0"]
impl crate::Resettable for StatSpec {}

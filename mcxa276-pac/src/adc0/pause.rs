#[doc = "Register `PAUSE` reader"]
pub type R = crate::R<PauseSpec>;
#[doc = "Register `PAUSE` writer"]
pub type W = crate::W<PauseSpec>;
#[doc = "Field `PAUSEDLY` reader - Pause Delay"]
pub type PausedlyR = crate::FieldReader<u16>;
#[doc = "Field `PAUSEDLY` writer - Pause Delay"]
pub type PausedlyW<'a, REG> = crate::FieldWriter<'a, REG, 9, u16>;
#[doc = "PAUSE Option Enable\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pauseen {
    #[doc = "0: Pause operation disabled"]
    Disabled = 0,
    #[doc = "1: Pause operation enabled"]
    Enabled = 1,
}
impl From<Pauseen> for bool {
    #[inline(always)]
    fn from(variant: Pauseen) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `PAUSEEN` reader - PAUSE Option Enable"]
pub type PauseenR = crate::BitReader<Pauseen>;
impl PauseenR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Pauseen {
        match self.bits {
            false => Pauseen::Disabled,
            true => Pauseen::Enabled,
        }
    }
    #[doc = "Pause operation disabled"]
    #[inline(always)]
    pub fn is_disabled(&self) -> bool {
        *self == Pauseen::Disabled
    }
    #[doc = "Pause operation enabled"]
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        *self == Pauseen::Enabled
    }
}
#[doc = "Field `PAUSEEN` writer - PAUSE Option Enable"]
pub type PauseenW<'a, REG> = crate::BitWriter<'a, REG, Pauseen>;
impl<'a, REG> PauseenW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "Pause operation disabled"]
    #[inline(always)]
    pub fn disabled(self) -> &'a mut crate::W<REG> {
        self.variant(Pauseen::Disabled)
    }
    #[doc = "Pause operation enabled"]
    #[inline(always)]
    pub fn enabled(self) -> &'a mut crate::W<REG> {
        self.variant(Pauseen::Enabled)
    }
}
impl R {
    #[doc = "Bits 0:8 - Pause Delay"]
    #[inline(always)]
    pub fn pausedly(&self) -> PausedlyR {
        PausedlyR::new((self.bits & 0x01ff) as u16)
    }
    #[doc = "Bit 31 - PAUSE Option Enable"]
    #[inline(always)]
    pub fn pauseen(&self) -> PauseenR {
        PauseenR::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:8 - Pause Delay"]
    #[inline(always)]
    pub fn pausedly(&mut self) -> PausedlyW<PauseSpec> {
        PausedlyW::new(self, 0)
    }
    #[doc = "Bit 31 - PAUSE Option Enable"]
    #[inline(always)]
    pub fn pauseen(&mut self) -> PauseenW<PauseSpec> {
        PauseenW::new(self, 31)
    }
}
#[doc = "Pause Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pause::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pause::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PauseSpec;
impl crate::RegisterSpec for PauseSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pause::R`](R) reader structure"]
impl crate::Readable for PauseSpec {}
#[doc = "`write(|w| ..)` method takes [`pause::W`](W) writer structure"]
impl crate::Writable for PauseSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PAUSE to value 0"]
impl crate::Resettable for PauseSpec {}

#[doc = "Register `ACTIVE_VDELAY` reader"]
pub type R = crate::R<ActiveVdelaySpec>;
#[doc = "Register `ACTIVE_VDELAY` writer"]
pub type W = crate::W<ActiveVdelaySpec>;
#[doc = "Field `ACTIVE_VDELAY` reader - Active Voltage Delay"]
pub type ActiveVdelayR = crate::FieldReader<u16>;
#[doc = "Field `ACTIVE_VDELAY` writer - Active Voltage Delay"]
pub type ActiveVdelayW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Active Voltage Delay"]
    #[inline(always)]
    pub fn active_vdelay(&self) -> ActiveVdelayR {
        ActiveVdelayR::new((self.bits & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Active Voltage Delay"]
    #[inline(always)]
    pub fn active_vdelay(&mut self) -> ActiveVdelayW<ActiveVdelaySpec> {
        ActiveVdelayW::new(self, 0)
    }
}
#[doc = "Active Voltage Trim Delay\n\nYou can [`read`](crate::Reg::read) this register and get [`active_vdelay::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`active_vdelay::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ActiveVdelaySpec;
impl crate::RegisterSpec for ActiveVdelaySpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`active_vdelay::R`](R) reader structure"]
impl crate::Readable for ActiveVdelaySpec {}
#[doc = "`write(|w| ..)` method takes [`active_vdelay::W`](W) writer structure"]
impl crate::Writable for ActiveVdelaySpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets ACTIVE_VDELAY to value 0xc8"]
impl crate::Resettable for ActiveVdelaySpec {
    const RESET_VALUE: u32 = 0xc8;
}

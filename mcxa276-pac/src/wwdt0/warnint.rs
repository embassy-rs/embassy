#[doc = "Register `WARNINT` reader"]
pub type R = crate::R<WarnintSpec>;
#[doc = "Register `WARNINT` writer"]
pub type W = crate::W<WarnintSpec>;
#[doc = "Field `WARNINT` reader - Watchdog Warning Interrupt Compare Value"]
pub type WarnintR = crate::FieldReader<u16>;
#[doc = "Field `WARNINT` writer - Watchdog Warning Interrupt Compare Value"]
pub type WarnintW<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
impl R {
    #[doc = "Bits 0:9 - Watchdog Warning Interrupt Compare Value"]
    #[inline(always)]
    pub fn warnint(&self) -> WarnintR {
        WarnintR::new((self.bits & 0x03ff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:9 - Watchdog Warning Interrupt Compare Value"]
    #[inline(always)]
    pub fn warnint(&mut self) -> WarnintW<WarnintSpec> {
        WarnintW::new(self, 0)
    }
}
#[doc = "Warning Interrupt Compare Value\n\nYou can [`read`](crate::Reg::read) this register and get [`warnint::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`warnint::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct WarnintSpec;
impl crate::RegisterSpec for WarnintSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`warnint::R`](R) reader structure"]
impl crate::Readable for WarnintSpec {}
#[doc = "`write(|w| ..)` method takes [`warnint::W`](W) writer structure"]
impl crate::Writable for WarnintSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets WARNINT to value 0"]
impl crate::Resettable for WarnintSpec {}

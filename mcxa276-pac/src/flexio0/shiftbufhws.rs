#[doc = "Register `SHIFTBUFHWS[%s]` reader"]
pub type R = crate::R<ShiftbufhwsSpec>;
#[doc = "Register `SHIFTBUFHWS[%s]` writer"]
pub type W = crate::W<ShiftbufhwsSpec>;
#[doc = "Field `SHIFTBUFHWS` reader - Shift Buffer"]
pub type ShiftbufhwsR = crate::FieldReader<u32>;
#[doc = "Field `SHIFTBUFHWS` writer - Shift Buffer"]
pub type ShiftbufhwsW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufhws(&self) -> ShiftbufhwsR {
        ShiftbufhwsR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufhws(&mut self) -> ShiftbufhwsW<ShiftbufhwsSpec> {
        ShiftbufhwsW::new(self, 0)
    }
}
#[doc = "Shifter Buffer Halfword Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufhws::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufhws::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShiftbufhwsSpec;
impl crate::RegisterSpec for ShiftbufhwsSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shiftbufhws::R`](R) reader structure"]
impl crate::Readable for ShiftbufhwsSpec {}
#[doc = "`write(|w| ..)` method takes [`shiftbufhws::W`](W) writer structure"]
impl crate::Writable for ShiftbufhwsSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SHIFTBUFHWS[%s] to value 0"]
impl crate::Resettable for ShiftbufhwsSpec {}

#[doc = "Register `SHIFTBUFHBS[%s]` reader"]
pub type R = crate::R<ShiftbufhbsSpec>;
#[doc = "Register `SHIFTBUFHBS[%s]` writer"]
pub type W = crate::W<ShiftbufhbsSpec>;
#[doc = "Field `SHIFTBUFHBS` reader - Shift Buffer"]
pub type ShiftbufhbsR = crate::FieldReader<u32>;
#[doc = "Field `SHIFTBUFHBS` writer - Shift Buffer"]
pub type ShiftbufhbsW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufhbs(&self) -> ShiftbufhbsR {
        ShiftbufhbsR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufhbs(&mut self) -> ShiftbufhbsW<ShiftbufhbsSpec> {
        ShiftbufhbsW::new(self, 0)
    }
}
#[doc = "Shifter Buffer Halfword Byte Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufhbs::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufhbs::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShiftbufhbsSpec;
impl crate::RegisterSpec for ShiftbufhbsSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shiftbufhbs::R`](R) reader structure"]
impl crate::Readable for ShiftbufhbsSpec {}
#[doc = "`write(|w| ..)` method takes [`shiftbufhbs::W`](W) writer structure"]
impl crate::Writable for ShiftbufhbsSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SHIFTBUFHBS[%s] to value 0"]
impl crate::Resettable for ShiftbufhbsSpec {}

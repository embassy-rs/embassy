#[doc = "Register `SHIFTBUFNBS[%s]` reader"]
pub type R = crate::R<ShiftbufnbsSpec>;
#[doc = "Register `SHIFTBUFNBS[%s]` writer"]
pub type W = crate::W<ShiftbufnbsSpec>;
#[doc = "Field `SHIFTBUFNBS` reader - Shift Buffer"]
pub type ShiftbufnbsR = crate::FieldReader<u32>;
#[doc = "Field `SHIFTBUFNBS` writer - Shift Buffer"]
pub type ShiftbufnbsW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufnbs(&self) -> ShiftbufnbsR {
        ShiftbufnbsR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufnbs(&mut self) -> ShiftbufnbsW<ShiftbufnbsSpec> {
        ShiftbufnbsW::new(self, 0)
    }
}
#[doc = "Shifter Buffer Nibble Byte Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufnbs::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufnbs::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShiftbufnbsSpec;
impl crate::RegisterSpec for ShiftbufnbsSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shiftbufnbs::R`](R) reader structure"]
impl crate::Readable for ShiftbufnbsSpec {}
#[doc = "`write(|w| ..)` method takes [`shiftbufnbs::W`](W) writer structure"]
impl crate::Writable for ShiftbufnbsSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SHIFTBUFNBS[%s] to value 0"]
impl crate::Resettable for ShiftbufnbsSpec {}

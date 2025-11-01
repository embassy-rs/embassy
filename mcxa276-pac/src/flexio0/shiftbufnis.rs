#[doc = "Register `SHIFTBUFNIS[%s]` reader"]
pub type R = crate::R<ShiftbufnisSpec>;
#[doc = "Register `SHIFTBUFNIS[%s]` writer"]
pub type W = crate::W<ShiftbufnisSpec>;
#[doc = "Field `SHIFTBUFNIS` reader - Shift Buffer"]
pub type ShiftbufnisR = crate::FieldReader<u32>;
#[doc = "Field `SHIFTBUFNIS` writer - Shift Buffer"]
pub type ShiftbufnisW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufnis(&self) -> ShiftbufnisR {
        ShiftbufnisR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufnis(&mut self) -> ShiftbufnisW<ShiftbufnisSpec> {
        ShiftbufnisW::new(self, 0)
    }
}
#[doc = "Shifter Buffer Nibble Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufnis::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufnis::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShiftbufnisSpec;
impl crate::RegisterSpec for ShiftbufnisSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shiftbufnis::R`](R) reader structure"]
impl crate::Readable for ShiftbufnisSpec {}
#[doc = "`write(|w| ..)` method takes [`shiftbufnis::W`](W) writer structure"]
impl crate::Writable for ShiftbufnisSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SHIFTBUFNIS[%s] to value 0"]
impl crate::Resettable for ShiftbufnisSpec {}

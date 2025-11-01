#[doc = "Register `SHIFTBUFBYS[%s]` reader"]
pub type R = crate::R<ShiftbufbysSpec>;
#[doc = "Register `SHIFTBUFBYS[%s]` writer"]
pub type W = crate::W<ShiftbufbysSpec>;
#[doc = "Field `SHIFTBUFBYS` reader - Shift Buffer"]
pub type ShiftbufbysR = crate::FieldReader<u32>;
#[doc = "Field `SHIFTBUFBYS` writer - Shift Buffer"]
pub type ShiftbufbysW<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufbys(&self) -> ShiftbufbysR {
        ShiftbufbysR::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Shift Buffer"]
    #[inline(always)]
    pub fn shiftbufbys(&mut self) -> ShiftbufbysW<ShiftbufbysSpec> {
        ShiftbufbysW::new(self, 0)
    }
}
#[doc = "Shifter Buffer Byte Swapped\n\nYou can [`read`](crate::Reg::read) this register and get [`shiftbufbys::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`shiftbufbys::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct ShiftbufbysSpec;
impl crate::RegisterSpec for ShiftbufbysSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`shiftbufbys::R`](R) reader structure"]
impl crate::Readable for ShiftbufbysSpec {}
#[doc = "`write(|w| ..)` method takes [`shiftbufbys::W`](W) writer structure"]
impl crate::Writable for ShiftbufbysSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SHIFTBUFBYS[%s] to value 0"]
impl crate::Resettable for ShiftbufbysSpec {}

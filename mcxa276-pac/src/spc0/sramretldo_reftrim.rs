#[doc = "Register `SRAMRETLDO_REFTRIM` reader"]
pub type R = crate::R<SramretldoReftrimSpec>;
#[doc = "Register `SRAMRETLDO_REFTRIM` writer"]
pub type W = crate::W<SramretldoReftrimSpec>;
#[doc = "Field `REFTRIM` reader - Reference Trim. Voltage range is around 0.48V - 0.85V. Trim step is 12 mV."]
pub type ReftrimR = crate::FieldReader;
#[doc = "Field `REFTRIM` writer - Reference Trim. Voltage range is around 0.48V - 0.85V. Trim step is 12 mV."]
pub type ReftrimW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
impl R {
    #[doc = "Bits 0:4 - Reference Trim. Voltage range is around 0.48V - 0.85V. Trim step is 12 mV."]
    #[inline(always)]
    pub fn reftrim(&self) -> ReftrimR {
        ReftrimR::new((self.bits & 0x1f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:4 - Reference Trim. Voltage range is around 0.48V - 0.85V. Trim step is 12 mV."]
    #[inline(always)]
    pub fn reftrim(&mut self) -> ReftrimW<SramretldoReftrimSpec> {
        ReftrimW::new(self, 0)
    }
}
#[doc = "SRAM Retention Reference Trim\n\nYou can [`read`](crate::Reg::read) this register and get [`sramretldo_reftrim::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sramretldo_reftrim::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SramretldoReftrimSpec;
impl crate::RegisterSpec for SramretldoReftrimSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sramretldo_reftrim::R`](R) reader structure"]
impl crate::Readable for SramretldoReftrimSpec {}
#[doc = "`write(|w| ..)` method takes [`sramretldo_reftrim::W`](W) writer structure"]
impl crate::Writable for SramretldoReftrimSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SRAMRETLDO_REFTRIM to value 0x17"]
impl crate::Resettable for SramretldoReftrimSpec {
    const RESET_VALUE: u32 = 0x17;
}

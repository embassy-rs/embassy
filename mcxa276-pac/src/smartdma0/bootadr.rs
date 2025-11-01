#[doc = "Register `BOOTADR` reader"]
pub type R = crate::R<BootadrSpec>;
#[doc = "Register `BOOTADR` writer"]
pub type W = crate::W<BootadrSpec>;
#[doc = "Field `ADDR` reader - 32-bit boot address, the boot address should be 4-byte aligned."]
pub type AddrR = crate::FieldReader<u32>;
#[doc = "Field `ADDR` writer - 32-bit boot address, the boot address should be 4-byte aligned."]
pub type AddrW<'a, REG> = crate::FieldWriter<'a, REG, 30, u32>;
impl R {
    #[doc = "Bits 2:31 - 32-bit boot address, the boot address should be 4-byte aligned."]
    #[inline(always)]
    pub fn addr(&self) -> AddrR {
        AddrR::new((self.bits >> 2) & 0x3fff_ffff)
    }
}
impl W {
    #[doc = "Bits 2:31 - 32-bit boot address, the boot address should be 4-byte aligned."]
    #[inline(always)]
    pub fn addr(&mut self) -> AddrW<BootadrSpec> {
        AddrW::new(self, 2)
    }
}
#[doc = "Boot Address\n\nYou can [`read`](crate::Reg::read) this register and get [`bootadr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`bootadr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct BootadrSpec;
impl crate::RegisterSpec for BootadrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`bootadr::R`](R) reader structure"]
impl crate::Readable for BootadrSpec {}
#[doc = "`write(|w| ..)` method takes [`bootadr::W`](W) writer structure"]
impl crate::Writable for BootadrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets BOOTADR to value 0"]
impl crate::Resettable for BootadrSpec {}

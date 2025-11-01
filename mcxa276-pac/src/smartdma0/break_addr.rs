#[doc = "Register `BREAK_ADDR` reader"]
pub type R = crate::R<BreakAddrSpec>;
#[doc = "Register `BREAK_ADDR` writer"]
pub type W = crate::W<BreakAddrSpec>;
#[doc = "Field `ADDR` reader - 32-bit address to swap to EZHB_BREAK_VECT location"]
pub type AddrR = crate::FieldReader<u32>;
#[doc = "Field `ADDR` writer - 32-bit address to swap to EZHB_BREAK_VECT location"]
pub type AddrW<'a, REG> = crate::FieldWriter<'a, REG, 30, u32>;
impl R {
    #[doc = "Bits 2:31 - 32-bit address to swap to EZHB_BREAK_VECT location"]
    #[inline(always)]
    pub fn addr(&self) -> AddrR {
        AddrR::new((self.bits >> 2) & 0x3fff_ffff)
    }
}
impl W {
    #[doc = "Bits 2:31 - 32-bit address to swap to EZHB_BREAK_VECT location"]
    #[inline(always)]
    pub fn addr(&mut self) -> AddrW<BreakAddrSpec> {
        AddrW::new(self, 2)
    }
}
#[doc = "Breakpoint Address\n\nYou can [`read`](crate::Reg::read) this register and get [`break_addr::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`break_addr::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct BreakAddrSpec;
impl crate::RegisterSpec for BreakAddrSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`break_addr::R`](R) reader structure"]
impl crate::Readable for BreakAddrSpec {}
#[doc = "`write(|w| ..)` method takes [`break_addr::W`](W) writer structure"]
impl crate::Writable for BreakAddrSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets BREAK_ADDR to value 0"]
impl crate::Resettable for BreakAddrSpec {}

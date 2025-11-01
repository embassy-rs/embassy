#[doc = "Register `SDCTL` reader"]
pub type R = crate::R<SdctlSpec>;
#[doc = "Register `SDCTL` writer"]
pub type W = crate::W<SdctlSpec>;
#[doc = "Field `SAMP_SIZE` reader - Sample Size"]
pub type SampSizeR = crate::FieldReader<u16>;
#[doc = "Field `SAMP_SIZE` writer - Sample Size"]
pub type SampSizeW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
#[doc = "Field `ENT_DLY` reader - Entropy Delay"]
pub type EntDlyR = crate::FieldReader<u16>;
#[doc = "Field `ENT_DLY` writer - Entropy Delay"]
pub type EntDlyW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
impl R {
    #[doc = "Bits 0:15 - Sample Size"]
    #[inline(always)]
    pub fn samp_size(&self) -> SampSizeR {
        SampSizeR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - Entropy Delay"]
    #[inline(always)]
    pub fn ent_dly(&self) -> EntDlyR {
        EntDlyR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Sample Size"]
    #[inline(always)]
    pub fn samp_size(&mut self) -> SampSizeW<SdctlSpec> {
        SampSizeW::new(self, 0)
    }
    #[doc = "Bits 16:31 - Entropy Delay"]
    #[inline(always)]
    pub fn ent_dly(&mut self) -> EntDlyW<SdctlSpec> {
        EntDlyW::new(self, 16)
    }
}
#[doc = "Seed Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sdctl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sdctl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SdctlSpec;
impl crate::RegisterSpec for SdctlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sdctl::R`](R) reader structure"]
impl crate::Readable for SdctlSpec {}
#[doc = "`write(|w| ..)` method takes [`sdctl::W`](W) writer structure"]
impl crate::Writable for SdctlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets SDCTL to value 0x0c80_0200"]
impl crate::Resettable for SdctlSpec {
    const RESET_VALUE: u32 = 0x0c80_0200;
}

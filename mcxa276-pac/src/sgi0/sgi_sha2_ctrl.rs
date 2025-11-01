#[doc = "Register `sgi_sha2_ctrl` reader"]
pub type R = crate::R<SgiSha2CtrlSpec>;
#[doc = "Register `sgi_sha2_ctrl` writer"]
pub type W = crate::W<SgiSha2CtrlSpec>;
#[doc = "Field `sha2_en` reader - SHA enable"]
pub type Sha2EnR = crate::BitReader;
#[doc = "Field `sha2_en` writer - SHA enable"]
pub type Sha2EnW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `sha2_mode` reader - SHA mode normal or automatic"]
pub type Sha2ModeR = crate::BitReader;
#[doc = "Field `sha2_mode` writer - SHA mode normal or automatic"]
pub type Sha2ModeW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `sha2_size` reader - SHA size 0=224;1=256;2=384;3=512"]
pub type Sha2SizeR = crate::FieldReader;
#[doc = "Field `sha2_size` writer - SHA size 0=224;1=256;2=384;3=512"]
pub type Sha2SizeW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `sha2_low_lim` reader - SHA FIFO low limit"]
pub type Sha2LowLimR = crate::FieldReader;
#[doc = "Field `sha2_low_lim` writer - SHA FIFO low limit"]
pub type Sha2LowLimW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `sha2_high_lim` reader - SHA FIFO high limit"]
pub type Sha2HighLimR = crate::FieldReader;
#[doc = "Field `sha2_high_lim` writer - SHA FIFO high limit"]
pub type Sha2HighLimW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `sha2_count_en` reader - SHA Calculation counter enable"]
pub type Sha2CountEnR = crate::BitReader;
#[doc = "Field `sha2_count_en` writer - SHA Calculation counter enable"]
pub type Sha2CountEnW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `hash_reload` reader - SHA HASH reload"]
pub type HashReloadR = crate::BitReader;
#[doc = "Field `hash_reload` writer - SHA HASH reload"]
pub type HashReloadW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `sha2_stop` writer - STOP SHA AUTO mode"]
pub type Sha2StopW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `no_auto_init` reader - SHA no automatic HASH initialisation"]
pub type NoAutoInitR = crate::BitReader;
#[doc = "Field `no_auto_init` writer - SHA no automatic HASH initialisation"]
pub type NoAutoInitW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `sha2ctl_rsvd` reader - reserved"]
pub type Sha2ctlRsvdR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bit 0 - SHA enable"]
    #[inline(always)]
    pub fn sha2_en(&self) -> Sha2EnR {
        Sha2EnR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - SHA mode normal or automatic"]
    #[inline(always)]
    pub fn sha2_mode(&self) -> Sha2ModeR {
        Sha2ModeR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 2:3 - SHA size 0=224;1=256;2=384;3=512"]
    #[inline(always)]
    pub fn sha2_size(&self) -> Sha2SizeR {
        Sha2SizeR::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:7 - SHA FIFO low limit"]
    #[inline(always)]
    pub fn sha2_low_lim(&self) -> Sha2LowLimR {
        Sha2LowLimR::new(((self.bits >> 4) & 0x0f) as u8)
    }
    #[doc = "Bits 8:11 - SHA FIFO high limit"]
    #[inline(always)]
    pub fn sha2_high_lim(&self) -> Sha2HighLimR {
        Sha2HighLimR::new(((self.bits >> 8) & 0x0f) as u8)
    }
    #[doc = "Bit 12 - SHA Calculation counter enable"]
    #[inline(always)]
    pub fn sha2_count_en(&self) -> Sha2CountEnR {
        Sha2CountEnR::new(((self.bits >> 12) & 1) != 0)
    }
    #[doc = "Bit 13 - SHA HASH reload"]
    #[inline(always)]
    pub fn hash_reload(&self) -> HashReloadR {
        HashReloadR::new(((self.bits >> 13) & 1) != 0)
    }
    #[doc = "Bit 15 - SHA no automatic HASH initialisation"]
    #[inline(always)]
    pub fn no_auto_init(&self) -> NoAutoInitR {
        NoAutoInitR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bits 16:31 - reserved"]
    #[inline(always)]
    pub fn sha2ctl_rsvd(&self) -> Sha2ctlRsvdR {
        Sha2ctlRsvdR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bit 0 - SHA enable"]
    #[inline(always)]
    pub fn sha2_en(&mut self) -> Sha2EnW<SgiSha2CtrlSpec> {
        Sha2EnW::new(self, 0)
    }
    #[doc = "Bit 1 - SHA mode normal or automatic"]
    #[inline(always)]
    pub fn sha2_mode(&mut self) -> Sha2ModeW<SgiSha2CtrlSpec> {
        Sha2ModeW::new(self, 1)
    }
    #[doc = "Bits 2:3 - SHA size 0=224;1=256;2=384;3=512"]
    #[inline(always)]
    pub fn sha2_size(&mut self) -> Sha2SizeW<SgiSha2CtrlSpec> {
        Sha2SizeW::new(self, 2)
    }
    #[doc = "Bits 4:7 - SHA FIFO low limit"]
    #[inline(always)]
    pub fn sha2_low_lim(&mut self) -> Sha2LowLimW<SgiSha2CtrlSpec> {
        Sha2LowLimW::new(self, 4)
    }
    #[doc = "Bits 8:11 - SHA FIFO high limit"]
    #[inline(always)]
    pub fn sha2_high_lim(&mut self) -> Sha2HighLimW<SgiSha2CtrlSpec> {
        Sha2HighLimW::new(self, 8)
    }
    #[doc = "Bit 12 - SHA Calculation counter enable"]
    #[inline(always)]
    pub fn sha2_count_en(&mut self) -> Sha2CountEnW<SgiSha2CtrlSpec> {
        Sha2CountEnW::new(self, 12)
    }
    #[doc = "Bit 13 - SHA HASH reload"]
    #[inline(always)]
    pub fn hash_reload(&mut self) -> HashReloadW<SgiSha2CtrlSpec> {
        HashReloadW::new(self, 13)
    }
    #[doc = "Bit 14 - STOP SHA AUTO mode"]
    #[inline(always)]
    pub fn sha2_stop(&mut self) -> Sha2StopW<SgiSha2CtrlSpec> {
        Sha2StopW::new(self, 14)
    }
    #[doc = "Bit 15 - SHA no automatic HASH initialisation"]
    #[inline(always)]
    pub fn no_auto_init(&mut self) -> NoAutoInitW<SgiSha2CtrlSpec> {
        NoAutoInitW::new(self, 15)
    }
}
#[doc = "SHA Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_sha2_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_sha2_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiSha2CtrlSpec;
impl crate::RegisterSpec for SgiSha2CtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_sha2_ctrl::R`](R) reader structure"]
impl crate::Readable for SgiSha2CtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_sha2_ctrl::W`](W) writer structure"]
impl crate::Writable for SgiSha2CtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_sha2_ctrl to value 0x0f00"]
impl crate::Resettable for SgiSha2CtrlSpec {
    const RESET_VALUE: u32 = 0x0f00;
}

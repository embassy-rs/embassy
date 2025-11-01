#[doc = "Register `sgi_ctrl2` reader"]
pub type R = crate::R<SgiCtrl2Spec>;
#[doc = "Register `sgi_ctrl2` writer"]
pub type W = crate::W<SgiCtrl2Spec>;
#[doc = "Field `flush` writer - Start Full SGI Flush"]
pub type FlushW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `key_flush` writer - Start KEY register-bank Flush"]
pub type KeyFlushW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `datin_flush` writer - Start DATIN register-bank Flush"]
pub type DatinFlushW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `incr` reader - Increment(Triggered by SFR write)"]
pub type IncrR = crate::BitReader;
#[doc = "Field `incr` writer - Increment(Triggered by SFR write)"]
pub type IncrW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `xorwr` reader - Write-XOR control"]
pub type XorwrR = crate::BitReader;
#[doc = "Field `xorwr` writer - Write-XOR control"]
pub type XorwrW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `flushwr` reader - Flush Write control"]
pub type FlushwrR = crate::BitReader;
#[doc = "Field `flushwr` writer - Flush Write control"]
pub type FlushwrW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `incr_cin` reader - Increment Carry-In control"]
pub type IncrCinR = crate::BitReader;
#[doc = "Field `incr_cin` writer - Increment Carry-In control"]
pub type IncrCinW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ctrl2_rsvd3` reader - reserved"]
pub type Ctrl2Rsvd3R = crate::BitReader;
#[doc = "Field `smasken` reader - SFRMASK Enable"]
pub type SmaskenR = crate::BitReader;
#[doc = "Field `smasken` writer - SFRMASK Enable"]
pub type SmaskenW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `smaskstep` reader - SFRSEED increment control"]
pub type SmaskstepR = crate::BitReader;
#[doc = "Field `smaskstep` writer - SFRSEED increment control"]
pub type SmaskstepW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `smasksw` reader - SFRMASK MASK control"]
pub type SmaskswR = crate::BitReader;
#[doc = "Field `smasksw` writer - SFRMASK MASK control"]
pub type SmaskswW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ctrl2_rsvd2` reader - reserved"]
pub type Ctrl2Rsvd2R = crate::BitReader;
#[doc = "Field `movem` reader - 4-bit optional input for MOVEM feature"]
pub type MovemR = crate::FieldReader;
#[doc = "Field `movem` writer - 4-bit optional input for MOVEM feature"]
pub type MovemW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `keyres` reader - Selects key registers to be updated when rkey=1"]
pub type KeyresR = crate::FieldReader;
#[doc = "Field `keyres` writer - Selects key registers to be updated when rkey=1"]
pub type KeyresW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Field `rkey` reader - Crypto result location"]
pub type RkeyR = crate::BitReader;
#[doc = "Field `rkey` writer - Crypto result location"]
pub type RkeyW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `bytes_order` reader - Byte order of regbank read/write data"]
pub type BytesOrderR = crate::BitReader;
#[doc = "Field `bytes_order` writer - Byte order of regbank read/write data"]
pub type BytesOrderW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `gcm_inxor` reader - GCM INXOR"]
pub type GcmInxorR = crate::BitReader;
#[doc = "Field `gcm_inxor` writer - GCM INXOR"]
pub type GcmInxorW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ctrl2_rsvd1` reader - reserved"]
pub type Ctrl2Rsvd1R = crate::FieldReader;
impl R {
    #[doc = "Bit 3 - Increment(Triggered by SFR write)"]
    #[inline(always)]
    pub fn incr(&self) -> IncrR {
        IncrR::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Write-XOR control"]
    #[inline(always)]
    pub fn xorwr(&self) -> XorwrR {
        XorwrR::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Flush Write control"]
    #[inline(always)]
    pub fn flushwr(&self) -> FlushwrR {
        FlushwrR::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Increment Carry-In control"]
    #[inline(always)]
    pub fn incr_cin(&self) -> IncrCinR {
        IncrCinR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - reserved"]
    #[inline(always)]
    pub fn ctrl2_rsvd3(&self) -> Ctrl2Rsvd3R {
        Ctrl2Rsvd3R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - SFRMASK Enable"]
    #[inline(always)]
    pub fn smasken(&self) -> SmaskenR {
        SmaskenR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bit 9 - SFRSEED increment control"]
    #[inline(always)]
    pub fn smaskstep(&self) -> SmaskstepR {
        SmaskstepR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bit 10 - SFRMASK MASK control"]
    #[inline(always)]
    pub fn smasksw(&self) -> SmaskswR {
        SmaskswR::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bit 11 - reserved"]
    #[inline(always)]
    pub fn ctrl2_rsvd2(&self) -> Ctrl2Rsvd2R {
        Ctrl2Rsvd2R::new(((self.bits >> 11) & 1) != 0)
    }
    #[doc = "Bits 12:15 - 4-bit optional input for MOVEM feature"]
    #[inline(always)]
    pub fn movem(&self) -> MovemR {
        MovemR::new(((self.bits >> 12) & 0x0f) as u8)
    }
    #[doc = "Bits 16:20 - Selects key registers to be updated when rkey=1"]
    #[inline(always)]
    pub fn keyres(&self) -> KeyresR {
        KeyresR::new(((self.bits >> 16) & 0x1f) as u8)
    }
    #[doc = "Bit 21 - Crypto result location"]
    #[inline(always)]
    pub fn rkey(&self) -> RkeyR {
        RkeyR::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Byte order of regbank read/write data"]
    #[inline(always)]
    pub fn bytes_order(&self) -> BytesOrderR {
        BytesOrderR::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - GCM INXOR"]
    #[inline(always)]
    pub fn gcm_inxor(&self) -> GcmInxorR {
        GcmInxorR::new(((self.bits >> 23) & 1) != 0)
    }
    #[doc = "Bits 24:31 - reserved"]
    #[inline(always)]
    pub fn ctrl2_rsvd1(&self) -> Ctrl2Rsvd1R {
        Ctrl2Rsvd1R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Start Full SGI Flush"]
    #[inline(always)]
    pub fn flush(&mut self) -> FlushW<SgiCtrl2Spec> {
        FlushW::new(self, 0)
    }
    #[doc = "Bit 1 - Start KEY register-bank Flush"]
    #[inline(always)]
    pub fn key_flush(&mut self) -> KeyFlushW<SgiCtrl2Spec> {
        KeyFlushW::new(self, 1)
    }
    #[doc = "Bit 2 - Start DATIN register-bank Flush"]
    #[inline(always)]
    pub fn datin_flush(&mut self) -> DatinFlushW<SgiCtrl2Spec> {
        DatinFlushW::new(self, 2)
    }
    #[doc = "Bit 3 - Increment(Triggered by SFR write)"]
    #[inline(always)]
    pub fn incr(&mut self) -> IncrW<SgiCtrl2Spec> {
        IncrW::new(self, 3)
    }
    #[doc = "Bit 4 - Write-XOR control"]
    #[inline(always)]
    pub fn xorwr(&mut self) -> XorwrW<SgiCtrl2Spec> {
        XorwrW::new(self, 4)
    }
    #[doc = "Bit 5 - Flush Write control"]
    #[inline(always)]
    pub fn flushwr(&mut self) -> FlushwrW<SgiCtrl2Spec> {
        FlushwrW::new(self, 5)
    }
    #[doc = "Bit 6 - Increment Carry-In control"]
    #[inline(always)]
    pub fn incr_cin(&mut self) -> IncrCinW<SgiCtrl2Spec> {
        IncrCinW::new(self, 6)
    }
    #[doc = "Bit 8 - SFRMASK Enable"]
    #[inline(always)]
    pub fn smasken(&mut self) -> SmaskenW<SgiCtrl2Spec> {
        SmaskenW::new(self, 8)
    }
    #[doc = "Bit 9 - SFRSEED increment control"]
    #[inline(always)]
    pub fn smaskstep(&mut self) -> SmaskstepW<SgiCtrl2Spec> {
        SmaskstepW::new(self, 9)
    }
    #[doc = "Bit 10 - SFRMASK MASK control"]
    #[inline(always)]
    pub fn smasksw(&mut self) -> SmaskswW<SgiCtrl2Spec> {
        SmaskswW::new(self, 10)
    }
    #[doc = "Bits 12:15 - 4-bit optional input for MOVEM feature"]
    #[inline(always)]
    pub fn movem(&mut self) -> MovemW<SgiCtrl2Spec> {
        MovemW::new(self, 12)
    }
    #[doc = "Bits 16:20 - Selects key registers to be updated when rkey=1"]
    #[inline(always)]
    pub fn keyres(&mut self) -> KeyresW<SgiCtrl2Spec> {
        KeyresW::new(self, 16)
    }
    #[doc = "Bit 21 - Crypto result location"]
    #[inline(always)]
    pub fn rkey(&mut self) -> RkeyW<SgiCtrl2Spec> {
        RkeyW::new(self, 21)
    }
    #[doc = "Bit 22 - Byte order of regbank read/write data"]
    #[inline(always)]
    pub fn bytes_order(&mut self) -> BytesOrderW<SgiCtrl2Spec> {
        BytesOrderW::new(self, 22)
    }
    #[doc = "Bit 23 - GCM INXOR"]
    #[inline(always)]
    pub fn gcm_inxor(&mut self) -> GcmInxorW<SgiCtrl2Spec> {
        GcmInxorW::new(self, 23)
    }
}
#[doc = "SGI Control register 2\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_ctrl2::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_ctrl2::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiCtrl2Spec;
impl crate::RegisterSpec for SgiCtrl2Spec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_ctrl2::R`](R) reader structure"]
impl crate::Readable for SgiCtrl2Spec {}
#[doc = "`write(|w| ..)` method takes [`sgi_ctrl2::W`](W) writer structure"]
impl crate::Writable for SgiCtrl2Spec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_ctrl2 to value 0"]
impl crate::Resettable for SgiCtrl2Spec {}

#[doc = "Register `sgi_ctrl` reader"]
pub type R = crate::R<SgiCtrlSpec>;
#[doc = "Register `sgi_ctrl` writer"]
pub type W = crate::W<SgiCtrlSpec>;
#[doc = "Field `start` writer - Start crypto operation"]
pub type StartW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `decrypt` reader - Sets Cipher direction(AES and DES)"]
pub type DecryptR = crate::BitReader;
#[doc = "Field `decrypt` writer - Sets Cipher direction(AES and DES)"]
pub type DecryptW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `aeskeysz` reader - Sets AES key size"]
pub type AeskeyszR = crate::FieldReader;
#[doc = "Field `aeskeysz` writer - Sets AES key size"]
pub type AeskeyszW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `crypto_op` reader - Sets 'Crypto Operation' type"]
pub type CryptoOpR = crate::FieldReader;
#[doc = "Field `crypto_op` writer - Sets 'Crypto Operation' type"]
pub type CryptoOpW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `insel` reader - 4'b0000 - DATIN\\[0\\]"]
pub type InselR = crate::FieldReader;
#[doc = "Field `insel` writer - 4'b0000 - DATIN\\[0\\]"]
pub type InselW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `outsel` reader - 3'b000 - DATOUT = 'Kernel Res'"]
pub type OutselR = crate::FieldReader;
#[doc = "Field `outsel` writer - 3'b000 - DATOUT = 'Kernel Res'"]
pub type OutselW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `datout_res` reader - Kernels data out options"]
pub type DatoutResR = crate::FieldReader;
#[doc = "Field `datout_res` writer - Kernels data out options"]
pub type DatoutResW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `aes_en` reader - AES Kernel Enable"]
pub type AesEnR = crate::BitReader;
#[doc = "Field `aes_en` writer - AES Kernel Enable"]
pub type AesEnW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `des_en` reader - DES Kernel Enable"]
pub type DesEnR = crate::BitReader;
#[doc = "Field `des_en` writer - DES Kernel Enable"]
pub type DesEnW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `gcm_en` reader - GFMUL Kernel Enable"]
pub type GcmEnR = crate::BitReader;
#[doc = "Field `gcm_en` writer - GFMUL Kernel Enable"]
pub type GcmEnW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `prng_en` reader - PRNG Enable(only if SGI has internal PRNG, Please"]
pub type PrngEnR = crate::BitReader;
#[doc = "Field `prng_en` writer - PRNG Enable(only if SGI has internal PRNG, Please"]
pub type PrngEnW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `inkeysel` reader - Input key selection"]
pub type InkeyselR = crate::FieldReader;
#[doc = "Field `inkeysel` writer - Input key selection"]
pub type InkeyselW<'a, REG> = crate::FieldWriter<'a, REG, 5>;
#[doc = "Field `tdeskey` reader - Triple-DES Key Configuration"]
pub type TdeskeyR = crate::BitReader;
#[doc = "Field `tdeskey` writer - Triple-DES Key Configuration"]
pub type TdeskeyW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `aes_no_kl` reader - AES No decryption key schedule"]
pub type AesNoKlR = crate::BitReader;
#[doc = "Field `aes_no_kl` writer - AES No decryption key schedule"]
pub type AesNoKlW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `aes_sel` reader - AES Dual Selection"]
pub type AesSelR = crate::BitReader;
#[doc = "Field `aes_sel` writer - AES Dual Selection"]
pub type AesSelW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `ctrl_rsvd` reader - reserved"]
pub type CtrlRsvdR = crate::FieldReader;
impl R {
    #[doc = "Bit 1 - Sets Cipher direction(AES and DES)"]
    #[inline(always)]
    pub fn decrypt(&self) -> DecryptR {
        DecryptR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 2:3 - Sets AES key size"]
    #[inline(always)]
    pub fn aeskeysz(&self) -> AeskeyszR {
        AeskeyszR::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:6 - Sets 'Crypto Operation' type"]
    #[inline(always)]
    pub fn crypto_op(&self) -> CryptoOpR {
        CryptoOpR::new(((self.bits >> 4) & 7) as u8)
    }
    #[doc = "Bits 7:10 - 4'b0000 - DATIN\\[0\\]"]
    #[inline(always)]
    pub fn insel(&self) -> InselR {
        InselR::new(((self.bits >> 7) & 0x0f) as u8)
    }
    #[doc = "Bits 11:13 - 3'b000 - DATOUT = 'Kernel Res'"]
    #[inline(always)]
    pub fn outsel(&self) -> OutselR {
        OutselR::new(((self.bits >> 11) & 7) as u8)
    }
    #[doc = "Bits 14:15 - Kernels data out options"]
    #[inline(always)]
    pub fn datout_res(&self) -> DatoutResR {
        DatoutResR::new(((self.bits >> 14) & 3) as u8)
    }
    #[doc = "Bit 16 - AES Kernel Enable"]
    #[inline(always)]
    pub fn aes_en(&self) -> AesEnR {
        AesEnR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - DES Kernel Enable"]
    #[inline(always)]
    pub fn des_en(&self) -> DesEnR {
        DesEnR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - GFMUL Kernel Enable"]
    #[inline(always)]
    pub fn gcm_en(&self) -> GcmEnR {
        GcmEnR::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - PRNG Enable(only if SGI has internal PRNG, Please"]
    #[inline(always)]
    pub fn prng_en(&self) -> PrngEnR {
        PrngEnR::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bits 20:24 - Input key selection"]
    #[inline(always)]
    pub fn inkeysel(&self) -> InkeyselR {
        InkeyselR::new(((self.bits >> 20) & 0x1f) as u8)
    }
    #[doc = "Bit 25 - Triple-DES Key Configuration"]
    #[inline(always)]
    pub fn tdeskey(&self) -> TdeskeyR {
        TdeskeyR::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bit 26 - AES No decryption key schedule"]
    #[inline(always)]
    pub fn aes_no_kl(&self) -> AesNoKlR {
        AesNoKlR::new(((self.bits >> 26) & 1) != 0)
    }
    #[doc = "Bit 27 - AES Dual Selection"]
    #[inline(always)]
    pub fn aes_sel(&self) -> AesSelR {
        AesSelR::new(((self.bits >> 27) & 1) != 0)
    }
    #[doc = "Bits 28:31 - reserved"]
    #[inline(always)]
    pub fn ctrl_rsvd(&self) -> CtrlRsvdR {
        CtrlRsvdR::new(((self.bits >> 28) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - Start crypto operation"]
    #[inline(always)]
    pub fn start(&mut self) -> StartW<SgiCtrlSpec> {
        StartW::new(self, 0)
    }
    #[doc = "Bit 1 - Sets Cipher direction(AES and DES)"]
    #[inline(always)]
    pub fn decrypt(&mut self) -> DecryptW<SgiCtrlSpec> {
        DecryptW::new(self, 1)
    }
    #[doc = "Bits 2:3 - Sets AES key size"]
    #[inline(always)]
    pub fn aeskeysz(&mut self) -> AeskeyszW<SgiCtrlSpec> {
        AeskeyszW::new(self, 2)
    }
    #[doc = "Bits 4:6 - Sets 'Crypto Operation' type"]
    #[inline(always)]
    pub fn crypto_op(&mut self) -> CryptoOpW<SgiCtrlSpec> {
        CryptoOpW::new(self, 4)
    }
    #[doc = "Bits 7:10 - 4'b0000 - DATIN\\[0\\]"]
    #[inline(always)]
    pub fn insel(&mut self) -> InselW<SgiCtrlSpec> {
        InselW::new(self, 7)
    }
    #[doc = "Bits 11:13 - 3'b000 - DATOUT = 'Kernel Res'"]
    #[inline(always)]
    pub fn outsel(&mut self) -> OutselW<SgiCtrlSpec> {
        OutselW::new(self, 11)
    }
    #[doc = "Bits 14:15 - Kernels data out options"]
    #[inline(always)]
    pub fn datout_res(&mut self) -> DatoutResW<SgiCtrlSpec> {
        DatoutResW::new(self, 14)
    }
    #[doc = "Bit 16 - AES Kernel Enable"]
    #[inline(always)]
    pub fn aes_en(&mut self) -> AesEnW<SgiCtrlSpec> {
        AesEnW::new(self, 16)
    }
    #[doc = "Bit 17 - DES Kernel Enable"]
    #[inline(always)]
    pub fn des_en(&mut self) -> DesEnW<SgiCtrlSpec> {
        DesEnW::new(self, 17)
    }
    #[doc = "Bit 18 - GFMUL Kernel Enable"]
    #[inline(always)]
    pub fn gcm_en(&mut self) -> GcmEnW<SgiCtrlSpec> {
        GcmEnW::new(self, 18)
    }
    #[doc = "Bit 19 - PRNG Enable(only if SGI has internal PRNG, Please"]
    #[inline(always)]
    pub fn prng_en(&mut self) -> PrngEnW<SgiCtrlSpec> {
        PrngEnW::new(self, 19)
    }
    #[doc = "Bits 20:24 - Input key selection"]
    #[inline(always)]
    pub fn inkeysel(&mut self) -> InkeyselW<SgiCtrlSpec> {
        InkeyselW::new(self, 20)
    }
    #[doc = "Bit 25 - Triple-DES Key Configuration"]
    #[inline(always)]
    pub fn tdeskey(&mut self) -> TdeskeyW<SgiCtrlSpec> {
        TdeskeyW::new(self, 25)
    }
    #[doc = "Bit 26 - AES No decryption key schedule"]
    #[inline(always)]
    pub fn aes_no_kl(&mut self) -> AesNoKlW<SgiCtrlSpec> {
        AesNoKlW::new(self, 26)
    }
    #[doc = "Bit 27 - AES Dual Selection"]
    #[inline(always)]
    pub fn aes_sel(&mut self) -> AesSelW<SgiCtrlSpec> {
        AesSelW::new(self, 27)
    }
}
#[doc = "SGI Control register\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiCtrlSpec;
impl crate::RegisterSpec for SgiCtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_ctrl::R`](R) reader structure"]
impl crate::Readable for SgiCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_ctrl::W`](W) writer structure"]
impl crate::Writable for SgiCtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_ctrl to value 0"]
impl crate::Resettable for SgiCtrlSpec {}

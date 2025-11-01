#[doc = "Register `sgi_auto_mode` reader"]
pub type R = crate::R<SgiAutoModeSpec>;
#[doc = "Register `sgi_auto_mode` writer"]
pub type W = crate::W<SgiAutoModeSpec>;
#[doc = "Field `auto_mode_en` reader - auto_start_en"]
pub type AutoModeEnR = crate::BitReader;
#[doc = "Field `auto_mode_en` writer - auto_start_en"]
pub type AutoModeEnW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `auto_mode_stop` writer - auto_mode_stop"]
pub type AutoModeStopW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `auto_mode_rsvd1` reader - reserved"]
pub type AutoModeRsvd1R = crate::FieldReader;
#[doc = "Field `incr_mode` reader - CTR increment mode"]
pub type IncrModeR = crate::FieldReader;
#[doc = "Field `incr_mode` writer - CTR increment mode"]
pub type IncrModeW<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `auto_mode_rsvd2` reader - reserved"]
pub type AutoModeRsvd2R = crate::FieldReader;
#[doc = "Auto mode of operation\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Cmd {
    #[doc = "0: 8'h00 - ECB mode"]
    Ecb = 0,
    #[doc = "1: 8'h01 - CTR mode"]
    Ctr = 1,
    #[doc = "2: 8'h02 - CBC mode"]
    Cbc = 2,
    #[doc = "3: 8'h03 - CBCMAC mode"]
    Cbcmac = 3,
    #[doc = "16: 8'h10 - Key Wrap/Unwrap(128 bit key data)"]
    Kw128 = 16,
    #[doc = "17: 8'h11 - Key Wrap/Unwrap(256 bit key data)"]
    Kw256 = 17,
}
impl From<Cmd> for u8 {
    #[inline(always)]
    fn from(variant: Cmd) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Cmd {
    type Ux = u8;
}
impl crate::IsEnum for Cmd {}
#[doc = "Field `cmd` reader - Auto mode of operation"]
pub type CmdR = crate::FieldReader<Cmd>;
impl CmdR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Cmd> {
        match self.bits {
            0 => Some(Cmd::Ecb),
            1 => Some(Cmd::Ctr),
            2 => Some(Cmd::Cbc),
            3 => Some(Cmd::Cbcmac),
            16 => Some(Cmd::Kw128),
            17 => Some(Cmd::Kw256),
            _ => None,
        }
    }
    #[doc = "8'h00 - ECB mode"]
    #[inline(always)]
    pub fn is_ecb(&self) -> bool {
        *self == Cmd::Ecb
    }
    #[doc = "8'h01 - CTR mode"]
    #[inline(always)]
    pub fn is_ctr(&self) -> bool {
        *self == Cmd::Ctr
    }
    #[doc = "8'h02 - CBC mode"]
    #[inline(always)]
    pub fn is_cbc(&self) -> bool {
        *self == Cmd::Cbc
    }
    #[doc = "8'h03 - CBCMAC mode"]
    #[inline(always)]
    pub fn is_cbcmac(&self) -> bool {
        *self == Cmd::Cbcmac
    }
    #[doc = "8'h10 - Key Wrap/Unwrap(128 bit key data)"]
    #[inline(always)]
    pub fn is_kw128(&self) -> bool {
        *self == Cmd::Kw128
    }
    #[doc = "8'h11 - Key Wrap/Unwrap(256 bit key data)"]
    #[inline(always)]
    pub fn is_kw256(&self) -> bool {
        *self == Cmd::Kw256
    }
}
#[doc = "Field `cmd` writer - Auto mode of operation"]
pub type CmdW<'a, REG> = crate::FieldWriter<'a, REG, 8, Cmd>;
impl<'a, REG> CmdW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "8'h00 - ECB mode"]
    #[inline(always)]
    pub fn ecb(self) -> &'a mut crate::W<REG> {
        self.variant(Cmd::Ecb)
    }
    #[doc = "8'h01 - CTR mode"]
    #[inline(always)]
    pub fn ctr(self) -> &'a mut crate::W<REG> {
        self.variant(Cmd::Ctr)
    }
    #[doc = "8'h02 - CBC mode"]
    #[inline(always)]
    pub fn cbc(self) -> &'a mut crate::W<REG> {
        self.variant(Cmd::Cbc)
    }
    #[doc = "8'h03 - CBCMAC mode"]
    #[inline(always)]
    pub fn cbcmac(self) -> &'a mut crate::W<REG> {
        self.variant(Cmd::Cbcmac)
    }
    #[doc = "8'h10 - Key Wrap/Unwrap(128 bit key data)"]
    #[inline(always)]
    pub fn kw128(self) -> &'a mut crate::W<REG> {
        self.variant(Cmd::Kw128)
    }
    #[doc = "8'h11 - Key Wrap/Unwrap(256 bit key data)"]
    #[inline(always)]
    pub fn kw256(self) -> &'a mut crate::W<REG> {
        self.variant(Cmd::Kw256)
    }
}
#[doc = "Field `auto_mode_rsvd3` reader - reserved"]
pub type AutoModeRsvd3R = crate::FieldReader<u16>;
impl R {
    #[doc = "Bit 0 - auto_start_en"]
    #[inline(always)]
    pub fn auto_mode_en(&self) -> AutoModeEnR {
        AutoModeEnR::new((self.bits & 1) != 0)
    }
    #[doc = "Bits 2:3 - reserved"]
    #[inline(always)]
    pub fn auto_mode_rsvd1(&self) -> AutoModeRsvd1R {
        AutoModeRsvd1R::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:5 - CTR increment mode"]
    #[inline(always)]
    pub fn incr_mode(&self) -> IncrModeR {
        IncrModeR::new(((self.bits >> 4) & 3) as u8)
    }
    #[doc = "Bits 6:7 - reserved"]
    #[inline(always)]
    pub fn auto_mode_rsvd2(&self) -> AutoModeRsvd2R {
        AutoModeRsvd2R::new(((self.bits >> 6) & 3) as u8)
    }
    #[doc = "Bits 8:15 - Auto mode of operation"]
    #[inline(always)]
    pub fn cmd(&self) -> CmdR {
        CmdR::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 16:31 - reserved"]
    #[inline(always)]
    pub fn auto_mode_rsvd3(&self) -> AutoModeRsvd3R {
        AutoModeRsvd3R::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bit 0 - auto_start_en"]
    #[inline(always)]
    pub fn auto_mode_en(&mut self) -> AutoModeEnW<SgiAutoModeSpec> {
        AutoModeEnW::new(self, 0)
    }
    #[doc = "Bit 1 - auto_mode_stop"]
    #[inline(always)]
    pub fn auto_mode_stop(&mut self) -> AutoModeStopW<SgiAutoModeSpec> {
        AutoModeStopW::new(self, 1)
    }
    #[doc = "Bits 4:5 - CTR increment mode"]
    #[inline(always)]
    pub fn incr_mode(&mut self) -> IncrModeW<SgiAutoModeSpec> {
        IncrModeW::new(self, 4)
    }
    #[doc = "Bits 8:15 - Auto mode of operation"]
    #[inline(always)]
    pub fn cmd(&mut self) -> CmdW<SgiAutoModeSpec> {
        CmdW::new(self, 8)
    }
}
#[doc = "SGI Auto Mode Control register\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_auto_mode::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_auto_mode::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiAutoModeSpec;
impl crate::RegisterSpec for SgiAutoModeSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_auto_mode::R`](R) reader structure"]
impl crate::Readable for SgiAutoModeSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_auto_mode::W`](W) writer structure"]
impl crate::Writable for SgiAutoModeSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_auto_mode to value 0"]
impl crate::Resettable for SgiAutoModeSpec {}

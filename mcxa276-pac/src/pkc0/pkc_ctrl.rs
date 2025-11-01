#[doc = "Register `PKC_CTRL` reader"]
pub type R = crate::R<PkcCtrlSpec>;
#[doc = "Register `PKC_CTRL` writer"]
pub type W = crate::W<PkcCtrlSpec>;
#[doc = "Field `RESET` reader - PKC reset control bit"]
pub type ResetR = crate::BitReader;
#[doc = "Field `RESET` writer - PKC reset control bit"]
pub type ResetW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `STOP` reader - Freeze PKC calculation"]
pub type StopR = crate::BitReader;
#[doc = "Field `STOP` writer - Freeze PKC calculation"]
pub type StopW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `GOD1` writer - Control bit to start direct operation using parameter set 1"]
pub type God1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `GOD2` writer - Control bit to start direct operation using parameter set 2"]
pub type God2W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `GOM1` writer - Control bit to start MC pattern using parameter set 1"]
pub type Gom1W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `GOM2` writer - Control bit to start MC pattern using parameter set 2"]
pub type Gom2W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `GOU` writer - Control bit to start pipe operation"]
pub type GouW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `GF2CONV` reader - Convert to GF2 calculation modes"]
pub type Gf2convR = crate::BitReader;
#[doc = "Field `GF2CONV` writer - Convert to GF2 calculation modes"]
pub type Gf2convW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CLRCACHE` writer - Clear universal pointer cache"]
pub type ClrcacheW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `CACHE_EN` reader - Enable universal pointer cache"]
pub type CacheEnR = crate::BitReader;
#[doc = "Field `CACHE_EN` writer - Enable universal pointer cache"]
pub type CacheEnW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Reduced multiplier mode\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Redmul {
    #[doc = "0: full size mode, 3 least significant bits of pointer and length are ignored, minimum supported length 0x0008"]
    Fullsz = 0,
    #[doc = "2: 64-bit mode, 3 least significant bits of pointer and length are ignored, minimum supported length 0x0008"]
    Value64bit = 2,
}
impl From<Redmul> for u8 {
    #[inline(always)]
    fn from(variant: Redmul) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Redmul {
    type Ux = u8;
}
impl crate::IsEnum for Redmul {}
#[doc = "Field `REDMUL` reader - Reduced multiplier mode"]
pub type RedmulR = crate::FieldReader<Redmul>;
impl RedmulR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Redmul> {
        match self.bits {
            0 => Some(Redmul::Fullsz),
            2 => Some(Redmul::Value64bit),
            _ => None,
        }
    }
    #[doc = "full size mode, 3 least significant bits of pointer and length are ignored, minimum supported length 0x0008"]
    #[inline(always)]
    pub fn is_fullsz(&self) -> bool {
        *self == Redmul::Fullsz
    }
    #[doc = "64-bit mode, 3 least significant bits of pointer and length are ignored, minimum supported length 0x0008"]
    #[inline(always)]
    pub fn is_value_64bit(&self) -> bool {
        *self == Redmul::Value64bit
    }
}
#[doc = "Field `REDMUL` writer - Reduced multiplier mode"]
pub type RedmulW<'a, REG> = crate::FieldWriter<'a, REG, 2, Redmul>;
impl<'a, REG> RedmulW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "full size mode, 3 least significant bits of pointer and length are ignored, minimum supported length 0x0008"]
    #[inline(always)]
    pub fn fullsz(self) -> &'a mut crate::W<REG> {
        self.variant(Redmul::Fullsz)
    }
    #[doc = "64-bit mode, 3 least significant bits of pointer and length are ignored, minimum supported length 0x0008"]
    #[inline(always)]
    pub fn value_64bit(self) -> &'a mut crate::W<REG> {
        self.variant(Redmul::Value64bit)
    }
}
impl R {
    #[doc = "Bit 0 - PKC reset control bit"]
    #[inline(always)]
    pub fn reset(&self) -> ResetR {
        ResetR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Freeze PKC calculation"]
    #[inline(always)]
    pub fn stop(&self) -> StopR {
        StopR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 7 - Convert to GF2 calculation modes"]
    #[inline(always)]
    pub fn gf2conv(&self) -> Gf2convR {
        Gf2convR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 9 - Enable universal pointer cache"]
    #[inline(always)]
    pub fn cache_en(&self) -> CacheEnR {
        CacheEnR::new(((self.bits >> 9) & 1) != 0)
    }
    #[doc = "Bits 10:11 - Reduced multiplier mode"]
    #[inline(always)]
    pub fn redmul(&self) -> RedmulR {
        RedmulR::new(((self.bits >> 10) & 3) as u8)
    }
}
impl W {
    #[doc = "Bit 0 - PKC reset control bit"]
    #[inline(always)]
    pub fn reset(&mut self) -> ResetW<PkcCtrlSpec> {
        ResetW::new(self, 0)
    }
    #[doc = "Bit 1 - Freeze PKC calculation"]
    #[inline(always)]
    pub fn stop(&mut self) -> StopW<PkcCtrlSpec> {
        StopW::new(self, 1)
    }
    #[doc = "Bit 2 - Control bit to start direct operation using parameter set 1"]
    #[inline(always)]
    pub fn god1(&mut self) -> God1W<PkcCtrlSpec> {
        God1W::new(self, 2)
    }
    #[doc = "Bit 3 - Control bit to start direct operation using parameter set 2"]
    #[inline(always)]
    pub fn god2(&mut self) -> God2W<PkcCtrlSpec> {
        God2W::new(self, 3)
    }
    #[doc = "Bit 4 - Control bit to start MC pattern using parameter set 1"]
    #[inline(always)]
    pub fn gom1(&mut self) -> Gom1W<PkcCtrlSpec> {
        Gom1W::new(self, 4)
    }
    #[doc = "Bit 5 - Control bit to start MC pattern using parameter set 2"]
    #[inline(always)]
    pub fn gom2(&mut self) -> Gom2W<PkcCtrlSpec> {
        Gom2W::new(self, 5)
    }
    #[doc = "Bit 6 - Control bit to start pipe operation"]
    #[inline(always)]
    pub fn gou(&mut self) -> GouW<PkcCtrlSpec> {
        GouW::new(self, 6)
    }
    #[doc = "Bit 7 - Convert to GF2 calculation modes"]
    #[inline(always)]
    pub fn gf2conv(&mut self) -> Gf2convW<PkcCtrlSpec> {
        Gf2convW::new(self, 7)
    }
    #[doc = "Bit 8 - Clear universal pointer cache"]
    #[inline(always)]
    pub fn clrcache(&mut self) -> ClrcacheW<PkcCtrlSpec> {
        ClrcacheW::new(self, 8)
    }
    #[doc = "Bit 9 - Enable universal pointer cache"]
    #[inline(always)]
    pub fn cache_en(&mut self) -> CacheEnW<PkcCtrlSpec> {
        CacheEnW::new(self, 9)
    }
    #[doc = "Bits 10:11 - Reduced multiplier mode"]
    #[inline(always)]
    pub fn redmul(&mut self) -> RedmulW<PkcCtrlSpec> {
        RedmulW::new(self, 10)
    }
}
#[doc = "Control Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkc_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`pkc_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkcCtrlSpec;
impl crate::RegisterSpec for PkcCtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkc_ctrl::R`](R) reader structure"]
impl crate::Readable for PkcCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`pkc_ctrl::W`](W) writer structure"]
impl crate::Writable for PkcCtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets PKC_CTRL to value 0x01"]
impl crate::Resettable for PkcCtrlSpec {
    const RESET_VALUE: u32 = 0x01;
}

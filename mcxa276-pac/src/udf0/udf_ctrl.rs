#[doc = "Register `udf_ctrl` reader"]
pub type R = crate::R<UdfCtrlSpec>;
#[doc = "Register `udf_ctrl` writer"]
pub type W = crate::W<UdfCtrlSpec>;
#[doc = "Field `salt` reader - Bits are internally XORed with i_custom"]
pub type SaltR = crate::FieldReader<u16>;
#[doc = "Field `salt` writer - Bits are internally XORed with i_custom"]
pub type SaltW<'a, REG> = crate::FieldWriter<'a, REG, 16, u16>;
#[doc = "Field `lock` reader - Lock access to UDF"]
pub type LockR = crate::FieldReader;
#[doc = "Field `lock` writer - Lock access to UDF"]
pub type LockW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `reserved21` reader - RFU"]
pub type Reserved21R = crate::FieldReader;
#[doc = "Field `reserved21` writer - RFU"]
pub type Reserved21W<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `udf_en` reader - Enable the UDF block"]
pub type UdfEnR = crate::FieldReader;
#[doc = "Field `udf_en` writer - Enable the UDF block"]
pub type UdfEnW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `reserved25` reader - RFU"]
pub type Reserved25R = crate::BitReader;
#[doc = "Field `reserved25` writer - RFU"]
pub type Reserved25W<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `reserved27` reader - RFU"]
pub type Reserved27R = crate::FieldReader;
#[doc = "Field `reserved27` writer - RFU"]
pub type Reserved27W<'a, REG> = crate::FieldWriter<'a, REG, 2>;
#[doc = "Field `flush` reader - Flush UDF and return to reset state"]
pub type FlushR = crate::FieldReader;
#[doc = "Field `flush` writer - Flush UDF and return to reset state"]
pub type FlushW<'a, REG> = crate::FieldWriter<'a, REG, 3>;
#[doc = "Field `reserved31` reader - reserved"]
pub type Reserved31R = crate::BitReader;
impl R {
    #[doc = "Bits 0:15 - Bits are internally XORed with i_custom"]
    #[inline(always)]
    pub fn salt(&self) -> SaltR {
        SaltR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:18 - Lock access to UDF"]
    #[inline(always)]
    pub fn lock(&self) -> LockR {
        LockR::new(((self.bits >> 16) & 7) as u8)
    }
    #[doc = "Bits 19:21 - RFU"]
    #[inline(always)]
    pub fn reserved21(&self) -> Reserved21R {
        Reserved21R::new(((self.bits >> 19) & 7) as u8)
    }
    #[doc = "Bits 22:24 - Enable the UDF block"]
    #[inline(always)]
    pub fn udf_en(&self) -> UdfEnR {
        UdfEnR::new(((self.bits >> 22) & 7) as u8)
    }
    #[doc = "Bit 25 - RFU"]
    #[inline(always)]
    pub fn reserved25(&self) -> Reserved25R {
        Reserved25R::new(((self.bits >> 25) & 1) != 0)
    }
    #[doc = "Bits 26:27 - RFU"]
    #[inline(always)]
    pub fn reserved27(&self) -> Reserved27R {
        Reserved27R::new(((self.bits >> 26) & 3) as u8)
    }
    #[doc = "Bits 28:30 - Flush UDF and return to reset state"]
    #[inline(always)]
    pub fn flush(&self) -> FlushR {
        FlushR::new(((self.bits >> 28) & 7) as u8)
    }
    #[doc = "Bit 31 - reserved"]
    #[inline(always)]
    pub fn reserved31(&self) -> Reserved31R {
        Reserved31R::new(((self.bits >> 31) & 1) != 0)
    }
}
impl W {
    #[doc = "Bits 0:15 - Bits are internally XORed with i_custom"]
    #[inline(always)]
    pub fn salt(&mut self) -> SaltW<UdfCtrlSpec> {
        SaltW::new(self, 0)
    }
    #[doc = "Bits 16:18 - Lock access to UDF"]
    #[inline(always)]
    pub fn lock(&mut self) -> LockW<UdfCtrlSpec> {
        LockW::new(self, 16)
    }
    #[doc = "Bits 19:21 - RFU"]
    #[inline(always)]
    pub fn reserved21(&mut self) -> Reserved21W<UdfCtrlSpec> {
        Reserved21W::new(self, 19)
    }
    #[doc = "Bits 22:24 - Enable the UDF block"]
    #[inline(always)]
    pub fn udf_en(&mut self) -> UdfEnW<UdfCtrlSpec> {
        UdfEnW::new(self, 22)
    }
    #[doc = "Bit 25 - RFU"]
    #[inline(always)]
    pub fn reserved25(&mut self) -> Reserved25W<UdfCtrlSpec> {
        Reserved25W::new(self, 25)
    }
    #[doc = "Bits 26:27 - RFU"]
    #[inline(always)]
    pub fn reserved27(&mut self) -> Reserved27W<UdfCtrlSpec> {
        Reserved27W::new(self, 26)
    }
    #[doc = "Bits 28:30 - Flush UDF and return to reset state"]
    #[inline(always)]
    pub fn flush(&mut self) -> FlushW<UdfCtrlSpec> {
        FlushW::new(self, 28)
    }
}
#[doc = "Control register\n\nYou can [`read`](crate::Reg::read) this register and get [`udf_ctrl::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`udf_ctrl::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct UdfCtrlSpec;
impl crate::RegisterSpec for UdfCtrlSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`udf_ctrl::R`](R) reader structure"]
impl crate::Readable for UdfCtrlSpec {}
#[doc = "`write(|w| ..)` method takes [`udf_ctrl::W`](W) writer structure"]
impl crate::Writable for UdfCtrlSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets udf_ctrl to value 0x0080_0000"]
impl crate::Resettable for UdfCtrlSpec {
    const RESET_VALUE: u32 = 0x0080_0000;
}

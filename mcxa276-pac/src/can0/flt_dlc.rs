#[doc = "Register `FLT_DLC` reader"]
pub type R = crate::R<FltDlcSpec>;
#[doc = "Register `FLT_DLC` writer"]
pub type W = crate::W<FltDlcSpec>;
#[doc = "Field `FLT_DLC_HI` reader - Upper Limit for Length of Data Bytes Filter"]
pub type FltDlcHiR = crate::FieldReader;
#[doc = "Field `FLT_DLC_HI` writer - Upper Limit for Length of Data Bytes Filter"]
pub type FltDlcHiW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Field `FLT_DLC_LO` reader - Lower Limit for Length of Data Bytes Filter"]
pub type FltDlcLoR = crate::FieldReader;
#[doc = "Field `FLT_DLC_LO` writer - Lower Limit for Length of Data Bytes Filter"]
pub type FltDlcLoW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
impl R {
    #[doc = "Bits 0:3 - Upper Limit for Length of Data Bytes Filter"]
    #[inline(always)]
    pub fn flt_dlc_hi(&self) -> FltDlcHiR {
        FltDlcHiR::new((self.bits & 0x0f) as u8)
    }
    #[doc = "Bits 16:19 - Lower Limit for Length of Data Bytes Filter"]
    #[inline(always)]
    pub fn flt_dlc_lo(&self) -> FltDlcLoR {
        FltDlcLoR::new(((self.bits >> 16) & 0x0f) as u8)
    }
}
impl W {
    #[doc = "Bits 0:3 - Upper Limit for Length of Data Bytes Filter"]
    #[inline(always)]
    pub fn flt_dlc_hi(&mut self) -> FltDlcHiW<FltDlcSpec> {
        FltDlcHiW::new(self, 0)
    }
    #[doc = "Bits 16:19 - Lower Limit for Length of Data Bytes Filter"]
    #[inline(always)]
    pub fn flt_dlc_lo(&mut self) -> FltDlcLoW<FltDlcSpec> {
        FltDlcLoW::new(self, 16)
    }
}
#[doc = "Pretended Networking Data Length Code (DLC) Filter\n\nYou can [`read`](crate::Reg::read) this register and get [`flt_dlc::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`flt_dlc::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct FltDlcSpec;
impl crate::RegisterSpec for FltDlcSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`flt_dlc::R`](R) reader structure"]
impl crate::Readable for FltDlcSpec {}
#[doc = "`write(|w| ..)` method takes [`flt_dlc::W`](W) writer structure"]
impl crate::Writable for FltDlcSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets FLT_DLC to value 0x08"]
impl crate::Resettable for FltDlcSpec {
    const RESET_VALUE: u32 = 0x08;
}

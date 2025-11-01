#[doc = "Register `PKRCNTDC` reader"]
pub type R = crate::R<PkrcntdcSpec>;
#[doc = "Field `PKR_C_CT` reader - Poker Ch Count"]
pub type PkrCCtR = crate::FieldReader<u16>;
#[doc = "Field `PKR_D_CT` reader - Poker Dh Count"]
pub type PkrDCtR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Poker Ch Count"]
    #[inline(always)]
    pub fn pkr_c_ct(&self) -> PkrCCtR {
        PkrCCtR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - Poker Dh Count"]
    #[inline(always)]
    pub fn pkr_d_ct(&self) -> PkrDCtR {
        PkrDCtR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
#[doc = "Statistical Check Poker Count D and C Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrcntdc::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkrcntdcSpec;
impl crate::RegisterSpec for PkrcntdcSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkrcntdc::R`](R) reader structure"]
impl crate::Readable for PkrcntdcSpec {}
#[doc = "`reset()` method sets PKRCNTDC to value 0"]
impl crate::Resettable for PkrcntdcSpec {}

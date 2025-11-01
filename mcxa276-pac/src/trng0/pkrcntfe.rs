#[doc = "Register `PKRCNTFE` reader"]
pub type R = crate::R<PkrcntfeSpec>;
#[doc = "Field `PKR_E_CT` reader - Poker Eh Count"]
pub type PkrECtR = crate::FieldReader<u16>;
#[doc = "Field `PKR_F_CT` reader - Poker Fh Count"]
pub type PkrFCtR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Poker Eh Count"]
    #[inline(always)]
    pub fn pkr_e_ct(&self) -> PkrECtR {
        PkrECtR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - Poker Fh Count"]
    #[inline(always)]
    pub fn pkr_f_ct(&self) -> PkrFCtR {
        PkrFCtR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
#[doc = "Statistical Check Poker Count F and E Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrcntfe::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkrcntfeSpec;
impl crate::RegisterSpec for PkrcntfeSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkrcntfe::R`](R) reader structure"]
impl crate::Readable for PkrcntfeSpec {}
#[doc = "`reset()` method sets PKRCNTFE to value 0"]
impl crate::Resettable for PkrcntfeSpec {}

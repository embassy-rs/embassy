#[doc = "Register `PKRCNTBA` reader"]
pub type R = crate::R<PkrcntbaSpec>;
#[doc = "Field `PKR_A_CT` reader - Poker Ah Count"]
pub type PkrACtR = crate::FieldReader<u16>;
#[doc = "Field `PKR_B_CT` reader - Poker Bh Count"]
pub type PkrBCtR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:15 - Poker Ah Count"]
    #[inline(always)]
    pub fn pkr_a_ct(&self) -> PkrACtR {
        PkrACtR::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bits 16:31 - Poker Bh Count"]
    #[inline(always)]
    pub fn pkr_b_ct(&self) -> PkrBCtR {
        PkrBCtR::new(((self.bits >> 16) & 0xffff) as u16)
    }
}
#[doc = "Statistical Check Poker Count B and A Register\n\nYou can [`read`](crate::Reg::read) this register and get [`pkrcntba::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct PkrcntbaSpec;
impl crate::RegisterSpec for PkrcntbaSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`pkrcntba::R`](R) reader structure"]
impl crate::Readable for PkrcntbaSpec {}
#[doc = "`reset()` method sets PKRCNTBA to value 0"]
impl crate::Resettable for PkrcntbaSpec {}

#[doc = "Reader of register RDLR"]
pub type R = crate::R<u32, super::RDLR>;
#[doc = "Reader of field `DATA3`"]
pub type DATA3_R = crate::R<u8, u8>;
#[doc = "Reader of field `DATA2`"]
pub type DATA2_R = crate::R<u8, u8>;
#[doc = "Reader of field `DATA1`"]
pub type DATA1_R = crate::R<u8, u8>;
#[doc = "Reader of field `DATA0`"]
pub type DATA0_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bits 24:31 - DATA3"]
    #[inline(always)]
    pub fn data3(&self) -> DATA3_R {
        DATA3_R::new(((self.bits >> 24) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - DATA2"]
    #[inline(always)]
    pub fn data2(&self) -> DATA2_R {
        DATA2_R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - DATA1"]
    #[inline(always)]
    pub fn data1(&self) -> DATA1_R {
        DATA1_R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 0:7 - DATA0"]
    #[inline(always)]
    pub fn data0(&self) -> DATA0_R {
        DATA0_R::new((self.bits & 0xff) as u8)
    }
}

#[doc = "Reader of register RDHR"]
pub type R = crate::R<u32, super::RDHR>;
#[doc = "Reader of field `DATA7`"]
pub type DATA7_R = crate::R<u8, u8>;
#[doc = "Reader of field `DATA6`"]
pub type DATA6_R = crate::R<u8, u8>;
#[doc = "Reader of field `DATA5`"]
pub type DATA5_R = crate::R<u8, u8>;
#[doc = "Reader of field `DATA4`"]
pub type DATA4_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bits 24:31 - DATA7"]
    #[inline(always)]
    pub fn data7(&self) -> DATA7_R {
        DATA7_R::new(((self.bits >> 24) & 0xff) as u8)
    }
    #[doc = "Bits 16:23 - DATA6"]
    #[inline(always)]
    pub fn data6(&self) -> DATA6_R {
        DATA6_R::new(((self.bits >> 16) & 0xff) as u8)
    }
    #[doc = "Bits 8:15 - DATA5"]
    #[inline(always)]
    pub fn data5(&self) -> DATA5_R {
        DATA5_R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 0:7 - DATA4"]
    #[inline(always)]
    pub fn data4(&self) -> DATA4_R {
        DATA4_R::new((self.bits & 0xff) as u8)
    }
}

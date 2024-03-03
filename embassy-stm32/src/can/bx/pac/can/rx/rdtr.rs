#[doc = "Reader of register RDTR"]
pub type R = crate::R<u32, super::RDTR>;
#[doc = "Reader of field `TIME`"]
pub type TIME_R = crate::R<u16, u16>;
#[doc = "Reader of field `FMI`"]
pub type FMI_R = crate::R<u8, u8>;
#[doc = "Reader of field `DLC`"]
pub type DLC_R = crate::R<u8, u8>;
impl R {
    #[doc = "Bits 16:31 - TIME"]
    #[inline(always)]
    pub fn time(&self) -> TIME_R {
        TIME_R::new(((self.bits >> 16) & 0xffff) as u16)
    }
    #[doc = "Bits 8:15 - FMI"]
    #[inline(always)]
    pub fn fmi(&self) -> FMI_R {
        FMI_R::new(((self.bits >> 8) & 0xff) as u8)
    }
    #[doc = "Bits 0:3 - DLC"]
    #[inline(always)]
    pub fn dlc(&self) -> DLC_R {
        DLC_R::new((self.bits & 0x0f) as u8)
    }
}

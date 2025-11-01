#[doc = "Register `sgi_status` reader"]
pub type R = crate::R<SgiStatusSpec>;
#[doc = "Register `sgi_status` writer"]
pub type W = crate::W<SgiStatusSpec>;
#[doc = "Field `busy` reader - Combined busy flag that remains high"]
pub type BusyR = crate::BitReader;
#[doc = "Field `oflow` reader - Overflow in INCR operation flag"]
pub type OflowR = crate::BitReader;
#[doc = "Field `oflow` writer - Overflow in INCR operation flag"]
pub type OflowW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `prng_rdy` reader - prng is ready after boot-up-phase"]
pub type PrngRdyR = crate::BitReader;
#[doc = "Error detected\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Error {
    #[doc = "0: ERROR(all values other than 0x05 indicate ERROR)"]
    Error = 0,
    #[doc = "5: NO_ERROR"]
    NoError = 5,
}
impl From<Error> for u8 {
    #[inline(always)]
    fn from(variant: Error) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Error {
    type Ux = u8;
}
impl crate::IsEnum for Error {}
#[doc = "Field `error` reader - Error detected"]
pub type ErrorR = crate::FieldReader<Error>;
impl ErrorR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Error> {
        match self.bits {
            0 => Some(Error::Error),
            5 => Some(Error::NoError),
            _ => None,
        }
    }
    #[doc = "ERROR(all values other than 0x05 indicate ERROR)"]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Error::Error
    }
    #[doc = "NO_ERROR"]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Error::NoError
    }
}
#[doc = "Field `error` writer - Error detected"]
pub type ErrorW<'a, REG> = crate::FieldWriter<'a, REG, 3, Error>;
impl<'a, REG> ErrorW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "ERROR(all values other than 0x05 indicate ERROR)"]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Error::Error)
    }
    #[doc = "NO_ERROR"]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Error::NoError)
    }
}
#[doc = "Field `sha2_busy` reader - SHA2 is busy"]
pub type Sha2BusyR = crate::BitReader;
#[doc = "Field `irq` reader - interrupt detected"]
pub type IrqR = crate::BitReader;
#[doc = "Field `irq` writer - interrupt detected"]
pub type IrqW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `sha_fifo_full` reader - SHA FIFO is full(operates in SHA AUTO mode)"]
pub type ShaFifoFullR = crate::BitReader;
#[doc = "Field `sha_fifo_level` reader - SHA FIFO level"]
pub type ShaFifoLevelR = crate::FieldReader;
#[doc = "Field `sha_error` reader - SHA ERROR"]
pub type ShaErrorR = crate::BitReader;
#[doc = "Field `key_read_err` reader - KEY SFR READ ERROR, sticky, cleared only with reset or flush"]
pub type KeyReadErrR = crate::BitReader;
#[doc = "Field `key_read_err` writer - KEY SFR READ ERROR, sticky, cleared only with reset or flush"]
pub type KeyReadErrW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `key_unwrap_err` reader - KEY UNWRAP ERROR , sticky, cleared only with reset or flush"]
pub type KeyUnwrapErrR = crate::BitReader;
#[doc = "Field `key_unwrap_err` writer - KEY UNWRAP ERROR , sticky, cleared only with reset or flush"]
pub type KeyUnwrapErrW<'a, REG> = crate::BitWriter<'a, REG>;
#[doc = "Field `status_rsvd3` reader - reserved"]
pub type StatusRsvd3R = crate::BitReader;
#[doc = "Field `status_rsvd` reader - reserved"]
pub type StatusRsvdR = crate::FieldReader<u16>;
impl R {
    #[doc = "Bit 0 - Combined busy flag that remains high"]
    #[inline(always)]
    pub fn busy(&self) -> BusyR {
        BusyR::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Overflow in INCR operation flag"]
    #[inline(always)]
    pub fn oflow(&self) -> OflowR {
        OflowR::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - prng is ready after boot-up-phase"]
    #[inline(always)]
    pub fn prng_rdy(&self) -> PrngRdyR {
        PrngRdyR::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bits 3:5 - Error detected"]
    #[inline(always)]
    pub fn error(&self) -> ErrorR {
        ErrorR::new(((self.bits >> 3) & 7) as u8)
    }
    #[doc = "Bit 6 - SHA2 is busy"]
    #[inline(always)]
    pub fn sha2_busy(&self) -> Sha2BusyR {
        Sha2BusyR::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - interrupt detected"]
    #[inline(always)]
    pub fn irq(&self) -> IrqR {
        IrqR::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 8 - SHA FIFO is full(operates in SHA AUTO mode)"]
    #[inline(always)]
    pub fn sha_fifo_full(&self) -> ShaFifoFullR {
        ShaFifoFullR::new(((self.bits >> 8) & 1) != 0)
    }
    #[doc = "Bits 9:14 - SHA FIFO level"]
    #[inline(always)]
    pub fn sha_fifo_level(&self) -> ShaFifoLevelR {
        ShaFifoLevelR::new(((self.bits >> 9) & 0x3f) as u8)
    }
    #[doc = "Bit 15 - SHA ERROR"]
    #[inline(always)]
    pub fn sha_error(&self) -> ShaErrorR {
        ShaErrorR::new(((self.bits >> 15) & 1) != 0)
    }
    #[doc = "Bit 16 - KEY SFR READ ERROR, sticky, cleared only with reset or flush"]
    #[inline(always)]
    pub fn key_read_err(&self) -> KeyReadErrR {
        KeyReadErrR::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - KEY UNWRAP ERROR , sticky, cleared only with reset or flush"]
    #[inline(always)]
    pub fn key_unwrap_err(&self) -> KeyUnwrapErrR {
        KeyUnwrapErrR::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - reserved"]
    #[inline(always)]
    pub fn status_rsvd3(&self) -> StatusRsvd3R {
        StatusRsvd3R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bits 19:31 - reserved"]
    #[inline(always)]
    pub fn status_rsvd(&self) -> StatusRsvdR {
        StatusRsvdR::new(((self.bits >> 19) & 0x1fff) as u16)
    }
}
impl W {
    #[doc = "Bit 1 - Overflow in INCR operation flag"]
    #[inline(always)]
    pub fn oflow(&mut self) -> OflowW<SgiStatusSpec> {
        OflowW::new(self, 1)
    }
    #[doc = "Bits 3:5 - Error detected"]
    #[inline(always)]
    pub fn error(&mut self) -> ErrorW<SgiStatusSpec> {
        ErrorW::new(self, 3)
    }
    #[doc = "Bit 7 - interrupt detected"]
    #[inline(always)]
    pub fn irq(&mut self) -> IrqW<SgiStatusSpec> {
        IrqW::new(self, 7)
    }
    #[doc = "Bit 16 - KEY SFR READ ERROR, sticky, cleared only with reset or flush"]
    #[inline(always)]
    pub fn key_read_err(&mut self) -> KeyReadErrW<SgiStatusSpec> {
        KeyReadErrW::new(self, 16)
    }
    #[doc = "Bit 17 - KEY UNWRAP ERROR , sticky, cleared only with reset or flush"]
    #[inline(always)]
    pub fn key_unwrap_err(&mut self) -> KeyUnwrapErrW<SgiStatusSpec> {
        KeyUnwrapErrW::new(self, 17)
    }
}
#[doc = "Status register\n\nYou can [`read`](crate::Reg::read) this register and get [`sgi_status::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`sgi_status::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct SgiStatusSpec;
impl crate::RegisterSpec for SgiStatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`sgi_status::R`](R) reader structure"]
impl crate::Readable for SgiStatusSpec {}
#[doc = "`write(|w| ..)` method takes [`sgi_status::W`](W) writer structure"]
impl crate::Writable for SgiStatusSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets sgi_status to value 0"]
impl crate::Resettable for SgiStatusSpec {}

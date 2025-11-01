#[doc = "Register `GEXP_STATUS` reader"]
pub type R = crate::R<GexpStatusSpec>;
#[doc = "Register `GEXP_STATUS` writer"]
pub type W = crate::W<GexpStatusSpec>;
#[doc = "Direct operation Error\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    #[doc = "0: No error is generated."]
    NoError = 0,
    #[doc = "1: An error is generated."]
    Error = 1,
}
impl From<Error> for bool {
    #[inline(always)]
    fn from(variant: Error) -> Self {
        variant as u8 != 0
    }
}
#[doc = "Field `ERROR` reader - Direct operation Error"]
pub type ErrorR = crate::BitReader<Error>;
impl ErrorR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Error {
        match self.bits {
            false => Error::NoError,
            true => Error::Error,
        }
    }
    #[doc = "No error is generated."]
    #[inline(always)]
    pub fn is_no_error(&self) -> bool {
        *self == Error::NoError
    }
    #[doc = "An error is generated."]
    #[inline(always)]
    pub fn is_error(&self) -> bool {
        *self == Error::Error
    }
}
#[doc = "Field `ERROR` writer - Direct operation Error"]
pub type ErrorW<'a, REG> = crate::BitWriter<'a, REG, Error>;
impl<'a, REG> ErrorW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
{
    #[doc = "No error is generated."]
    #[inline(always)]
    pub fn no_error(self) -> &'a mut crate::W<REG> {
        self.variant(Error::NoError)
    }
    #[doc = "An error is generated."]
    #[inline(always)]
    pub fn error(self) -> &'a mut crate::W<REG> {
        self.variant(Error::Error)
    }
}
impl R {
    #[doc = "Bit 0 - Direct operation Error"]
    #[inline(always)]
    pub fn error(&self) -> ErrorR {
        ErrorR::new((self.bits & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Direct operation Error"]
    #[inline(always)]
    pub fn error(&mut self) -> ErrorW<GexpStatusSpec> {
        ErrorW::new(self, 0)
    }
}
#[doc = "General Exception Status\n\nYou can [`read`](crate::Reg::read) this register and get [`gexp_status::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gexp_status::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct GexpStatusSpec;
impl crate::RegisterSpec for GexpStatusSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`gexp_status::R`](R) reader structure"]
impl crate::Readable for GexpStatusSpec {}
#[doc = "`write(|w| ..)` method takes [`gexp_status::W`](W) writer structure"]
impl crate::Writable for GexpStatusSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets GEXP_STATUS to value 0"]
impl crate::Resettable for GexpStatusSpec {}

#[doc = "Register `TOKEN` reader"]
pub type R = crate::R<TokenSpec>;
#[doc = "Register `TOKEN` writer"]
pub type W = crate::W<TokenSpec>;
#[doc = "Field `TOKENENDPT` reader - Token Endpoint Address"]
pub type TokenendptR = crate::FieldReader;
#[doc = "Field `TOKENENDPT` writer - Token Endpoint Address"]
pub type TokenendptW<'a, REG> = crate::FieldWriter<'a, REG, 4>;
#[doc = "Token Type\n\nValue on reset: 0"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tokenpid {
    #[doc = "1: OUT token. USBFS performs an OUT (TX) transaction."]
    EnOutToken = 1,
    #[doc = "9: IN token. USBFS performs an IN (RX) transaction."]
    EnInToken = 9,
    #[doc = "13: SETUP token. USBFS performs a SETUP (TX) transaction"]
    EnSetupToken = 13,
}
impl From<Tokenpid> for u8 {
    #[inline(always)]
    fn from(variant: Tokenpid) -> Self {
        variant as _
    }
}
impl crate::FieldSpec for Tokenpid {
    type Ux = u8;
}
impl crate::IsEnum for Tokenpid {}
#[doc = "Field `TOKENPID` reader - Token Type"]
pub type TokenpidR = crate::FieldReader<Tokenpid>;
impl TokenpidR {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub const fn variant(&self) -> Option<Tokenpid> {
        match self.bits {
            1 => Some(Tokenpid::EnOutToken),
            9 => Some(Tokenpid::EnInToken),
            13 => Some(Tokenpid::EnSetupToken),
            _ => None,
        }
    }
    #[doc = "OUT token. USBFS performs an OUT (TX) transaction."]
    #[inline(always)]
    pub fn is_en_out_token(&self) -> bool {
        *self == Tokenpid::EnOutToken
    }
    #[doc = "IN token. USBFS performs an IN (RX) transaction."]
    #[inline(always)]
    pub fn is_en_in_token(&self) -> bool {
        *self == Tokenpid::EnInToken
    }
    #[doc = "SETUP token. USBFS performs a SETUP (TX) transaction"]
    #[inline(always)]
    pub fn is_en_setup_token(&self) -> bool {
        *self == Tokenpid::EnSetupToken
    }
}
#[doc = "Field `TOKENPID` writer - Token Type"]
pub type TokenpidW<'a, REG> = crate::FieldWriter<'a, REG, 4, Tokenpid>;
impl<'a, REG> TokenpidW<'a, REG>
where
    REG: crate::Writable + crate::RegisterSpec,
    REG::Ux: From<u8>,
{
    #[doc = "OUT token. USBFS performs an OUT (TX) transaction."]
    #[inline(always)]
    pub fn en_out_token(self) -> &'a mut crate::W<REG> {
        self.variant(Tokenpid::EnOutToken)
    }
    #[doc = "IN token. USBFS performs an IN (RX) transaction."]
    #[inline(always)]
    pub fn en_in_token(self) -> &'a mut crate::W<REG> {
        self.variant(Tokenpid::EnInToken)
    }
    #[doc = "SETUP token. USBFS performs a SETUP (TX) transaction"]
    #[inline(always)]
    pub fn en_setup_token(self) -> &'a mut crate::W<REG> {
        self.variant(Tokenpid::EnSetupToken)
    }
}
impl R {
    #[doc = "Bits 0:3 - Token Endpoint Address"]
    #[inline(always)]
    pub fn tokenendpt(&self) -> TokenendptR {
        TokenendptR::new(self.bits & 0x0f)
    }
    #[doc = "Bits 4:7 - Token Type"]
    #[inline(always)]
    pub fn tokenpid(&self) -> TokenpidR {
        TokenpidR::new((self.bits >> 4) & 0x0f)
    }
}
impl W {
    #[doc = "Bits 0:3 - Token Endpoint Address"]
    #[inline(always)]
    pub fn tokenendpt(&mut self) -> TokenendptW<TokenSpec> {
        TokenendptW::new(self, 0)
    }
    #[doc = "Bits 4:7 - Token Type"]
    #[inline(always)]
    pub fn tokenpid(&mut self) -> TokenpidW<TokenSpec> {
        TokenpidW::new(self, 4)
    }
}
#[doc = "Token\n\nYou can [`read`](crate::Reg::read) this register and get [`token::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`token::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct TokenSpec;
impl crate::RegisterSpec for TokenSpec {
    type Ux = u8;
}
#[doc = "`read()` method returns [`token::R`](R) reader structure"]
impl crate::Readable for TokenSpec {}
#[doc = "`write(|w| ..)` method takes [`token::W`](W) writer structure"]
impl crate::Writable for TokenSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets TOKEN to value 0"]
impl crate::Resettable for TokenSpec {}

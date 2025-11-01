#[doc = "Register `GRAY_CODE_MSB` reader"]
pub type R = crate::R<GrayCodeMsbSpec>;
#[doc = "Register `GRAY_CODE_MSB` writer"]
pub type W = crate::W<GrayCodeMsbSpec>;
#[doc = "Field `code_gray_41_32` reader - Gray code \\[41:32\\]"]
pub type CodeGray41_32R = crate::FieldReader<u16>;
#[doc = "Field `code_gray_41_32` writer - Gray code \\[41:32\\]"]
pub type CodeGray41_32W<'a, REG> = crate::FieldWriter<'a, REG, 10, u16>;
impl R {
    #[doc = "Bits 0:9 - Gray code \\[41:32\\]"]
    #[inline(always)]
    pub fn code_gray_41_32(&self) -> CodeGray41_32R {
        CodeGray41_32R::new((self.bits & 0x03ff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:9 - Gray code \\[41:32\\]"]
    #[inline(always)]
    pub fn code_gray_41_32(&mut self) -> CodeGray41_32W<GrayCodeMsbSpec> {
        CodeGray41_32W::new(self, 0)
    }
}
#[doc = "Gray to Binary Converter Gray Code \\[41:32\\]\n\nYou can [`read`](crate::Reg::read) this register and get [`gray_code_msb::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gray_code_msb::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct GrayCodeMsbSpec;
impl crate::RegisterSpec for GrayCodeMsbSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`gray_code_msb::R`](R) reader structure"]
impl crate::Readable for GrayCodeMsbSpec {}
#[doc = "`write(|w| ..)` method takes [`gray_code_msb::W`](W) writer structure"]
impl crate::Writable for GrayCodeMsbSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets GRAY_CODE_MSB to value 0"]
impl crate::Resettable for GrayCodeMsbSpec {}

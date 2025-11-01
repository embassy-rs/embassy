#[doc = "Register `GRAY_CODE_LSB` reader"]
pub type R = crate::R<GrayCodeLsbSpec>;
#[doc = "Register `GRAY_CODE_LSB` writer"]
pub type W = crate::W<GrayCodeLsbSpec>;
#[doc = "Field `code_gray_31_0` reader - Gray code \\[31:0\\]"]
pub type CodeGray31_0R = crate::FieldReader<u32>;
#[doc = "Field `code_gray_31_0` writer - Gray code \\[31:0\\]"]
pub type CodeGray31_0W<'a, REG> = crate::FieldWriter<'a, REG, 32, u32>;
impl R {
    #[doc = "Bits 0:31 - Gray code \\[31:0\\]"]
    #[inline(always)]
    pub fn code_gray_31_0(&self) -> CodeGray31_0R {
        CodeGray31_0R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Gray code \\[31:0\\]"]
    #[inline(always)]
    pub fn code_gray_31_0(&mut self) -> CodeGray31_0W<GrayCodeLsbSpec> {
        CodeGray31_0W::new(self, 0)
    }
}
#[doc = "Gray to Binary Converter Gray Code \\[31:0\\]\n\nYou can [`read`](crate::Reg::read) this register and get [`gray_code_lsb::R`](R). You can [`reset`](crate::Reg::reset), [`write`](crate::Reg::write), [`write_with_zero`](crate::Reg::write_with_zero) this register using [`gray_code_lsb::W`](W). You can also [`modify`](crate::Reg::modify) this register. See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct GrayCodeLsbSpec;
impl crate::RegisterSpec for GrayCodeLsbSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`gray_code_lsb::R`](R) reader structure"]
impl crate::Readable for GrayCodeLsbSpec {}
#[doc = "`write(|w| ..)` method takes [`gray_code_lsb::W`](W) writer structure"]
impl crate::Writable for GrayCodeLsbSpec {
    type Safety = crate::Unsafe;
}
#[doc = "`reset()` method sets GRAY_CODE_LSB to value 0"]
impl crate::Resettable for GrayCodeLsbSpec {}

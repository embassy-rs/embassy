#[doc = "Register `BINARY_CODE_LSB` reader"]
pub type R = crate::R<BinaryCodeLsbSpec>;
#[doc = "Field `code_bin_31_0` reader - Binary code \\[31:0\\]"]
pub type CodeBin31_0R = crate::FieldReader<u32>;
impl R {
    #[doc = "Bits 0:31 - Binary code \\[31:0\\]"]
    #[inline(always)]
    pub fn code_bin_31_0(&self) -> CodeBin31_0R {
        CodeBin31_0R::new(self.bits)
    }
}
#[doc = "Gray to Binary Converter Binary Code \\[31:0\\]\n\nYou can [`read`](crate::Reg::read) this register and get [`binary_code_lsb::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct BinaryCodeLsbSpec;
impl crate::RegisterSpec for BinaryCodeLsbSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`binary_code_lsb::R`](R) reader structure"]
impl crate::Readable for BinaryCodeLsbSpec {}
#[doc = "`reset()` method sets BINARY_CODE_LSB to value 0"]
impl crate::Resettable for BinaryCodeLsbSpec {}

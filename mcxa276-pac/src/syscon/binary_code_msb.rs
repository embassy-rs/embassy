#[doc = "Register `BINARY_CODE_MSB` reader"]
pub type R = crate::R<BinaryCodeMsbSpec>;
#[doc = "Field `code_bin_41_32` reader - Binary code \\[41:32\\]"]
pub type CodeBin41_32R = crate::FieldReader<u16>;
impl R {
    #[doc = "Bits 0:9 - Binary code \\[41:32\\]"]
    #[inline(always)]
    pub fn code_bin_41_32(&self) -> CodeBin41_32R {
        CodeBin41_32R::new((self.bits & 0x03ff) as u16)
    }
}
#[doc = "Gray to Binary Converter Binary Code \\[41:32\\]\n\nYou can [`read`](crate::Reg::read) this register and get [`binary_code_msb::R`](R). See [API](https://docs.rs/svd2rust/#read--modify--write-api)."]
pub struct BinaryCodeMsbSpec;
impl crate::RegisterSpec for BinaryCodeMsbSpec {
    type Ux = u32;
}
#[doc = "`read()` method returns [`binary_code_msb::R`](R) reader structure"]
impl crate::Readable for BinaryCodeMsbSpec {}
#[doc = "`reset()` method sets BINARY_CODE_MSB to value 0"]
impl crate::Resettable for BinaryCodeMsbSpec {}

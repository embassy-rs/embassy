#[doc = "CAN_TI0R\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tir](tir) module"]
pub type TIR = crate::Reg<u32, _TIR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TIR;
#[doc = "`read()` method returns [tir::R](tir::R) reader structure"]
impl crate::Readable for TIR {}
#[doc = "`write(|w| ..)` method takes [tir::W](tir::W) writer structure"]
impl crate::Writable for TIR {}
#[doc = "CAN_TI0R"]
pub mod tir;
#[doc = "CAN_TDT0R\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tdtr](tdtr) module"]
pub type TDTR = crate::Reg<u32, _TDTR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TDTR;
#[doc = "`read()` method returns [tdtr::R](tdtr::R) reader structure"]
impl crate::Readable for TDTR {}
#[doc = "`write(|w| ..)` method takes [tdtr::W](tdtr::W) writer structure"]
impl crate::Writable for TDTR {}
#[doc = "CAN_TDT0R"]
pub mod tdtr;
#[doc = "CAN_TDL0R\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tdlr](tdlr) module"]
pub type TDLR = crate::Reg<u32, _TDLR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TDLR;
#[doc = "`read()` method returns [tdlr::R](tdlr::R) reader structure"]
impl crate::Readable for TDLR {}
#[doc = "`write(|w| ..)` method takes [tdlr::W](tdlr::W) writer structure"]
impl crate::Writable for TDLR {}
#[doc = "CAN_TDL0R"]
pub mod tdlr;
#[doc = "CAN_TDH0R\n\nThis register you can [`read`](crate::generic::Reg::read), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tdhr](tdhr) module"]
pub type TDHR = crate::Reg<u32, _TDHR>;
#[allow(missing_docs)]
#[doc(hidden)]
pub struct _TDHR;
#[doc = "`read()` method returns [tdhr::R](tdhr::R) reader structure"]
impl crate::Readable for TDHR {}
#[doc = "`write(|w| ..)` method takes [tdhr::W](tdhr::W) writer structure"]
impl crate::Writable for TDHR {}
#[doc = "CAN_TDH0R"]
pub mod tdhr;

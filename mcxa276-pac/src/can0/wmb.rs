#[repr(C)]
#[doc = "Array of registers: WMB_CS, WMB_D03, WMB_D47, WMB_ID"]
#[doc(alias = "WMB")]
pub struct Wmb {
    wmb_cs: WmbCs,
    wmb_id: WmbId,
    wmb_d03: WmbD03,
    wmb_d47: WmbD47,
}
impl Wmb {
    #[doc = "0x00 - Wake-Up Message Buffer"]
    #[inline(always)]
    pub const fn wmb_cs(&self) -> &WmbCs {
        &self.wmb_cs
    }
    #[doc = "0x04 - Wake-Up Message Buffer for ID"]
    #[inline(always)]
    pub const fn wmb_id(&self) -> &WmbId {
        &self.wmb_id
    }
    #[doc = "0x08 - Wake-Up Message Buffer for Data 0-3"]
    #[inline(always)]
    pub const fn wmb_d03(&self) -> &WmbD03 {
        &self.wmb_d03
    }
    #[doc = "0x0c - Wake-Up Message Buffer Register Data 4-7"]
    #[inline(always)]
    pub const fn wmb_d47(&self) -> &WmbD47 {
        &self.wmb_d47
    }
}
#[doc = "WMB_CS (r) register accessor: Wake-Up Message Buffer\n\nYou can [`read`](crate::Reg::read) this register and get [`wmb_cs::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@wmb_cs`] module"]
#[doc(alias = "WMB_CS")]
pub type WmbCs = crate::Reg<wmb_cs::WmbCsSpec>;
#[doc = "Wake-Up Message Buffer"]
pub mod wmb_cs;
#[doc = "WMB_ID (r) register accessor: Wake-Up Message Buffer for ID\n\nYou can [`read`](crate::Reg::read) this register and get [`wmb_id::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@wmb_id`] module"]
#[doc(alias = "WMB_ID")]
pub type WmbId = crate::Reg<wmb_id::WmbIdSpec>;
#[doc = "Wake-Up Message Buffer for ID"]
pub mod wmb_id;
#[doc = "WMB_D03 (r) register accessor: Wake-Up Message Buffer for Data 0-3\n\nYou can [`read`](crate::Reg::read) this register and get [`wmb_d03::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@wmb_d03`] module"]
#[doc(alias = "WMB_D03")]
pub type WmbD03 = crate::Reg<wmb_d03::WmbD03Spec>;
#[doc = "Wake-Up Message Buffer for Data 0-3"]
pub mod wmb_d03;
#[doc = "WMB_D47 (r) register accessor: Wake-Up Message Buffer Register Data 4-7\n\nYou can [`read`](crate::Reg::read) this register and get [`wmb_d47::R`]. See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [`mod@wmb_d47`] module"]
#[doc(alias = "WMB_D47")]
pub type WmbD47 = crate::Reg<wmb_d47::WmbD47Spec>;
#[doc = "Wake-Up Message Buffer Register Data 4-7"]
pub mod wmb_d47;

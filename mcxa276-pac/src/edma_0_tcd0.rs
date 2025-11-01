#[repr(C)]
#[doc = "Register block"]
pub struct RegisterBlock {
    tcd: (),
}
impl RegisterBlock {
    #[doc = "0x00..0x200 - Array of registers: CH_CSR, CH_ES, CH_INT, CH_MUX, CH_PRI, CH_SBR, TCD_ATTR, TCD_BITER_ELINKNO, TCD_BITER_ELINKYES, TCD_CITER_ELINKNO, TCD_CITER_ELINKYES, TCD_CSR, TCD_DADDR, TCD_DLAST_SGA, TCD_DOFF, TCD_NBYTES_MLOFFNO, TCD_NBYTES_MLOFFYES, TCD_SADDR, TCD_SLAST_SDA, TCD_SOFF"]
    #[inline(always)]
    pub const fn tcd(&self, n: usize) -> &Tcd {
        #[allow(clippy::no_effect)]
        [(); 8][n];
        unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(4096 * n).cast() }
    }
    #[doc = "Iterator for array of:"]
    #[doc = "0x00..0x200 - Array of registers: CH_CSR, CH_ES, CH_INT, CH_MUX, CH_PRI, CH_SBR, TCD_ATTR, TCD_BITER_ELINKNO, TCD_BITER_ELINKYES, TCD_CITER_ELINKNO, TCD_CITER_ELINKYES, TCD_CSR, TCD_DADDR, TCD_DLAST_SGA, TCD_DOFF, TCD_NBYTES_MLOFFNO, TCD_NBYTES_MLOFFYES, TCD_SADDR, TCD_SLAST_SDA, TCD_SOFF"]
    #[inline(always)]
    pub fn tcd_iter(&self) -> impl Iterator<Item = &Tcd> {
        (0..8)
            .map(move |n| unsafe { &*core::ptr::from_ref(self).cast::<u8>().add(4096 * n).cast() })
    }
}
#[doc = "Array of registers: CH_CSR, CH_ES, CH_INT, CH_MUX, CH_PRI, CH_SBR, TCD_ATTR, TCD_BITER_ELINKNO, TCD_BITER_ELINKYES, TCD_CITER_ELINKNO, TCD_CITER_ELINKYES, TCD_CSR, TCD_DADDR, TCD_DLAST_SGA, TCD_DOFF, TCD_NBYTES_MLOFFNO, TCD_NBYTES_MLOFFYES, TCD_SADDR, TCD_SLAST_SDA, TCD_SOFF"]
pub use self::tcd::Tcd;
#[doc = r"Cluster"]
#[doc = "Array of registers: CH_CSR, CH_ES, CH_INT, CH_MUX, CH_PRI, CH_SBR, TCD_ATTR, TCD_BITER_ELINKNO, TCD_BITER_ELINKYES, TCD_CITER_ELINKNO, TCD_CITER_ELINKYES, TCD_CSR, TCD_DADDR, TCD_DLAST_SGA, TCD_DOFF, TCD_NBYTES_MLOFFNO, TCD_NBYTES_MLOFFYES, TCD_SADDR, TCD_SLAST_SDA, TCD_SOFF"]
pub mod tcd;

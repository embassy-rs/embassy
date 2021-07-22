pub struct Dbgmcu {}

impl Dbgmcu {
    pub unsafe fn enable_all() {
        crate::pac::DBGMCU.cr().modify(|cr| {
            crate::pac::dbgmcu! {
                (cr, $fn_name:ident) => {
                    cr.$fn_name(true);
                };
            }
        });
    }
}

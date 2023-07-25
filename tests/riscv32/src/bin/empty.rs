#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Spawner;

use crate::pac::PLIC;

mod pac {
    use embassy_hal_common::interrupt::Priority;

    // PLIC context at 0xc00_0000.
    riscv::plic_context!(PLIC, 0xc00_0000, 0, Interrupt, Priority);

    unsafe impl embassy_hal_common::interrupt::InterruptExt for Interrupt {
        unsafe fn enable(self) {
            let mut plic: PLIC = core::mem::transmute(());
            plic.enable_interrupt(self)
        }

        fn disable(self) {
            unsafe {
                let mut plic: PLIC = core::mem::transmute(());
                plic.disable_interrupt(self)
            }
        }

        fn is_enabled(self) -> bool {
            PLIC::is_interrupt_enabled(self)
        }

        fn is_pending(self) -> bool {
            PLIC::is_interrupt_pending(self)
        }

        fn pend(self) {
            todo!()
        }

        fn unpend(self) {
            todo!()
        }

        fn get_priority(self) -> Priority {
            PLIC::priority(self)
        }

        fn set_priority(self, prio: Priority) {
            unsafe {
                let mut plic: PLIC = core::mem::transmute(());
                plic.set_priority(self, prio);
            }
        }
    }

    #[derive(Copy, Clone)]
    pub enum Interrupt {
        INT0,
    }

    unsafe impl riscv::peripheral::plic::InterruptNumber for Interrupt {
        const MAX_INTERRUPT_NUMBER: u16 = 1;

        fn number(self) -> u16 {
            1
        }

        fn try_from(value: u16) -> Result<Self, u16> {
            match value {
                1 => Ok(Interrupt::INT0),
                v => Err(v),
            }
        }
    }
}

embassy_hal_common::interrupt_mod_core!(INT0);

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    use embassy_hal_common::interrupt::{InterruptExt, Priority};

    use crate::interrupt::INT0;

    // Example, running not recommended.
    unsafe { INT0.enable() };
    INT0.disable();
    let _ = INT0.get_priority();
    // `P0` is the highest priority.
    INT0.set_priority(Priority::P0);

    unsafe {
        let mut plic: PLIC = core::mem::transmute(());
        plic.set_threshold(Priority::P3);
    };

    unsafe {
        // Enable all interrupts in the current Hart.
        riscv::interrupt::enable();
        // Enable all interrupts in the PLIC.
        PLIC::enable();
    };

    // Don't do anything, just make sure it compiles.
    loop {}
}

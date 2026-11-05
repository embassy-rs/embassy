#![no_std]
#![no_main]

//! Single-bit ECC diagnostics. Do not open debugger memory views in ECC SRAM.
//! The per-bin layout keeps stack, ordinary data and RTT above 0x20010000.

use core::ptr::{read_volatile, write_volatile};

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa::eim::{Bit, Eim};
use embassy_mcxa::erm::{Erm, Events, Report, SramEvents};
use embassy_mcxa::pac;
use panic_probe as _;

unsafe extern "C" {
    static mut __ecc_scratch: u32;
}

#[used]
#[unsafe(link_section = ".ecc_scratch")]
static mut ECC_SCRATCH: [u32; 2] = [0; 2];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_mcxa::init(Default::default());
    // Observe the existing policy; never enable ECC over live memory.
    assert!(
        pac::SYSCON.ram_ctrl().read().rama_ecc_enable(),
        "RAM A ECC must already be enabled"
    );
    let mut eim = Eim::new(p.EIM0).unwrap();
    let mut erm = Erm::new_blocking(p.ERM0).unwrap();
    let scratch = core::ptr::addr_of_mut!(__ecc_scratch);
    assert_eq!(scratch as usize, 0x2000_0000);
    let words = [0x1234_5678, 0xa5a5_5a5a];
    // SAFETY: the linker reserves two aligned words, excluded from all ordinary
    // allocations. Full-word writes establish ECC while injection is disabled.
    unsafe {
        write_volatile(scratch, words[0]);
        write_volatile(scratch.add(1), words[1]);
    }
    cortex_m::asm::dsb();
    erm.acknowledge(Events::ALL);
    erm.reset_corrected_count();

    for (index, expected) in words.into_iter().enumerate() {
        // SAFETY: initialized reserved ECC word; no DMA or other tasks are
        // running. Debugger RTT reads are outside ECC SRAM. The bounded pulse
        // masks ordinary interrupts and disarms before returning.
        let value = unsafe { eim.inject_read(scratch.add(index), Bit::data(0).unwrap()) };
        assert_eq!(value, expected);
        assert!(!eim.is_enabled());
        assert!(!pac::EIM0.eimcr().read().geien());
        assert!(!pac::EIM0.eichen().read().eich0en());
        let status = erm.status();
        assert_eq!(status.sram, Some(SramEvents::Corrected));
        assert!(!status.flash_uncorrectable);
        assert_eq!(erm.corrected_count(), index as u8 + 1);
        match erm.snapshot(Events::ALL).unwrap() {
            Report::Sram { events, address, .. } => {
                assert_eq!(events, SramEvents::Corrected);
                assert_eq!(address, scratch as u32 + index as u32 * 4);
            }
            _ => panic!("expected a corrected SRAM report"),
        }
        assert_eq!(unsafe { read_volatile(scratch.add(index)) }, expected);
        assert_eq!(erm.corrected_count(), index as u8 + 1);
        erm.acknowledge(Events::SRAM_CORRECTED);
        assert!(erm.snapshot(Events::ALL).is_none());
    }
    erm.reset_corrected_count();
    assert_eq!(erm.corrected_count(), 0);
    defmt::info!("ERM/EIM single-bit diagnostics OK");
    cortex_m::asm::bkpt();
}

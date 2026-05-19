//! Common code for all examples
//!
//! ## Interrupt Map
//!
//! | Interrupt ID | Description                  |
//! |--------------|------------------------------|
//! | `EXTPPI0[0]` | UART 0 Receive Interrupt    |
//! | `EXTPPI0[1]` | UART 0 Transmit Interrupt   |
//! | `EXTPPI0[2]` | UART 0 Combined Interrupt   |
//! | `EXTPPI0[3]` | UART 0 Overflow             |
//! | `EXTPPI1[0]` | UART 1 Receive Interrupt    |
//! | `EXTPPI1[1]` | UART 1 Transmit Interrupt   |
//! | `EXTPPI1[2]` | UART 1 Combined Interrupt   |
//! | `EXTPPI1[3]` | UART 1 Overflow             |
//! | `SP[0]`      | WDG                         |
//! | `SP[1]`      | DualTimer 1                 |
//! | `SP[2]`      | DualTimer 2                 |
//! | `SP[3]`      | DualTimer Combined          |
//! | `SP[4]`      | RTC                         |
//! | `SP[5]`      | UART 2 Receive Interrupt    |
//! | `SP[6]`      | UART 2 Transmit Interrupt   |
//! | `SP[7]`      | UART 3 Receive Interrupt    |
//! | `SP[8]`      | UART 3 Transmit Interrupt   |
//! | `SP[9]`      | UART 4 Receive Interrupt    |
//! | `SP[10]`     | UART 4 Transmit Interrupt   |
//! | `SP[11]`     | UART 5 Receive Interrupt    |
//! | `SP[12]`     | UART 5 Transmit Interrupt   |
//! | `SP[13]`     | UART 2 Combined Interrupt   |
//! | `SP[14]`     | UART 3 Combined Interrupt   |
//! | `SP[15]`     | UART 4 Combined Interrupt   |
//! | `SP[16]`     | UART 5 Combined Interrupt   |
//! | `SP[17]`     | UART Overflow (2, 3, 4 & 5) |
//! | `SP[18]`     | Ethernet                    |
//! | `SP[19]`     | USB                         |
//! | `SP[20]`     | FPGA Audio I2S              |
//! | `SP[21]`     | Touch Screen                |
//! | `SP[22]`     | SPI ADC                     |
//! | `SP[23]`     | SPI Shield 0                |
//! | `SP[24]`     | SPI Shield 1                |
//! | `SP[25]`     | HDCLCD Interrupt            |
//! | `SP[26]`     | GPIO 0 Combined Interrupt   |
//! | `SP[27]`     | GPIO 1 Combined Interrupt   |
//! | `SP[28]`     | GPIO 2 Combined Interrupt   |
//! | `SP[29]`     | GPIO 3 Combined Interrupt   |
//! | `SP[30..=45]`| GPIO 0.x Interrupt          |
//! | `SP[46..=61]`| GPIO 1.x Interrupt          |
//! | `SP[62..=77]`| GPIO 2.x Interrupt          |
//! | `SP[78..=93]`| GPIO 3.x Interrupt          |
//!
//! * Interrupt ID `SP[x]` are shared across cores
//! * Interrupt ID `EXTPPI0[x]` is only available on Core 0
//! * Interrupt ID `EXTPPI1[x]` is only available on Core 1

#![no_std]

use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::Waker;

use aarch32_cpu::generic_timer::{self, GenericTimer as _};
use arm_gic::{IntId, gicv3};
use defmt_semihosting as _;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

/// The PPI for the virtual timer, according to the Cortex-R52 Technical Reference Manual,
/// Table 10-3: PPI assignments.
///
/// This corresponds to Interrupt ID 27.
pub const VIRTUAL_TIMER_PPI: IntId = IntId::ppi(11);

/// Called when the application raises an unrecoverable `panic!`.
///
/// Prints the panic to the console and then exits QEMU using a semihosting
/// breakpoint.
#[panic_handler]
#[cfg(target_os = "none")]
fn panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("PANIC: {}", info);
    semihosting::process::exit(1);
}

/// Represents all the hardware we support in our MPS3-AN536 system
pub struct Board {
    /// The Arm Generic Interrupt Controller (v3)
    pub gic: gicv3::GicV3<'static>,
}

impl Board {
    /// Create a new board structure.
    ///
    /// Returns `Some(board)` the first time you call it, and None thereafter,
    /// so you cannot have two copies of the [`Board`] structure.
    pub fn new() -> Option<Board> {
        static TAKEN: AtomicBool = AtomicBool::new(false);
        defmt::info!("Board::new()...");
        if TAKEN.swap(true, Ordering::SeqCst) {
            // they already took the peripherals
            return None;
        }

        defmt::info!("Configure GIC...");
        // SAFETY: This is the first and only call to `make_gic()` as guaranteed by
        // the atomic flag check above, ensuring no aliasing of GIC register access.
        let mut gic = unsafe { make_gic() };

        defmt::info!("Configure virtual timer interrupts on core 0...");
        gic.set_interrupt_priority(VIRTUAL_TIMER_PPI, Some(0), 0x31).unwrap();
        gic.set_group(VIRTUAL_TIMER_PPI, Some(0), gicv3::Group::Group1NS)
            .unwrap();
        gic.enable_interrupt(VIRTUAL_TIMER_PPI, Some(0), true).unwrap();

        defmt::info!("Configure virtual timer interrupts on core 1...");
        gic.set_interrupt_priority(VIRTUAL_TIMER_PPI, Some(1), 0x31).unwrap();
        gic.set_group(VIRTUAL_TIMER_PPI, Some(1), gicv3::Group::Group1NS)
            .unwrap();
        gic.enable_interrupt(VIRTUAL_TIMER_PPI, Some(1), true).unwrap();

        // our initialised peripherals
        Some(Board { gic })
    }
}

/// Create the ARM GIC driver
///
/// # Safety
///
/// Only call this function once.
unsafe fn make_gic() -> gicv3::GicV3<'static> {
    defmt::info!("Making a gic...");
    /// Offset from PERIPHBASE for GIC Distributor
    const GICD_BASE_OFFSET: usize = 0x0000_0000usize;

    /// Offset from PERIPHBASE for the first GIC Redistributor
    const GICR_BASE_OFFSET: usize = 0x0010_0000usize;

    // Get the GIC address by reading CBAR
    let periphbase = aarch32_cpu::register::ImpCbar::read().periphbase();
    defmt::info!("Found PERIPHBASE 0x{=usize:08x}", periphbase as usize);
    let gicd_base = periphbase.wrapping_byte_add(GICD_BASE_OFFSET);
    let gicr_base = periphbase.wrapping_byte_add(GICR_BASE_OFFSET);

    // Initialise the GIC.
    // SAFETY: `gicd_base` points to the valid GICD MMIO region as obtained from the
    // hardware CBAR register. This pointer is used exclusively by this GIC instance.
    let gicd = unsafe { arm_gic::UniqueMmioPointer::new(core::ptr::NonNull::new(gicd_base.cast()).unwrap()) };
    let gicr_base = core::ptr::NonNull::new(gicr_base.cast()).unwrap();
    // SAFETY: The GICD and GICR base addresses point to valid GICv3 MMIO regions as
    // obtained from the hardware CBAR register. This function is only called once
    // (via Board::new()'s atomic guard), ensuring exclusive ownership of the GIC.
    let mut gic = unsafe { gicv3::GicV3::new(gicd, gicr_base, 2, false) };
    defmt::info!("Calling gic.setup(0)");
    gic.setup(0);
    defmt::info!("Calling gic.init_cpu(1)");
    gic.init_cpu(1);
    defmt::info!("Made a gic...");

    gic
}

/// A type for handling a queue of alarms on the EL1 Virtual Timer
struct Aarch32VirtualTimerQueue {
    core0: Mutex<CriticalSectionRawMutex, RefCell<Aarch32VirtualTimerQueueInner>>,
    core1: Mutex<CriticalSectionRawMutex, RefCell<Aarch32VirtualTimerQueueInner>>,
}

impl embassy_time_driver::Driver for Aarch32VirtualTimerQueue {
    fn now(&self) -> u64 {
        generic_timer::read_virtual_timer()
    }

    fn schedule_wake(&self, at: u64, waker: &Waker) {
        let cpuid = cpuid();
        defmt::trace!(
            "Scheduling wake at {=u64} ({=u64:tus}) on {=u8}",
            at,
            (at * 2) / 125,
            cpuid
        );
        if cpuid == 0 {
            critical_section::with(|cs| {
                let mut inner = self.core0.borrow(cs).borrow_mut();
                inner.schedule_wake(at, waker);
            });
        } else {
            critical_section::with(|cs| {
                let mut inner = self.core1.borrow(cs).borrow_mut();
                inner.schedule_wake(at, waker);
            });
        }
    }
}

impl Aarch32VirtualTimerQueue {
    /// Call this from the interrupt handler when it goes off
    fn on_irq(&self) {
        defmt::trace!("Alarm went off on {=u8}", cpuid());
        if cpuid() == 0 {
            critical_section::with(|cs| {
                let mut inner = self.core0.borrow(cs).borrow_mut();
                inner.update_alarm();
            });
        } else {
            critical_section::with(|cs| {
                let mut inner = self.core1.borrow(cs).borrow_mut();
                inner.update_alarm();
            });
        }
    }
}

/// Call this from the interrupt handler when VIRTUAL_TIMER_PPI fires
pub fn timer_irq() {
    DRIVER.on_irq();
}

/// Mutable state for our alarm queue
struct Aarch32VirtualTimerQueueInner {
    queue: embassy_time_queue_utils::Queue,
}

impl Aarch32VirtualTimerQueueInner {
    /// Schedule a wake-up for the next thing in the queue
    fn schedule_wake(&mut self, at: u64, waker: &Waker) {
        if self.queue.schedule_wake(at, waker) {
            // alarm needs updating
            self.update_alarm();
        }
    }

    /// Check the time, and the queue, and maybe set an alarm (or turn it off)
    fn update_alarm(&mut self) {
        let now = generic_timer::read_virtual_timer();
        let next = self.queue.next_expiration(now);

        // SAFETY: we have &mut on this timer driver, and it's the only thing that owns
        // a timer.
        let mut vt = unsafe { generic_timer::El1VirtualTimer::new() };
        if next == u64::MAX {
            // turn the timer interrupt off
            vt.interrupt_mask(true);
        } else {
            // set an alarm - will fire instantly if it's in the past
            vt.counter_compare_set(next);
            vt.interrupt_mask(false);
            vt.enable(true);
        }
    }
}

embassy_time_driver::time_driver_impl!(static DRIVER: Aarch32VirtualTimerQueue = Aarch32VirtualTimerQueue {
    core0: Mutex::new(RefCell::new(
        Aarch32VirtualTimerQueueInner {
            queue: embassy_time_queue_utils::Queue::new(),
        }
    )),
    core1: Mutex::new(RefCell::new(
        Aarch32VirtualTimerQueueInner {
            queue: embassy_time_queue_utils::Queue::new(),
        }
    ))
});

/// Default Core 1 entry point
#[unsafe(no_mangle)]
pub extern "C" fn default_kmain2() -> ! {
    loop {
        aarch32_cpu::asm::wfi();
    }
}

/// Release core1 from spin loop
pub fn start_core1() {
    let fpga_led = 0xE020_2000 as *mut u32;
    unsafe {
        // Activate second core by writing to FPGA LEDs.
        // We needed a shared register that wasn't in RAM, and this will do.
        fpga_led.write_volatile(1);
    }
}

/// Start-up code for multi-core Armv8-R, as implemented on the MPS3-AN536.
///
/// We boot into EL2, set up a stack pointer, init .data on .bss on core0, and
/// run `kmain` in EL1 on all cores.
///
/// # Safety
///
/// This function should not be called manually. It should only be called on reset
/// from the reset vector.
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.startup")]
pub unsafe extern "C" fn _start() {
    core::arch::naked_asm!(
        r#"
        // Read MPIDR into R0
        mrc     p15, 0, r0, c0, c0, 5
        ands    r0, r0, 0xFF
        bne     core1
    core0:
        ldr     pc, =_default_start
    core1:
        // LED GPIO register base address
        ldr     r0, =0xE0202000
        mov     r1, #0
    core1_spin:
        wfe
        // spin until an LED0 is on. We use the LED because unlike RAM this register resets to a known value.
        ldr     r2, [r0]  
        cmp     r1, r2
        beq     core1_spin
    core1_released:
        // now an LED is on, we assume _core1_stack_pointer contains our stack pointer
        // First we must exit EL2...
        // Set the HVBAR (for EL2) to _vector_table
        ldr     r0, =_vector_table
        mcr     p15, 4, r0, c12, c0, 0
        // Configure HACTLR to let us enter EL1
        mrc     p15, 4, r0, c1, c0, 1
        mov     r1, {hactlr_bits}
        orr     r0, r0, r1
        mcr     p15, 4, r0, c1, c0, 1
        // Program the SPSR - enter system mode (0x1F) in Arm mode with IRQ, FIQ masked
        mov		r0, {sys_mode}
        msr		spsr_hyp, r0
        adr		r0, 1f
        msr		elr_hyp, r0
        dsb
        isb
        eret
    1:
        // Allow VFP coprocessor access
        mrc     p15, 0, r0, c1, c0, 2
        orr     r0, r0, #0xF00000
        mcr     p15, 0, r0, c1, c0, 2
        // Enable VFP
        mov     r0, #0x40000000
        vmsr    fpexc, r0
        // Set the VBAR (for EL1) to _vector_table. NB: This isn't required on
        // Armv7-R because that only supports 'low' (default) or 'high'.
        ldr     r0, =_vector_table
        mcr     p15, 0, r0, c12, c0, 0
        // set up our stacks - also switches to SYS mode
        movs    r0, #1
        bl      _stack_setup_preallocated
        // Zero all registers before calling kmain2
        mov     r0, 0
        mov     r1, 0
        mov     r2, 0
        mov     r3, 0
        mov     r4, 0
        mov     r5, 0
        mov     r6, 0
        mov     r7, 0
        mov     r8, 0
        mov     r9, 0
        mov     r10, 0
        mov     r11, 0
        mov     r12, 0
        // call our kmain2 for core 1
        bl      kmain2
    "#,
        hactlr_bits = const {
            aarch32_cpu::register::Hactlr::new_with_raw_value(0)
                .with_cpuactlr(true)
                .with_cdbgdci(true)
                .with_flashifregionr(true)
                .with_periphpregionr(true)
                .with_qosr(true)
                .with_bustimeoutr(true)
                .with_intmonr(true)
                .with_err(true)
                .with_testr1(true)
                .raw_value()
        },
        sys_mode = const {
            aarch32_cpu::register::Cpsr::new_with_raw_value(0)
                .with_mode(aarch32_cpu::register::cpsr::ProcessorMode::Sys)
                .with_i(true)
                .with_f(true)
                .raw_value()
        },
    )
}

/// Get the Multi-Processor ID lowest byte (either 0 or 1 on this platform)
pub fn cpuid() -> u8 {
    aarch32_cpu::register::Mpidr::read().0 as u8
}

// defmt logs get a timestamp and a core ID
//
// this reads from the local timer for that core, but both cores have the same time because
// qemu took them out of reset at the same time.
defmt::timestamp! {"{=u64:us} (Core{=u8})", embassy_time::Instant::now().as_micros(), cpuid() }

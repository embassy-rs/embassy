//! Register definitions for the CMSDK Timer

/// Register block of the CMSDK Timer.
#[derive(derive_mmio::Mmio)]
#[repr(C)]
#[derive(Debug)]
pub struct Registers {
    /// Control register.
    control: Control,
    value: u32,
    reload: u32,
    interrupt: Interrupt,
    _reserved: [u32; 0x3F0],
    peripheral_id_4: u32,
    peripheral_id_5: u32,
    peripheral_id_6: u32,
    peripheral_id_7: u32,
    peripheral_id_0: u32,
    peripheral_id_1: u32,
    peripheral_id_2: u32,
    peripheral_id_3: u32,
    component_id_0: u32,
    component_id_1: u32,
    component_id_2: u32,
    component_id_3: u32,
}

/// Control register.
#[bitbybit::bitfield(u32, debug, default = 0x0, forbid_overlaps, defmt_bitfields(feature = "defmt"))]
pub struct Control {
    /// Interrupt enable bit.
    #[bit(3, rw)]
    interrupt_enable: bool,
    /// Use external input as clock.
    #[bit(2, rw)]
    external_input_as_clock: bool,
    /// Use external input as enable bit.
    #[bit(1, rw)]
    external_input_as_enable: bool,
    /// Enable the timer.
    #[bit(0, rw)]
    enable: bool,
}

/// Interrupt register.
#[bitbybit::bitfield(u32, debug, default = 0x0, forbid_overlaps, defmt_bitfields(feature = "defmt"))]
pub struct Interrupt {
    /// Write 1 to clear.
    #[bit(0, rw)]
    interrupt_bit: bool,
}

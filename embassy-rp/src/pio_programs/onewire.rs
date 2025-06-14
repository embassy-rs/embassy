//! OneWire pio driver

use crate::clocks::clk_sys_freq;
use crate::gpio::Level;
use crate::pio::{
    Common, Config, Direction, Instance, LoadedProgram, PioPin, ShiftConfig, ShiftDirection, StateMachine,
};
use crate::Peri;

/// This struct represents a onewire driver program
pub struct PioOneWireProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
    reset_addr: u8,
    next_bit_addr: u8,
}

impl<'a, PIO: Instance> PioOneWireProgram<'a, PIO> {
    /// Load the program into the given pio
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        let prg = pio::pio_asm!(
            r#"
                ; We need to use the pins direction to simulate open drain output
                ; This results in all the side-set values being swapped from the actual pin value
                .side_set 1 pindirs

                ; Set the origin to 0 so we can correctly use jmp instructions externally
                .origin 0

                ; Tick rate is 1 tick per 6us, so all delays should be calculated back to that
                ; All the instructions have a calculated delay XX in us as [(XX / CLK) - 1].
                ; The - 1 is for the instruction which also takes one clock cyle.
                ; The delay can be 0 which will result in just 6us for the instruction itself
                .define CLK 6

                ; Write the reset block after trigger
                public reset:
                        set x, 4                side 0 [(60 / CLK) - 1]     ; idle before reset
                    reset_inner:                                            ; Repeat the following 5 times, so 5*96us = 480us in total
                        nop                     side 1 [(90 / CLK) - 1]
                        jmp x--, reset_inner    side 1 [( 6 / CLK) - 1]
                        ; Fallthrough

                    ; Check for presence of one or more devices.
                    ; This samples 32 times with an interval of 12us after a 18us delay.
                    ; If any bit is zero in the end value, there is a detection
                    ; This whole function takes 480us
                        set x, 31               side 0 [(24 / CLK) - 1]     ; Loop 32 times -> 32*12us = 384us
                    presence_check:
                        in pins, 1              side 0 [( 6 / CLK) - 1]     ; poll pin and push to isr
                        jmp x--, presence_check side 0 [( 6 / CLK) - 1]
                        jmp next_bit            side 0 [(72 / CLK) - 1]

                    ; The low pulse was already done, we only need to delay and poll the bit in case we are reading
                    write_1:
                        nop                     side 0 [( 6 / CLK) - 1]     ; Delay before sampling the input pin
                        in pins, 1              side 0 [(48 / CLK) - 1]     ; This writes the state of the pin into the ISR
                        ; Fallthrough

                ; This is the entry point when reading and writing data
                public next_bit:
                    .wrap_target
                        out x, 1                side 0 [(12 / CLK) - 1]     ; Stalls if no data available in TX FIFO and OSR
                        jmp x--, write_1        side 1 [( 6 / CLK) - 1]     ; Do the always low part of a bit, jump to write_1 if we want to write a 1 bit
                        in null, 1              side 1 [(54 / CLK) - 1]     ; Do the remainder of the low part of a 0 bit
                                                                            ; This writes 0 into the ISR so that the shift count stays in sync
                    .wrap
            "#
        );

        Self {
            prg: common.load_program(&prg.program),
            reset_addr: prg.public_defines.reset as u8,
            next_bit_addr: prg.public_defines.next_bit as u8,
        }
    }
}
/// Pio backed OneWire driver
pub struct PioOneWire<'d, PIO: Instance, const SM: usize> {
    sm: StateMachine<'d, PIO, SM>,
    cfg: Config<'d, PIO>,
    reset_addr: u8,
    next_bit_addr: u8,
}

impl<'d, PIO: Instance, const SM: usize> PioOneWire<'d, PIO, SM> {
    /// Create a new instance the driver
    pub fn new(
        common: &mut Common<'d, PIO>,
        mut sm: StateMachine<'d, PIO, SM>,
        pin: Peri<'d, impl PioPin>,
        program: &PioOneWireProgram<'d, PIO>,
    ) -> Self {
        let pin = common.make_pio_pin(pin);

        sm.set_pin_dirs(Direction::In, &[&pin]);
        sm.set_pins(Level::Low, &[&pin]);

        let mut cfg = Config::default();
        cfg.use_program(&program.prg, &[&pin]);
        cfg.set_in_pins(&[&pin]);

        let shift_cfg = ShiftConfig {
            auto_fill: true,
            direction: ShiftDirection::Right,
            threshold: 8,
        };
        cfg.shift_in = shift_cfg;
        cfg.shift_out = shift_cfg;

        let divider = (clk_sys_freq() / 1000000) as u16 * 6;
        cfg.clock_divider = divider.into();

        sm.set_config(&cfg);
        sm.clear_fifos();
        sm.restart();
        unsafe {
            sm.exec_jmp(program.next_bit_addr);
        }
        sm.set_enable(true);

        Self {
            sm,
            cfg,
            reset_addr: program.reset_addr,
            next_bit_addr: program.next_bit_addr,
        }
    }

    /// Perform an initialization sequence, will return true if a presence pulse was detected from a device
    pub async fn reset(&mut self) -> bool {
        // The state machine immediately starts running when jumping to this address
        unsafe {
            self.sm.exec_jmp(self.reset_addr);
        }

        let rx = self.sm.rx();
        let mut found = false;
        for _ in 0..4 {
            if rx.wait_pull().await != 0 {
                found = true;
            }
        }

        found
    }

    /// Write bytes to the onewire bus
    pub async fn write_bytes(&mut self, data: &[u8]) {
        let (rx, tx) = self.sm.rx_tx();
        for b in data {
            tx.wait_push(*b as u32).await;

            // Empty the buffer that is being filled with every write
            let _ = rx.wait_pull().await;
        }
    }

    /// Read bytes from the onewire bus
    pub async fn read_bytes(&mut self, data: &mut [u8]) {
        let (rx, tx) = self.sm.rx_tx();
        for b in data {
            // Write all 1's so that we can read what the device responds
            tx.wait_push(0xff).await;

            *b = (rx.wait_pull().await >> 24) as u8;
        }
    }

    async fn search(&mut self, state: &mut PioOneWireSearch) -> Option<u64> {
        if !self.reset().await {
            // No device present, no use in searching
            state.finished = true;
            return None;
        }
        self.write_bytes(&[0xF0]).await; // 0xF0 is the search rom command

        self.prepare_search();

        let (rx, tx) = self.sm.rx_tx();

        let mut value = 0;
        let mut last_zero = 0;

        for bit in 0..64 {
            // Write 2 dummy bits to read a bit and its complement
            tx.wait_push(0x1).await;
            tx.wait_push(0x1).await;
            let in1 = rx.wait_pull().await;
            let in2 = rx.wait_pull().await;
            let push = match (in1, in2) {
                (0, 0) => {
                    // If both are 0, it means we have devices with 0 and 1 bits in this position
                    let write_value = if bit < state.last_discrepancy {
                        (state.last_rom & (1 << bit)) != 0
                    } else {
                        bit == state.last_discrepancy
                    };

                    if write_value {
                        1
                    } else {
                        last_zero = bit;
                        0
                    }
                }
                (0, 1) => 0, // Only devices with a 0 bit in this position
                (1, 0) => 1, // Only devices with a 1 bit in this position
                _ => {
                    // If both are 1, it means there is no device active and there is no point in continuing
                    self.restore_after_search();
                    state.finished = true;
                    return None;
                }
            };
            value >>= 1;
            if push == 1 {
                value |= 1 << 63;
            }
            tx.wait_push(push).await;
            let _ = rx.wait_pull().await; // Discard the result of the write action
        }

        self.restore_after_search();

        state.last_discrepancy = last_zero;
        state.finished = last_zero == 0;
        state.last_rom = value;
        Some(value)
    }

    fn prepare_search(&mut self) {
        self.cfg.shift_in.threshold = 1;
        self.cfg.shift_in.direction = ShiftDirection::Left;
        self.cfg.shift_out.threshold = 1;

        self.sm.set_enable(false);
        self.sm.set_config(&self.cfg);

        // set_config jumps to the wrong address so jump to the right one here
        unsafe {
            self.sm.exec_jmp(self.next_bit_addr);
        }
        self.sm.set_enable(true);
    }

    fn restore_after_search(&mut self) {
        self.cfg.shift_in.threshold = 8;
        self.cfg.shift_in.direction = ShiftDirection::Right;
        self.cfg.shift_out.threshold = 8;

        self.sm.set_enable(false);
        self.sm.set_config(&self.cfg);

        // Clear the state in case we aborted prematurely with some bits still in the shift registers
        self.sm.clear_fifos();
        self.sm.restart();

        // set_config jumps to the wrong address so jump to the right one here
        unsafe {
            self.sm.exec_jmp(self.next_bit_addr);
        }
        self.sm.set_enable(true);
    }
}

/// Onewire search state
pub struct PioOneWireSearch {
    last_rom: u64,
    last_discrepancy: u8,
    finished: bool,
}

impl PioOneWireSearch {
    /// Create a new Onewire search state
    pub fn new() -> Self {
        Self {
            last_rom: 0,
            last_discrepancy: 0,
            finished: false,
        }
    }

    /// Search for the next address on the bus
    pub async fn next<PIO: Instance, const SM: usize>(&mut self, pio: &mut PioOneWire<'_, PIO, SM>) -> Option<u64> {
        if self.finished {
            None
        } else {
            pio.search(self).await
        }
    }

    /// Is finished when all devices have been found
    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

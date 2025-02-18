//! OneWire pio driver

use crate::pio::{Common, Config, Instance, LoadedProgram, PioPin, ShiftConfig, ShiftDirection, StateMachine};

/// This struct represents an onewire driver program
pub struct PioOneWireProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> PioOneWireProgram<'a, PIO> {
    /// Load the program into the given pio
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        let prg = pio::pio_asm!(
            r#"
                .wrap_target
                    again:
                        pull block
                        mov x, osr
                        jmp !x, read
                        write:
                            set pindirs, 1 
                            set pins, 0  
                            loop1: 
                                jmp x--,loop1
                            set pindirs, 0 [31]
                            wait 1 pin 0 [31]
                            pull block
                            mov x, osr
                            bytes1:
                                pull block
                                set y, 7    
                                set pindirs, 1 
                                bit1:
                                    set pins, 0 [1]
                                    out pins,1 [31]
                                    set pins, 1 [20]
                                    jmp y--,bit1
                                jmp x--,bytes1
                            set pindirs, 0 [31]
                            jmp again
                        read:
                            pull block
                            mov x, osr
                            bytes2:
                                set y, 7
                                bit2:
                                    set pindirs, 1 
                                    set pins, 0 [1]  
                                    set pindirs, 0 [5]
                                    in pins,1 [10]   
                                    jmp y--,bit2
                            jmp x--,bytes2
                .wrap
            "#,
        );
        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}

/// Pio backed OneWire driver
pub struct PioOneWire<'d, PIO: Instance, const SM: usize> {
    sm: StateMachine<'d, PIO, SM>,
}

impl<'d, PIO: Instance, const SM: usize> PioOneWire<'d, PIO, SM> {
    /// Create a new instance the driver
    pub fn new(
        common: &mut Common<'d, PIO>,
        mut sm: StateMachine<'d, PIO, SM>,
        pin: impl PioPin,
        program: &PioOneWireProgram<'d, PIO>,
    ) -> Self {
        let pin = common.make_pio_pin(pin);
        let mut cfg = Config::default();
        cfg.use_program(&program.prg, &[]);
        cfg.set_out_pins(&[&pin]);
        cfg.set_in_pins(&[&pin]);
        cfg.set_set_pins(&[&pin]);
        cfg.shift_in = ShiftConfig {
            auto_fill: true,
            direction: ShiftDirection::Right,
            threshold: 8,
        };
        cfg.clock_divider = 255_u8.into();
        sm.set_config(&cfg);
        sm.set_enable(true);
        Self { sm }
    }

    /// Write bytes over the wire
    pub async fn write_bytes(&mut self, bytes: &[u8]) {
        self.sm.tx().wait_push(250).await;
        self.sm.tx().wait_push(bytes.len() as u32 - 1).await;
        for b in bytes {
            self.sm.tx().wait_push(*b as u32).await;
        }
    }

    /// Read bytes from the wire
    pub async fn read_bytes(&mut self, bytes: &mut [u8]) {
        self.sm.tx().wait_push(0).await;
        self.sm.tx().wait_push(bytes.len() as u32 - 1).await;
        for b in bytes.iter_mut() {
            *b = (self.sm.rx().wait_pull().await >> 24) as u8;
        }
    }
}

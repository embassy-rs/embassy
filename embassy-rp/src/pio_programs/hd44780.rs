//! [HD44780 display driver](https://www.sparkfun.com/datasheets/LCD/HD44780.pdf)

use crate::dma::{AnyChannel, Channel};
use crate::pio::{
    Common, Config, Direction, FifoJoin, Instance, Irq, LoadedProgram, PioPin, ShiftConfig, ShiftDirection,
    StateMachine,
};
use crate::Peri;

/// This struct represents a HD44780 program that takes command words (<wait:24> <command:4> <0:4>)
pub struct PioHD44780CommandWordProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> PioHD44780CommandWordProgram<'a, PIO> {
    /// Load the program into the given pio
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        let prg = pio::pio_asm!(
            r#"
                .side_set 1 opt
                .origin 20

                loop:
                    out x,     24
                delay:
                    jmp x--,   delay
                    out pins,  4     side 1
                    out null,  4     side 0
                    jmp !osre, loop
                irq 0
            "#,
        );

        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}

/// This struct represents a HD44780 program that takes command sequences (<rs:1> <count:7>, data...)
pub struct PioHD44780CommandSequenceProgram<'a, PIO: Instance> {
    prg: LoadedProgram<'a, PIO>,
}

impl<'a, PIO: Instance> PioHD44780CommandSequenceProgram<'a, PIO> {
    /// Load the program into the given pio
    pub fn new(common: &mut Common<'a, PIO>) -> Self {
        // many side sets are only there to free up a delay bit!
        let prg = pio::pio_asm!(
            r#"
                .origin 27
                .side_set 1

                .wrap_target
                pull     side 0
                out  x 1 side 0 ; !rs
                out  y 7 side 0 ; #data - 1

                ; rs/rw to e: >= 60ns
                ; e high time: >= 500ns
                ; e low time: >= 500ns
                ; read data valid after e falling: ~5ns
                ; write data hold after e falling: ~10ns

                loop:
                    pull                 side 0
                    jmp  !x       data   side 0
                command:
                    set  pins     0b00   side 0
                    jmp  shift           side 0
                data:
                    set  pins     0b01   side 0
                shift:
                    out  pins     4      side 1 [9]
                    nop                  side 0 [9]
                    out  pins     4      side 1 [9]
                    mov  osr      null   side 0 [7]
                    out  pindirs  4      side 0
                    set  pins     0b10   side 0
                busy:
                    nop                  side 1 [9]
                    jmp  pin      more   side 0 [9]
                    mov  osr      ~osr   side 1 [9]
                    nop                  side 0 [4]
                    out  pindirs  4      side 0
                    jmp  y--      loop   side 0
                .wrap
                more:
                    nop                  side 1 [9]
                    jmp busy             side 0 [9]
            "#
        );

        let prg = common.load_program(&prg.program);

        Self { prg }
    }
}

/// Pio backed HD44780 driver
pub struct PioHD44780<'l, P: Instance, const S: usize> {
    dma: Peri<'l, AnyChannel>,
    sm: StateMachine<'l, P, S>,

    buf: [u8; 40],
}

impl<'l, P: Instance, const S: usize> PioHD44780<'l, P, S> {
    /// Configure the given state machine to first init, then write data to, a HD44780 display.
    pub async fn new(
        common: &mut Common<'l, P>,
        mut sm: StateMachine<'l, P, S>,
        mut irq: Irq<'l, P, S>,
        mut dma: Peri<'l, impl Channel>,
        rs: Peri<'l, impl PioPin>,
        rw: Peri<'l, impl PioPin>,
        e: Peri<'l, impl PioPin>,
        db4: Peri<'l, impl PioPin>,
        db5: Peri<'l, impl PioPin>,
        db6: Peri<'l, impl PioPin>,
        db7: Peri<'l, impl PioPin>,
        word_prg: &PioHD44780CommandWordProgram<'l, P>,
        seq_prg: &PioHD44780CommandSequenceProgram<'l, P>,
    ) -> PioHD44780<'l, P, S> {
        let rs = common.make_pio_pin(rs);
        let rw = common.make_pio_pin(rw);
        let e = common.make_pio_pin(e);
        let db4 = common.make_pio_pin(db4);
        let db5 = common.make_pio_pin(db5);
        let db6 = common.make_pio_pin(db6);
        let db7 = common.make_pio_pin(db7);

        sm.set_pin_dirs(Direction::Out, &[&rs, &rw, &e, &db4, &db5, &db6, &db7]);

        let mut cfg = Config::default();
        cfg.use_program(&word_prg.prg, &[&e]);
        cfg.clock_divider = 125u8.into();
        cfg.set_out_pins(&[&db4, &db5, &db6, &db7]);
        cfg.shift_out = ShiftConfig {
            auto_fill: true,
            direction: ShiftDirection::Left,
            threshold: 32,
        };
        cfg.fifo_join = FifoJoin::TxOnly;
        sm.set_config(&cfg);

        sm.set_enable(true);
        // init to 8 bit thrice
        sm.tx().push((50000 << 8) | 0x30);
        sm.tx().push((5000 << 8) | 0x30);
        sm.tx().push((200 << 8) | 0x30);
        // init 4 bit
        sm.tx().push((200 << 8) | 0x20);
        // set font and lines
        sm.tx().push((50 << 8) | 0x20);
        sm.tx().push(0b1100_0000);

        irq.wait().await;
        sm.set_enable(false);

        let mut cfg = Config::default();
        cfg.use_program(&seq_prg.prg, &[&e]);
        cfg.clock_divider = 8u8.into(); // ~64ns/insn
        cfg.set_jmp_pin(&db7);
        cfg.set_set_pins(&[&rs, &rw]);
        cfg.set_out_pins(&[&db4, &db5, &db6, &db7]);
        cfg.shift_out.direction = ShiftDirection::Left;
        cfg.fifo_join = FifoJoin::TxOnly;
        sm.set_config(&cfg);

        sm.set_enable(true);

        // display on and cursor on and blinking, reset display
        sm.tx().dma_push(dma.reborrow(), &[0x81u8, 0x0f, 1], false).await;

        Self {
            dma: dma.into(),
            sm,
            buf: [0x20; 40],
        }
    }

    /// Write a line to the display
    pub async fn add_line(&mut self, s: &[u8]) {
        // move cursor to 0:0, prepare 16 characters
        self.buf[..3].copy_from_slice(&[0x80, 0x80, 15]);
        // move line 2 up
        self.buf.copy_within(22..38, 3);
        // move cursor to 1:0, prepare 16 characters
        self.buf[19..22].copy_from_slice(&[0x80, 0xc0, 15]);
        // file line 2 with spaces
        self.buf[22..38].fill(0x20);
        // copy input line
        let len = s.len().min(16);
        self.buf[22..22 + len].copy_from_slice(&s[0..len]);
        // set cursor to 1:15
        self.buf[38..].copy_from_slice(&[0x80, 0xcf]);

        self.sm.tx().dma_push(self.dma.reborrow(), &self.buf, false).await;
    }
}

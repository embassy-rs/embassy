//! This example shows how you can use PIO to read a `DS18B20` one-wire temperature sensor.

#![no_std]
#![no_main]
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{self, Common, Config, InterruptHandler, Pio, PioPin, ShiftConfig, ShiftDirection, StateMachine};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut pio = Pio::new(p.PIO0, Irqs);
    let mut sensor = Ds18b20::new(&mut pio.common, pio.sm0, p.PIN_2);

    loop {
        sensor.start().await; // Start a new measurement
        Timer::after_secs(1).await; // Allow 1s for the measurement to finish
        match sensor.temperature().await {
            Ok(temp) => info!("temp = {:?} deg C", temp),
            _ => error!("sensor error"),
        }
        Timer::after_secs(1).await;
    }
}

/// DS18B20 temperature sensor driver
pub struct Ds18b20<'d, PIO: pio::Instance, const SM: usize> {
    sm: StateMachine<'d, PIO, SM>,
}

impl<'d, PIO: pio::Instance, const SM: usize> Ds18b20<'d, PIO, SM> {
    /// Create a new instance the driver
    pub fn new(common: &mut Common<'d, PIO>, mut sm: StateMachine<'d, PIO, SM>, pin: impl PioPin) -> Self {
        let prg = pio_proc::pio_asm!(
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

        let pin = common.make_pio_pin(pin);
        let mut cfg = Config::default();
        cfg.use_program(&common.load_program(&prg.program), &[]);
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
    async fn write_bytes(&mut self, bytes: &[u8]) {
        self.sm.tx().wait_push(250).await;
        self.sm.tx().wait_push(bytes.len() as u32 - 1).await;
        for b in bytes {
            self.sm.tx().wait_push(*b as u32).await;
        }
    }

    /// Read bytes from the wire
    async fn read_bytes(&mut self, bytes: &mut [u8]) {
        self.sm.tx().wait_push(0).await;
        self.sm.tx().wait_push(bytes.len() as u32 - 1).await;
        for b in bytes.iter_mut() {
            *b = (self.sm.rx().wait_pull().await >> 24) as u8;
        }
    }

    /// Calculate CRC8 of the data
    fn crc8(data: &[u8]) -> u8 {
        let mut temp;
        let mut data_byte;
        let mut crc = 0;
        for b in data {
            data_byte = *b;
            for _ in 0..8 {
                temp = (crc ^ data_byte) & 0x01;
                crc >>= 1;
                if temp != 0 {
                    crc ^= 0x8C;
                }
                data_byte >>= 1;
            }
        }
        crc
    }

    /// Start a new measurement. Allow at least 1000ms before getting `temperature`.
    pub async fn start(&mut self) {
        self.write_bytes(&[0xCC, 0x44]).await;
    }

    /// Read the temperature. Ensure >1000ms has passed since `start` before calling this.
    pub async fn temperature(&mut self) -> Result<f32, ()> {
        self.write_bytes(&[0xCC, 0xBE]).await;
        let mut data = [0; 9];
        self.read_bytes(&mut data).await;
        match Self::crc8(&data) == 0 {
            true => Ok(((data[1] as u32) << 8 | data[0] as u32) as f32 / 16.),
            false => Err(()),
        }
    }
}

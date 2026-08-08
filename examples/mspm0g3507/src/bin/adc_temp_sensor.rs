//! Example of reading the internal temperature sensor of the MSPM0G3507.
//!
//! This ports the TI C example `adc12_internal_temp_sensor_mathacl` to embassy. It reads
//! the factory calibration constant, samples the sensor and converts the result to degrees
//! Celsius and Fahrenheit, using the MATHACL accelerator for the divisions.

#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mspm0::adc::{self, Adc, Conversion, TempSensorChannel};
use embassy_mspm0::mathacl::{IQType, Mathacl};
use embassy_mspm0::{bind_interrupts, peripherals, read_temp_calibration_constant};
use embassy_time::Timer;
use panic_halt as _;

bind_interrupts!(struct Irqs {
    ADC0 => adc::InterruptHandler<peripherals::ADC0>;
});

// Temperature sensor trim parameters from the device datasheet ("Temperature Sensor").
const TEMP_TS_TRIM_C: f32 = 30.0;
// 1 / TSc, where TSc is the temperature sensor coefficient from the datasheet.
const TEMP_TS_COEF_MV_C: f32 = -555.55;
const ADC_VREF_VOLTAGE: f32 = 3.3;
const ADC_BIT_RESOLUTION: f32 = 4096.0; // 2^12

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Hello world!");

    let d = embassy_mspm0::init(Default::default());

    let mut adc = Adc::new_async(d.ADC0, Irqs, Default::default());
    let mut mathacl = Mathacl::new(d.MATHACL);
    let mut temp_sensor = TempSensorChannel;

    // 12-bit ADC code of the temp sensor at 30 C.
    let cal = read_temp_calibration_constant();

    // Vtrim = ADC_VREF * (TEMP_SENSE0 - 0.5) / 2^12, computed with MATHACL division.
    let vtrim = mathacl
        .div_iq(
            IQType::from_f32((cal as f32 - 0.5) * ADC_VREF_VOLTAGE, 15, true).unwrap(),
            IQType::from_f32(ADC_BIT_RESOLUTION, 15, true).unwrap(),
        )
        .unwrap();

    loop {
        let adc_result = adc.blocking_read(&mut temp_sensor, Conversion::default());

        // Vsample = ADC_VREF * (adcResult - 0.5) / 2^12, computed with MATHACL division.
        let vsample = mathacl
            .div_iq(
                IQType::from_f32((adc_result as f32 - 0.5) * ADC_VREF_VOLTAGE, 15, true).unwrap(),
                IQType::from_f32(ADC_BIT_RESOLUTION, 15, true).unwrap(),
            )
            .unwrap();

        // TSAMPLE = TEMP_TS_COEF_mV_C * (Vsample - Vtrim) + TEMP_TS_TRIM_C
        let temp_deg_c = TEMP_TS_COEF_MV_C * (vsample - vtrim) + TEMP_TS_TRIM_C;
        let temp_deg_f = temp_deg_c * 1.8 + 32.0;

        info!("Temperature: {} C, {} F", temp_deg_c, temp_deg_f);

        Timer::after_millis(500).await;
    }
}

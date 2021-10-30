#[derive(Debug, Clone, Copy)]
pub struct Test {
    pub package: &'static str,
    pub target: &'static str,
    pub features: &'static str,
}

pub const TESTS: &[Test] = &[
    Test {
        package: "embassy",
        target: "thumbv7em-none-eabi",
        features: "",
    },
    Test {
        package: "embassy",
        target: "thumbv7em-none-eabi",
        features: "log,executor-agnostic",
    },
    Test {
        package: "embassy",
        target: "thumbv7em-none-eabi",
        features: "defmt",
    },
    Test {
        package: "embassy",
        target: "thumbv6m-none-eabi",
        features: "defmt",
    },
    /*
    Test {
        package: "examples/std",
        target: "",
        features: "",
    },
     */
    Test {
        package: "embassy-nrf",
        target: "thumbv7em-none-eabi",
        features: "nrf52805",
    },
    Test {
        package: "embassy-nrf",
        target: "thumbv7em-none-eabi",
        features: "nrf52810",
    },
    Test {
        package: "embassy-nrf",
        target: "thumbv7em-none-eabi",
        features: "nrf52811",
    },
    Test {
        package: "embassy-nrf",
        target: "thumbv7em-none-eabi",
        features: "nrf52820",
    },
    Test {
        package: "embassy-nrf",
        target: "thumbv7em-none-eabi",
        features: "nrf52832",
    },
    Test {
        package: "embassy-nrf",
        target: "thumbv7em-none-eabi",
        features: "nrf52833",
    },
    Test {
        package: "embassy-nrf",
        target: "thumbv7em-none-eabi",
        features: "nrf52840",
    },
    Test {
        package: "embassy-nrf",
        target: "thumbv7em-none-eabi",
        features: "nrf52840,log",
    },
    Test {
        package: "embassy-nrf",
        target: "thumbv7em-none-eabi",
        features: "nrf52840,defmt",
    },
    Test {
        package: "embassy-nrf",
        target: "thumbv8m.main-none-eabihf",
        features: "nrf9160-s",
    },
    Test {
        package: "embassy-nrf",
        target: "thumbv8m.main-none-eabihf",
        features: "nrf9160-ns",
    },
    Test {
        package: "examples/nrf",
        target: "thumbv7em-none-eabi",
        features: "",
    },
    Test {
        package: "examples/rp",
        target: "thumbv6m-none-eabi",
        features: "",
    },
    Test {
        package: "embassy-stm32",
        target: "thumbv7em-none-eabi",
        features: "stm32f411ce,defmt",
    },
    Test {
        package: "embassy-stm32",
        target: "thumbv7em-none-eabi",
        features: "stm32f429zi,log",
    },
    Test {
        package: "embassy-stm32",
        target: "thumbv7em-none-eabi",
        features: "stm32h755zi_cm7,defmt",
    },
    Test {
        package: "embassy-stm32",
        target: "thumbv7em-none-eabi",
        features: "stm32l476vg,defmt",
    },
    Test {
        package: "embassy-stm32",
        target: "thumbv6m-none-eabi",
        features: "stm32l072cz,defmt",
    },
    Test {
        package: "embassy-stm32",
        target: "thumbv7m-none-eabi",
        features: "stm32l151cb-a,defmt",
    },
    Test {
        package: "examples/stm32f4",
        target: "thumbv7em-none-eabi",
        features: "",
    },
    Test {
        package: "examples/stm32l4",
        target: "thumbv7em-none-eabi",
        features: "",
    },
    Test {
        package: "examples/stm32h7",
        target: "thumbv7em-none-eabi",
        features: "",
    },
    Test {
        package: "examples/stm32l0",
        target: "thumbv6m-none-eabi",
        features: "",
    },
    Test {
        package: "examples/stm32l1",
        target: "thumbv7m-none-eabi",
        features: "",
    },
    Test {
        package: "examples/stm32wb55",
        target: "thumbv7em-none-eabihf",
        features: "",
    },
    Test {
        package: "examples/stm32wl55",
        target: "thumbv7em-none-eabihf",
        features: "",
    },
    Test {
        package: "examples/stm32f0",
        target: "thumbv6m-none-eabi",
        features: "",
    },
    Test {
        package: "examples/stm32g0",
        target: "thumbv6m-none-eabi",
        features: "",
    },
    Test {
        package: "examples/wasm",
        target: "wasm32-unknown-unknown",
        features: "",
    },
    Test {
        package: "examples/stm32f1",
        target: "thumbv7m-none-eabi",
        features: "",
    },
];

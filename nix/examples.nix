{ inputs, ... }: {
  imports = [ inputs.devshell.flakeModule ];
  perSystem = { pkgs, ... }: {
    devshells.default = {
      commands = [
        {
          name = "atmega328";
          help = "Build atmega328 examples";
          command = "cargo build --release --manifest-path embassy-executor/Cargo.toml --target avr-unknown-gnu-atmega328 -Z build-std=core,alloc --features nightly,arch-avr,avr-device/atmega328p";
        }
        {
          name = "nrf52840";
          help = "Build nrf52840 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/nrf52840/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/nrf52840";
        }
        {
          name = "nrf5340";
          help = "Build nrf5340 examples";
          command = "cargo build --release --manifest-path examples/nrf5340/Cargo.toml --target thumbv8m.main-none-eabihf --out-dir out/examples/nrf5340";
        }
        {
          name = "nrf9160";
          help = "Build nrf9160 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/nrf9160/Cargo.toml --target thumbv8m.main-none-eabihf --out-dir out/examples/nrf9160";
        }
        {
          name = "nrf51";
          help = "Build nrf51 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/nrf51/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/nrf51";
        }
        {
          name = "rp";
          help = "Build rp examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/rp/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/rp";
        }
        {
          name = "stm32f0";
          help = "Build stm32f0 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32f0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/stm32f0";
        }
        {
          name = "stm32f1";
          help = "Build stm32f1 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32f1/Cargo.toml --target thumbv7m-none-eabi --out-dir out/examples/stm32f1";
        }
        {
          name = "stm32f2";
          help = "Build stm32f2 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32f2/Cargo.toml --target thumbv7m-none-eabi --out-dir out/examples/stm32f2";
        }
        {
          name = "stm32f2";
          help = "Build stm32f2 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32f2/Cargo.toml --target thumbv7m-none-eabi --out-dir out/examples/stm32f2";
        }
        {
          name = "stm32f3";
          help = "Build stm32f3 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32f3/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32f3";
        }
        {
          name = "stm32f334";
          help = "Build stm32f334 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32f334/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32f334";
        }
        {
          name = "stm32f4";
          help = "Build stm32f4 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32f4/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32f4";
        }
        {
          name = "stm32f7";
          help = "Build stm32f7 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32f7/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32f7";
        }
        {
          name = "stm32c0";
          help = "Build stm32c0 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32c0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/stm32c0";
        }
        {
          name = "stm32g0";
          help = "Build stm32g0 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32g0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/stm32g0";
        }
        {
          name = "stm32g4";
          help = "Build stm32g4 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32g4/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32g4";
        }
        {
          name = "stm32h5";
          help = "Build stm32h5 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32h5/Cargo.toml --target thumbv8m.main-none-eabihf --out-dir out/examples/stm32h5";
        }
        {
          name = "stm32h7";
          help = "Build stm32h7 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32h7/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32h7";
        }
        {
          name = "stm32h7rs";
          help = "Build stm32h7rs examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32h7rs/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32h7rs";
        }
        {
          name = "stm32l0";
          help = "Build stm32l0 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32l0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/stm32l0";
        }
        {
          name = "stm32l1";
          help = "Build stm32l1 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32l1/Cargo.toml --target thumbv7m-none-eabi --out-dir out/examples/stm32l1";
        }
        {
          name = "stm32l4";
          help = "Build stm32l4 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32l4/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32l4";
        }
        {
          name = "stm32l5";
          help = "Build stm32l5 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32l5/Cargo.toml --target thumbv8m.main-none-eabihf --out-dir out/examples/stm32l5";
        }
        {
          name = "stm32u0";
          help = "Build stm32u0 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32u0/Cargo.toml --target thumbv6m-none-eabi --out-dir out/examples/stm32u0";
        }
        {
          name = "stm32u5";
          help = "Build stm32u5 examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32u5/Cargo.toml --target thumbv8m.main-none-eabihf --out-dir out/examples/stm32u5";
        }
        {
          name = "stm32wb";
          help = "Build stm32wb examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32wb/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32wb";
        }
        {
          name = "stm32wba";
          help = "Build stm32wba examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32wba/Cargo.toml --target thumbv8m.main-none-eabihf --out-dir out/examples/stm32wba";
        }
        {
          name = "stm32wl";
          help = "Build stm32wl examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/stm32wl/Cargo.toml --target thumbv7em-none-eabi --out-dir out/examples/stm32wl";
        }
        {
          name = "nrf52840-bootloader-skip-includes";
          help = "Build nrf52840 bootloader examples";
          command = "cargo build --release --manifest-path examples/boot/application/nrf/Cargo.toml --target -Z unstable-options thumbv7em-none-eabi --features embassy-nrf/nrf52840,skip-include --out-dir out/examples/boot/nrf52840";
        }
        {
          name = "nrf9160-bootloader";
          help = "Build nrf9160 bootloader examples";
          command = "cargo build --release --manifest-path examples/boot/application/nrf/Cargo.toml --target -Z unstable-options thumbv8m.main-none-eabihf --features embassy-nrf/nrf9160-ns,skip-include --out-dir out/examples/boot/nrf9160";
        }
        {
          name = "rp-bootloader";
          help = "Build rp bootloader examples";
          command = "cargo build --release --manifest-path examples/boot/application/rp/Cargo.toml --target -Z unstable-options thumbv6m-none-eabi --features skip-include --out-dir out/examples/boot/rp";
        }
        {
          name = "stm32f3-bootloader";
          help = "Build stm32f3 bootloader examples";
          command = "cargo build --release --manifest-path examples/boot/application/stm32f3/Cargo.toml --target -Z unstable-options thumbv7em-none-eabi --features skip-include --out-dir out/examples/boot/stm32f3";
        }
        {
          name = "stm32f7-bootloader";
          help = "Build stm32f7 bootloader examples";
          command = "cargo build --release --manifest-path examples/boot/application/stm32f7/Cargo.toml --target -Z unstable-options thumbv7em-none-eabi --features skip-include --out-dir out/examples/boot/stm32f7";
        }
        {
          name = "stm32h7-bootloader";
          help = "Build stm32h7 bootloader examples";
          command = "cargo build --release --manifest-path examples/boot/application/stm32h7/Cargo.toml --target -Z unstable-options thumbv7em-none-eabi --features skip-include --out-dir out/examples/boot/stm32h7";
        }
        {
          name = "stm32l0-bootloader";
          help = "Build stm32l0 bootloader examples";
          command = "cargo build --release --manifest-path examples/boot/application/stm32l0/Cargo.toml --target -Z unstable-options thumbv6m-none-eabi --features skip-include --out-dir out/examples/boot/stm32l0";
        }
        {
          name = "stm32l1-bootloader";
          help = "Build stm32l1 bootloader examples";
          command = "cargo build --release --manifest-path examples/boot/application/stm32l1/Cargo.toml --target -Z unstable-options thumbv7m-none-eabi --features skip-include --out-dir out/examples/boot/stm32l1";
        }
        {
          name = "stm32l4-bootloader";
          help = "Build stm32l4 bootloader examples";
          command = "cargo build --release --manifest-path examples/boot/application/stm32l4/Cargo.toml --target -Z unstable-options thumbv7em-none-eabi --features skip-include --out-dir out/examples/boot/stm32l4";
        }
        {
          name = "stm32wl-bootloader";
          help = "Build stm32wl bootloader examples";
          command = "cargo build --release --manifest-path examples/boot/application/stm32wl/Cargo.toml --target -Z unstable-options thumbv7em-none-eabi --features skip-include --out-dir out/examples/boot/stm32wl";
        }
        {
          name = "stm32wb-dfu-bootloader";
          help = "Build stm32wb-dfu bootloader examples";
          command = "cargo build --release --manifest-path examples/boot/application/stm32wb-dfu/Cargo.toml --target -Z unstable-options thumbv7em-none-eabi --out-dir out/examples/boot/stm32wb-dfu";
        }
        {
          name = "nrf52840-bootloader";
          help = "Build nrf52840 bootloader examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/boot/bootloader/nrf/Cargo.toml --target thumbv7em-none-eabi --features embassy-nrf/nrf52840";
        }
        {
          name = "nrf9160-ns-bootloader";
          help = "Build nrf9160-ns bootloader examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/boot/bootloader/nrf/Cargo.toml --target thumbv8m.main-none-eabihf --features embassy-nrf/nrf9160-ns";
        }
        {
          name = "rp-bootloader2";
          help = "Build rp-bootloader2 bootloader examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/boot/bootloader/rp/Cargo.toml --target thumbv6m-none-eabi";
        }
        {
          name = "stm32-bootloader";
          help = "Build stm32 bootloader examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/boot/bootloader/stm32/Cargo.toml --target thumbv7em-none-eabi --features embassy-stm32/stm32wl55jc-cm4";
        }
        {
          name = "stm32wb55rg-dfu-bootloader";
          help = "Build stm32wb55rg-dfu bootloader examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/boot/bootloader/stm32wb-dfu/Cargo.toml --target thumbv7em-none-eabi --features embassy-stm32/stm32wb55rg";
        }
        {
          name = "stm32-dual-bank-bootloader";
          help = "Build stm32-dual-bank bootloader examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/boot/bootloader/stm32-dual-bank/Cargo.toml --target thumbv7em-none-eabi --features embassy-stm32/stm32h747xi-cm7";
        }
        {
          name = "wasm";
          help = "Build wasm examples";
          command = "cargo build -Z unstable-options --release --manifest-path examples/wasm/Cargo.toml --target wasm32-unknown-unknown --out-dir out/examples/wasm";
        }
      ];
    };
  };
}

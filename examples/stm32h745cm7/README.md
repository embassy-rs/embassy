This example demonstrates how to use the display on the STM32H745I-DISCO board. It runs on the Cortex M7 core. It does not do anything with the CM4 core, so if you find that you have "weird" issues, disable the CM4 core using ``option byte`` ``BCM4=0`` or just flash an empty loop onto it.

You can set the option byte using the following command:

```sh
.\STM32_Programmer_CLI.exe -c port=SWD -ob BCM4=0 BCM7=1
```

---

The display code was adapted from [the STM32H735 sample](../stm32h735/src/bin/ltdc.rs).

If you're interested in creating a multi-core application, please take a look at the stm32h755 examples ([cm4](../stm32h755cm4) and [cm7](../stm32h755cm7)).
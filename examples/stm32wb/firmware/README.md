# STM32WB5x coprocessor binaries

`fus_update` embeds the ST-signed coprocessor binaries from the STM32CubeWB package
and installs them with the FUS (firmware upgrade services) at runtime — no
STM32CubeProgrammer needed.

Download them from the
[STM32CubeWB repository](https://github.com/STMicroelectronics/STM32CubeWB/tree/master/Projects/STM32WB_Copro_Wireless_Binaries/STM32WB5x)
(also mirrored in the STM32CubeWB MCU package and on st.com):

```sh
cd examples/stm32wb/firmware
base=https://raw.githubusercontent.com/STMicroelectronics/STM32CubeWB/master/Projects/STM32WB_Copro_Wireless_Binaries/STM32WB5x
for f in stm32wb5x_FUS_fw.bin stm32wb5x_FUS_fw_1_2_0.bin stm32wb5x_FUS_fw_for_fus_0_5_3.bin stm32wb5x_BLE_Mac_802_15_4_fw.bin; do
    curl -fLO "$base/$f"
done
```

| File                              | Purpose                                            |
| --------------------------------- | -------------------------------------------------- |
| `stm32wb5x_FUS_fw_for_fus_0_5_3.bin` | upgrades FUS V0.5.3 (factory default) to V1.2.0 |
| `stm32wb5x_FUS_fw_1_2_0.bin`      | upgrades FUS V1.x (< V1.2.0) to V1.2.0             |
| `stm32wb5x_FUS_fw.bin`            | upgrades FUS V1.2.0 to the latest FUS V2           |
| `stm32wb5x_BLE_Mac_802_15_4_fw.bin` | BLE + MAC 802.15.4 combo wireless stack          |

See the `Release_Notes.html` in the same repository directory for version details.

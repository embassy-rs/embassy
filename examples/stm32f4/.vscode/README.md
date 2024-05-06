# VS Code Configuration

Example configurations for debugging programs in-editor with VS Code.  
This directory contains configurations for two platforms:

 - `LM3S6965EVB` on QEMU
 - `STM32F303x` via OpenOCD

## Required Extensions

If you have the `code` command in your path, you can run the following commands to install the necessary extensions.

```sh
code --install-extension rust-lang.rust-analyzer
code --install-extension marus25.cortex-debug
```

Otherwise, you can use the Extensions view to search for and install them, or go directly to their marketplace pages and click the "Install" button.

- [Rust Language Server (rust-analyzer)](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [Cortex-Debug](https://marketplace.visualstudio.com/items?itemName=marus25.cortex-debug)

## Use

The quickstart comes with two debug configurations.
Both are configured to build the project, using the default settings from `.cargo/config`, prior to starting a debug session.

1. QEMU: Starts a debug session using an emulation of the `LM3S6965EVB` mcu.
   - This works on a fresh `cargo generate` without modification of any of the settings described above.
   - Semihosting output will be written to the Output view `Adapter Output`.
   - `ITM` logging does not work with QEMU emulation.

2. OpenOCD: Starts a debug session for a `STM32F3DISCOVERY` board (or any `STM32F303x` running at 8MHz).
   - Follow the instructions above for configuring the build with `.cargo/config` and the `memory.x` linker script.
   - `ITM` output will be written to the Output view `SWO: ITM [port: 0, type: console]` output.

### Git

Files in the `.vscode/` directory are `.gitignore`d by default because many files that may end up in the `.vscode/` directory should not be committed and shared.  
If you would like to save this debug configuration to your repository and share it with your team, you'll need to explicitly `git add` the files to your repository.

```sh
git add -f .vscode/launch.json
git add -f .vscode/tasks.json
git add -f .vscode/*.svd
```

## Customizing for other targets

For full documentation, see the [Cortex-Debug][cortex-debug] repository.

### Device

Some configurations use this to automatically find the SVD file.  
Replace this with the part number for your device.

```json
"device": "STM32F303VCT6",
```

### OpenOCD Config Files

The `configFiles` property specifies a list of files to pass to OpenOCD.

```json
"configFiles": [
    "interface/stlink-v2-1.cfg",
    "target/stm32f3x.cfg"
],
```

See the [OpenOCD config docs][openocd-config] for more information and the [OpenOCD repository for available configuration files][openocd-repo].

### SVD

The SVD file is a standard way of describing all registers and peripherals of an ARM Cortex-M mCU.  
Cortex-Debug needs this file to display the current register values for the peripherals on the device.  

You can probably find the SVD for your device on the vendor's website.  


For example, the STM32F3DISCOVERY board uses an mcu from the `STM32F303x` line of processors.  
All the SVD files for the STM32F3 series are available on [ST's Website][stm32f3].  
Download the [stm32f3 SVD pack][stm32f3-svd], and copy the `STM32F303.svd` file into `.vscode/`.  
This line of the config tells the Cortex-Debug plug in where to find the file.

```json
"svdFile": "${workspaceRoot}/.vscode/STM32F303.svd",
```

For other processors, simply copy the correct `*.svd` file into the project and update the config accordingly.

### CPU Frequency

If your device is running at a frequency other than 8MHz, you'll need to modify this line of `launch.json` for the `ITM` output to work correctly.

```json
"cpuFrequency": 8000000,
```

### Other GDB Servers

For information on setting up GDB servers other than OpenOCD, see the [Cortex-Debug repository][cortex-debug].

[cortex-debug]: https://github.com/Marus/cortex-debug
[stm32f3]: https://www.st.com/content/st_com/en/products/microcontrollers-microprocessors/stm32-32-bit-arm-cortex-mcus/stm32-mainstream-mcus/stm32f3-series.html#resource
[stm32f3-svd]: https://www.st.com/resource/en/svd/stm32f3_svd.zip
[openocd-config]: http://openocd.org/doc/html/Config-File-Guidelines.html
[openocd-repo]: https://sourceforge.net/p/openocd/code/ci/master/tree/tcl/

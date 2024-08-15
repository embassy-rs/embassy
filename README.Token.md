# Token modifications to baseline embassy.

1. **Files**: `*/.cargo/config.toml`

    * **Changes**: Added our local repository information.
    * **Rationale**: So that we can publish to our artifactory insance.
    * **Status**: No longer necessary. We aren't publishing to artifactory anymore. We are consuming `embassy` as a
    submodule.
    * **Conflict Potential**: Low
        * Upstream would have to add a configuration of their own.

1. **File**: `embassy-boot-stm32/Cargo.toml`

    * **Changes**:
        1. Added `defmt-rtt` dependency.
        1. Enabled the `rt` feature in `embassy-stm32`.
        1. Set `stm32wb55vg` as a `default` feature.
        1. Set `debug` to `defmt-rtt`
        1. Added a bunch of boards as options (including ours).
    * **Rationale**:
        1. Allows us to use `defmt-rtt` for debugging.
        1. Enables the minimal runtime for cortex-m. See [the embassy docs for that feature](https://docs.embassy.dev/embassy-stm32/git/stm32f103c8/index.html#feature-flags) which enables the [rt feature in stm32-metapac](https://docs.rs/crate/stm32-metapac/latest/features).
        1. This is our chip variant. Defaulting it here means we don't need to set it everywhere.
        1. Automatically uses `defmt-rtt` for debug output when debugging is enabled.
        1. Allows us to select our board.
    * **Status**: Necessary
    * **Conflict Potential**: Medium

1. **File**: `embassy-boot/Cargo.toml`

    * **Changes**: Changed uses of `default_features` to `default-features`.
    * **Rationale**: Suppresses a deprecation warning from the linter.
    * **Status**: Necessary, unless you want to be annoyed.
    * **Conflict Potential**: High
        * Eventually, upstream is going to make this change, at which point we will be guaranteed a conflict. However, we should just accept it and then we will no longer be deviating from upstream.

1. **Files**: `embassy-executor-macros/Cargo.toml`, `embassy-futures/Cargo.toml`, and `embassy-time-queue-driver/Cargo.toml`.

    * **Changes**: Set `doctest` to `false`.
    * **Rationale**: We don't care about a third party module's documentation failing tests.
    * **Status**: Necessary, unless we want to add the docs ourselves.
    * **Conflict Potential**: Low

1. **File**: `embassy-executor/Cargo.toml`

    * **Changes**: Added `optional = true` to the `embassy-time-driver` and `embassy-time-queue-driver` dependency lines.
    * **Rationale**: We want to be able to switch the dependency on and off.
    * **Status**: Necessary, assuming that we want to have this flexibility.
    * **Conflict Potential**: Medium
       * There's a version number in this line, so if upstream ever updates the version, it will conflict.

1. **File**: `embassy-stm32-wpan/Cargo.toml`

    * **Changes**:
        1. Changed `embassy-stm32`'s dependency line to be ` default-features = false, features=["rt"]`.
        1. Some option ordering on some other lines.
        1. `stm32wb-hci` line changed to use our artifactory and version.
        1. Add a `default` entry enabling the `stm32wb55vg` feature.
    * **Rationale**:
        1. We're using the minimal runtime for cortex-m (See [the embassy docs for that feature](https://docs.embassy.dev/embassy-stm32/git/stm32f103c8/index.html#feature-flags) which enables the [rt feature in stm32-metapac](https://docs.rs/crate/stm32-metapac/latest/features).), but none of the other features for this crate.
        1. I expect someone's automatic formatter made these changes, as there is no functional change here.
        1. We added things to `stm32wb-hci` relative to upstream, so we need to use our fork.
        1. We're using this chip variant.
    * **Status**:
        1. Necessary
        1. Unnecessary, but the code formatter is going to change it on save every time we touch this file.
        1. Necessary
        1. Necessary
    * **Conflict Potential**: High
       1. There are a version numbers in these lines, so if upstream ever updates the version, it will conflict.
       1. As above.
       1. As above, but more, as upstream `stm32wb-hci` changes more frequently - and therefore we need to merge the changes into ours, update it, etc.
       1. Will conflict if upstream adds a default features line.

1. **File**: `embassy-stm32-wpan/src/sub/sys.rs`

    * **Changes**: Added `shci_c2_flash_erase_activity` function.
    * **Rationale**: We need to be able to tell CPU2 that we're erasing flash (per best practices as detailed on the [Flash and BLE contention page](https://tokenring.atlassian.net/wiki/spaces/Firmware/pages/216301570/Flash+and+BLE+contention)).
    * **Status**: Necessary
    * **Conflict Potential**: Low
       * There will only be a conflict if upstream adds this.

1. **File**: `embassy-stm32-wpan/src/unsafe_linked_list.rs`

    * **Changes**: Commented out a debug line in the `debug_linked_list` function.
    * **Rationale**: Taylor found this annoying. He committed the change with the message "Remove problematic print statement".
    * **Status**: Unnecessary
    * **Conflict Potential**: Low

1. **File**: `embassy-stm32/Cargo.toml`

    * **Changes**:
        1. Changed the `stm32-metapac` line to use our repository and version.
        1. Added `stm32wb55vg` to the `default` feature list.
        1. Added lines to allow TIM16 and TIM17 as the time driver.
    * **Rationale**:
        1. We added things to `stm32-metapac` relative to upstream, so we need to use our fork.
        1. This is the chip variant that we are using.
        1. We have these timers in our chip, this allows us to use them.
    * **Status**: Necessary
    * **Conflict Potential**: High
        1. Upstream `stm32-metapac` changes relatively frequently - and therefore we need to merge the changes into ours, update it, etc.
        1. If upstream changes the default feature list, it will conflict.
        1. If upstream ever adds these, it will conflict.

1. **File**: `embassy-stm32/build.rs`

    * **Changes**:
        1. Added build support for using TIM16 and TIM17 as the time drivers.
        1. Added build support for using the AES hardware.
    * **Rationale**:
        1. We have these timers in our chip, this allows us to use them.
        1. We have AES hardware in our chip, this allows us to use it.
    * **Status**: Necessary
    * **Conflict Potential**: Medium
        1. If upstream ever adds these, it will conflict.
        1. As above.

1. **File**: `embassy-stm32/src/aes/mod.rs`

    * **Changes**: Added AES driver.
    * **Rationale**: We wanted to be able to use the AES hardware.
    * **Status**: Necessary, unless we want to drop hardware AES.
    * **Conflict Potential**: Medium
        * If upstream ever adds this driver, it will conflict.

1. **File**: `embassy-stm32/src/exti.rs`

    * **Changes**: Changed to not require a mutable reference when waiting on an external interrupt.
    * **Rationale**: Unsure. Taylor made this change and didn't explain, apart from:
        > don't require mutable reference for EXTI waiting
        >
        > Requires removing implementation of embedded-hal, otherwise it creates a recursive loop

        And there was no ticket associated with it.
    * **Status**: Unsure if necessary.
    * **Conflict Potential**: Low

1. **File**: `embassy-stm32/src/hsem/mod.rs`

    * **Changes**: Fix a bug in the HSEM implementation on our core.
    * **Rationale**: HSEM was panicking on CPUID check. (See [TR3FIR-238](https://tokenring.atlassian.net/browse/TR3FIR-238).)
    * **Status**: Necessary.
    * **Conflict Potential**: Medium
        * Eventually, upstream will fix this bug and we'll have to replace our fix with theirs.

1. **File**: `embassy-stm32/src/lib.rs`

    * **Changes**:
        1. Added some feature flags to the library crate which are necessary to support the AES driver.
        1. Added the AES driver as a `mod` so it gets compiled in.
    * **Rationale**:
        1. We need these flags enable to support code constructs used by the AES driver.
        1. We wanted the AES driver.
    * **Status**: Necessary, unless we want to drop hardware AES.
    * **Conflict Potential**: Medium
        * If upstream ever adds this driver, it will conflict.

1. **File**: `embassy-stm32/src/rcc/l.rs`

    * **Changes**: Don't enable the PLL unless we were provided with a config.
    * **Rationale**: Power savings. Fixed in [TR3FIR-168](https://tokenring.atlassian.net/browse/TR3FIR-168) where we don't enable clocks we don't want.
    * **Status**: Necessary, for power savings.
    * **Conflict Potential**: Low

1. **File**: `embassy-stm32/src/spi/mod.rs`

    * **Changes**: Added `allow(dead_code)` for an unused function.
    * **Rationale**: Newly-merged code from upstream was now causing a build warning, so suppress it.
    * **Status**: Necessary, unless you like compilation warnings.
    * **Conflict Potential**: Low

1. **File**: `embassy-stm32/src/time_driver.rs`

    * **Changes**:
        1. Added support for using TIM16 and TIM17.
        1. Change `calc_now` logic to handle overflow.
        1. Enable overflow interrupts and handle them.
    * **Rationale**:
        1. We have these timers in our chip, this allows us to use them.
        1. We wanted to be able to calculate the current time properly, even if the counter had overflowed.
        1. As above.
    * **Status**: Necessary
    * **Conflict Potential**: Low

1. **Files**: `rust-toolchain-nightly.toml` and `rust-toolchain-stable.toml`

    * **Changes**: Swapped the two files.
    * **Rationale**: We are using unstable nightly features.
    * **Status**: Necessary, though we could do this better.
        * Rather than just lying to `cargo` and telling it that the nightly toolchain is the stable one, there should be a way to just tell it to use the nightly toolchain for all builds.
    * **Conflict Potential**: Medium
        * Whenever upstream updates their toolchain, we need to adjust this accordingly.

1. **Files**: `README.Token.md`

    * **Changes**: Added Token-specifc readme.
    * **Rationale**: We wanted documentation of our changes.
    * **Status**: Necessary
    * **Conflict Potential**: None
        * Does not exist upstream, and highly unlikely they would ever add it.

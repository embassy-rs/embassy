//! Replicate `whd_chip.c` functionality

use embassy_time::{Duration, Timer};

use crate::consts::*;
use crate::runner::Bus;
use crate::util::try_until;
use crate::{Chip, ChipId, Core, WithContext};

/// Returns `true` is the core identified by the provided coreId is up, otherwise `false`
pub async fn check_device_core_is_up(bus: &mut impl Bus, chip: impl Chip, core: Core) -> crate::Result<()> {
    let base = chip.base_addr(core);

    let io = bus.bp_read8(base + AI_IOCTRL_OFFSET).await;
    if io & (AI_IOCTRL_BIT_FGC | AI_IOCTRL_BIT_CLOCK_EN) != AI_IOCTRL_BIT_CLOCK_EN {
        debug!("device_core_is_up: returning false due to bad ioctrl {:02x}", io);
        return Err(crate::Error);
    }

    let r = bus.bp_read8(base + AI_RESETCTRL_OFFSET).await;
    if r & (AI_RESETCTRL_BIT_RESET) != 0 {
        debug!("device_core_is_up: returning false due to bad resetctrl {:02x}", r);
        return Err(crate::Error);
    }

    return Ok(());
}

/// Resets the core identified by the provided coreId
pub async fn reset_core(
    bus: &mut impl Bus,
    chip: impl Chip,
    core: Core,
    halt: bool,
    reset_halt: bool,
) -> crate::Result<()> {
    let base = chip.base_addr(core);

    async fn wait_for_backplane_idle(bus: &mut impl Bus, base: u32) -> crate::Result<()> {
        try_until(
            async || bus.bp_read8(base + AI_RESETSTATUS_OFFSET).await != 0,
            Duration::from_millis(300),
        )
        .await
        .ctx("timeout while waiting for backplane idle")
    }

    // ensure there are no pending backplane operations
    wait_for_backplane_idle(bus, base).await?;

    // put core into reset state
    bus.bp_write8(base + AI_RESETCTRL_OFFSET, AI_RESETCTRL_BIT_RESET).await;

    // ensure there are no pending backplane operations
    wait_for_backplane_idle(bus, base).await?;

    bus.bp_write8(
        base + AI_IOCTRL_OFFSET,
        if halt || reset_halt {
            AI_IOCTRL_BIT_CPUHALT | AI_IOCTRL_BIT_FGC | AI_IOCTRL_BIT_CLOCK_EN
        } else {
            AI_IOCTRL_BIT_FGC | AI_IOCTRL_BIT_CLOCK_EN
        },
    )
    .await;

    // whd tries ten times to take core out of reset
    let mut reset_state: u8 = 0;
    for _ in 0..10 {
        // ensure there are no pending backplane operations
        wait_for_backplane_idle(bus, base).await?;

        // take core out of reset
        bus.bp_write8(base + AI_RESETCTRL_OFFSET, 0).await;

        // ensure there are no pending backplane operations
        wait_for_backplane_idle(bus, base).await?;

        // verify the core is out of reset
        reset_state = bus.bp_read8(base + AI_RESETCTRL_OFFSET).await;
        if reset_state == 0 {
            break;
        }
    }

    bus.bp_write8(
        base + AI_IOCTRL_OFFSET,
        if halt || reset_halt {
            AI_IOCTRL_BIT_CPUHALT | AI_IOCTRL_BIT_CLOCK_EN
        } else {
            AI_IOCTRL_BIT_CLOCK_EN
        },
    )
    .await;

    match reset_state {
        0 => Ok(()),
        _ => {
            debug!("reset_core: failed to take core out of reset {:02x}", reset_state);

            Err(crate::Error)
        }
    }
}

/// Disables the core identified by the provided coreId
pub async fn disable_device_core(bus: &mut impl Bus, chip: impl Chip, core: Core, halt: bool) -> crate::Result<()> {
    let base = chip.base_addr(core);

    // read the reset control
    let _ = bus.bp_read8(base + AI_RESETCTRL_OFFSET);

    // read the reset control and check if it is already in reset
    if bus.bp_read8(base + AI_RESETCTRL_OFFSET).await & AI_RESETCTRL_BIT_RESET != 0 {
        // core already in reset
        return Ok(());
    }

    // Write 0 to the IO control and read it back
    bus.bp_write8(base + AI_IOCTRL_OFFSET, if halt { AI_IOCTRL_BIT_CPUHALT } else { 0 })
        .await;

    let _ = bus.bp_read8(base + AI_IOCTRL_OFFSET);

    Timer::after_millis(1).await;

    // put core into reset state
    bus.bp_write8(base + AI_RESETCTRL_OFFSET, AI_RESETCTRL_BIT_RESET).await;

    Timer::after_millis(1).await;

    Ok(())
}

/// Resets the core identified by the provided coreId
pub async fn reset_device_core(bus: &mut impl Bus, chip: impl Chip, core: Core, halt: bool) -> crate::Result<()> {
    let base = chip.base_addr(core);

    disable_device_core(bus, chip, core, halt).await?;

    bus.bp_write8(
        base + AI_IOCTRL_OFFSET,
        if halt {
            AI_IOCTRL_BIT_CPUHALT | AI_IOCTRL_BIT_FGC | AI_IOCTRL_BIT_CLOCK_EN
        } else {
            AI_IOCTRL_BIT_FGC | AI_IOCTRL_BIT_CLOCK_EN
        },
    )
    .await;

    let _ = bus.bp_read8(base + AI_IOCTRL_OFFSET).await;
    bus.bp_write8(base + AI_RESETCTRL_OFFSET, 0).await;

    Timer::after_millis(1).await;

    bus.bp_write8(
        base + AI_IOCTRL_OFFSET,
        if halt {
            AI_IOCTRL_BIT_CPUHALT | AI_IOCTRL_BIT_CLOCK_EN
        } else {
            AI_IOCTRL_BIT_CLOCK_EN
        },
    )
    .await;

    let _ = bus.bp_read8(base + AI_IOCTRL_OFFSET).await;
    Timer::after_millis(1).await;

    Ok(())
}

pub async fn chip_specific_socsram_init(bus: &mut impl Bus, chip: impl Chip) -> crate::Result<()> {
    if matches!(chip.id(), ChipId::C43439) {
        // this is 4343x specific stuff: Disable remap for SRAM_3
        bus.bp_write32(chip.socsram_base_address() + 0x10, 3).await;
        bus.bp_write32(chip.socsram_base_address() + 0x44, 0).await;
    }

    Ok(())
}

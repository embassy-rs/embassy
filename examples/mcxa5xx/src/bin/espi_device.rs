//! Interactive eSPI device example for the FRDM-MCXA577.
//!
//! Rust port of the MCUX SDK `espi_device` test bench used with the TotalPhase
//! Promira eSPI host/analyzer setup (see the "MCXA577 eSPI Test Bench and
//! Development" write-up). The device exposes:
//! - an OOB mailbox (port 0), an ACPI endpoint at IO 0x100 (port 1), an
//!   index/data endpoint at IO 0x200 (port 2), a mailbox at IO 0x300 (port 3),
//!   and a 256-byte virtual slave-attached flash (port 4, base0 = 0x1000);
//! - Port 80 POST-code capture and virtual-wire reporting;
//! - the same serial console commands as the SDK example on the FRDM debug
//!   UART (LPUART1, 115200-8-N-1) so `espi_autotest.py` / `espi_generator.py`
//!   can drive it.

#![no_std]
#![no_main]

use core::fmt::Write as _;

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::clocks::config::{Div8, SoscConfig, SoscMode};
use embassy_mcxa::espi::{
    AddrBase, Config as EspiDriverConfig, Espi, EspiRam, Event, FlashRequest, FlashRequestKind, GpioWire,
    InterruptHandler, MailboxStatus, OMFLEN_SAF_COMPLETION_FAIL, OMFLEN_SAF_COMPLETION_NO_DATA,
    OMFLEN_SAF_COMPLETION_WITH_DATA, PortConfig, PortError, PortEvent, PortType, RamSize, SSTCL_SAF_COMPLETION,
    SSTCL_SAF_REQ_ACCEPTED, SafCompletionType, VWireOut,
};
use embassy_mcxa::lpuart::{Buffered, BufferedInterruptHandler, Config as UartConfig, Lpuart, LpuartTx};
use embassy_mcxa::{bind_interrupts, peripherals};
use embedded_io_async::Write;
use heapless::String;
use static_cell::StaticCell;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    ESPI => InterruptHandler;
    LPUART1 => BufferedInterruptHandler<peripherals::LPUART1>;
});

const FLASH_SIZE: usize = 256;
const SAF_RX_SPLIT_MAX: usize = 4;

/// Pending split completions for a host flash read (port of `g_readQueue`).
struct ReadQueue {
    item: [(u32, u32, SafCompletionType); SAF_RX_SPLIT_MAX],
    tag: u8,
    cur: usize,
    tot: usize,
}

impl ReadQueue {
    const fn new() -> Self {
        Self {
            item: [(0, 0, SafCompletionType::Only); SAF_RX_SPLIT_MAX],
            tag: 0,
            cur: 0,
            tot: 0,
        }
    }
}

static ESPI_RAM: StaticCell<EspiRam> = StaticCell::new();

/// UART print helper: format into a stack buffer, then write it out.
macro_rules! uprint {
    ($tx:expr, $($arg:tt)*) => {{
        let mut s: String<512> = String::new();
        let _ = write!(s, $($arg)*);
        let _ = $tx.write_all(s.as_bytes()).await;
    }};
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut hal_config = hal::config::Config::default();
    // Match the SDK board profile: 24 MHz SOSC available, FRO_LF divider for
    // the debug UART. The eSPI functional clock uses FRO_HF undivided.
    hal_config.clock_cfg.sosc = Some(SoscConfig {
        mode: SoscMode::CrystalOscillator,
        frequency: 24_000_000,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
    });
    hal_config.clock_cfg.sirc.fro_12m_enabled = true;
    hal_config.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    let p = hal::init(hal_config);
    info!("=== eSPI device example ===");

    // Debug-console UART (FRDM MCU-Link CMSIS-DAP COM port).
    let uart_config = UartConfig {
        baudrate_bps: 115_200,
        rx_fifo_watermark: 0,
        tx_fifo_watermark: 0,
        ..Default::default()
    };
    static TX_BUF: StaticCell<[u8; 512]> = StaticCell::new();
    static RX_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    let mut uart = Lpuart::new_buffered(
        p.LPUART1,
        p.P1_9,
        p.P1_8,
        Irqs,
        TX_BUF.init([0; 512]),
        RX_BUF.init([0; 64]),
        uart_config,
    )
    .unwrap();
    let (tx, rx) = uart.split_ref();

    // Port layout of the SDK test bench (`g_portCfg`).
    let ports = [
        PortConfig {
            port_type: PortType::MailboxOobSplit,
            direction: 0,
            ram_offset: 0x0000,
            ram_size: RamSize::Size256B,
            addr_offset: 0x0000,
            addr_base: AddrBase::Direct,
            idx_offset: 0,
        },
        PortConfig {
            port_type: PortType::AcpiEndpoint,
            direction: 0,
            ram_offset: 0x0100,
            ram_size: RamSize::Size4B,
            addr_offset: 0x0100,
            addr_base: AddrBase::Direct,
            idx_offset: 0,
        },
        PortConfig {
            port_type: PortType::AcpiIndexData,
            direction: 1,
            ram_offset: 0x0200,
            ram_size: RamSize::Size4B,
            addr_offset: 0x0200,
            addr_base: AddrBase::Direct,
            idx_offset: 1,
        },
        PortConfig {
            port_type: PortType::MailboxSingle,
            direction: 0,
            ram_offset: 0x0300,
            ram_size: RamSize::Size256B,
            addr_offset: 0x0300,
            addr_base: AddrBase::Direct,
            idx_offset: 0,
        },
        PortConfig {
            port_type: PortType::BusMasterFlashSingle,
            direction: 0,
            ram_offset: 0x0500,
            ram_size: RamSize::Size64B,
            addr_offset: 0x0500,
            addr_base: AddrBase::Base0,
            idx_offset: 0,
        },
    ];

    let mut espi_config = EspiDriverConfig::default();
    espi_config.base0_addr = 0x1000;
    espi_config.base1_addr = 0x2000;
    espi_config.enable_alert_pin = true;
    espi_config.enable_oob = true;
    espi_config.enable_saf = true;
    espi_config.enable_p80 = true;

    let ram = ESPI_RAM.init(EspiRam::new());
    let mut espi = match Espi::new(
        p.ESPI0,
        Irqs,
        p.P4_10, // CLK
        p.P4_11, // CSn
        p.P4_9,  // DATA0
        p.P4_8,  // DATA1
        p.P4_13, // DATA2
        p.P4_12, // DATA3
        p.P4_6,  // RST
        p.P4_7,  // NOTIFY
        ram,
        &ports,
        espi_config,
    ) {
        Ok(espi) => espi,
        Err(e) => defmt::panic!("eSPI init failed: {:?}", e),
    };
    info!("eSPI configured; waiting for host");

    let mut flash = [0u8; FLASH_SIZE];
    let mut queue = ReadQueue::new();
    let mut last_gpio = espi.gpio_wires();
    let mut line = [0u8; 48];
    let mut line_len = 0usize;
    let mut chunk = [0u8; 16];

    uprint!(tx, "\r\nInteractive eSPI device example.\r\n");
    print_help(tx).await;
    uprint!(tx, "> ");

    loop {
        match select(espi.wait_event(), rx.read(&mut chunk)).await {
            Either::First(event) => {
                handle_event(&mut espi, tx, &mut flash, &mut queue, &mut last_gpio, event).await;
            }
            Either::Second(Ok(n)) => {
                for &byte in &chunk[..n] {
                    match byte {
                        b'\r' | b'\n' => {
                            uprint!(tx, "\r\n");
                            if line_len > 0 {
                                let cmd = core::str::from_utf8(&line[..line_len]).unwrap_or("");
                                handle_command(&mut espi, tx, cmd).await;
                                line_len = 0;
                            }
                            uprint!(tx, "> ");
                        }
                        0x08 | 0x7F => {
                            if line_len > 0 {
                                line_len -= 1;
                                uprint!(tx, "\x08 \x08");
                            }
                        }
                        0x20..=0x7E if line_len < line.len() => {
                            line[line_len] = byte;
                            line_len += 1;
                            let _ = tx.write_all(&[byte]).await;
                        }
                        _ => {}
                    }
                }
            }
            Either::Second(Err(_)) => {}
        }
    }
}

// =========================================================================
// Event handling (port of ESPI_CommonCallback / ESPI_PortCallback)
// =========================================================================

async fn handle_event(
    espi: &mut Espi<'_>,
    tx: &mut LpuartTx<'_, Buffered>,
    flash: &mut [u8; FLASH_SIZE],
    queue: &mut ReadQueue,
    last_gpio: &mut GpioWire,
    event: Event,
) {
    match event {
        Event::BusReset => {
            uprint!(tx, "eSPI bus reset.\r\n");
            print_espi_config(espi, tx).await;
        }
        Event::CrcError => uprint!(tx, "eSPI bus CRC Error!\r\n"),
        Event::HostStall => info!("espi: host stall"),
        Event::WireChange(vw) => {
            uprint!(tx, "Virtual wire change: =0x{:08X}\r\n", vw.0);
            for (bit, name) in [
                (vw.slp_s3n(), " - SLP_S3N\r\n"),
                (vw.slp_s4n(), " - SLP_S4N\r\n"),
                (vw.slp_s5n(), " - SLP_S5N\r\n"),
                (vw.sus_stat(), " - SUS_STAT\r\n"),
                (vw.pltrstn(), " - PLTRST\r\n"),
                (vw.oob_rst_warn(), " - OOB_RST_WARN\r\n"),
                (vw.host_rst_warn(), " - HOST_RST_WARN\r\n"),
                (vw.sus_warn(), " - SUS_WARN\r\n"),
                (vw.sus_pwrdn_ackn(), " - SUS_PWRDN_ACK\r\n"),
                (vw.slp_an(), " - SLP_AN\r\n"),
                (vw.slp_lan(), " - SLP_LAN\r\n"),
                (vw.slp_wlan(), " - SLP_WLAN\r\n"),
            ] {
                if bit {
                    uprint!(tx, "{}", name);
                }
            }
            if vw.p2e() != 0 {
                uprint!(tx, " - P2E group set = {:02X}\r\n", vw.p2e());
            }
            if vw.host_c10n() {
                uprint!(tx, " - HOST_C10N\r\n");
            }
        }
        Event::GpioWire(gpio) => {
            // Only report actual changes, like the SDK example.
            if gpio != *last_gpio {
                if gpio.valid != 0 {
                    uprint!(
                        tx,
                        "\r\nVirtual wire change: index {}, pin {}\r\n",
                        gpio.index,
                        gpio.level
                    );
                }
                *last_gpio = gpio;
            }
        }
        Event::IrqPushDone => uprint!(tx, "IRQ update completed\r\n"),
        Event::Port80(p80) => {
            uprint!(
                tx,
                "P80 Code 0x{:02X} (prev: 0x{:02X}, count: {})\r\n",
                p80.current,
                p80.previous,
                p80.counter
            );
        }
        Event::Port(pe) => handle_port_event(espi, tx, flash, queue, pe).await,
    }
}

async fn handle_port_event(
    espi: &mut Espi<'_>,
    tx: &mut LpuartTx<'_, Buffered>,
    flash: &mut [u8; FLASH_SIZE],
    queue: &mut ReadQueue,
    pe: PortEvent,
) {
    if let Some(error) = pe.error {
        // Numeric codes and explanations match the SDK's espi_port_error_t.
        let (code, explain) = match error {
            PortError::EndpointWriteOverrun => (1, Some(" - Endpoint Write Overrun: Host wrote when WRDY=1.\r\n")),
            PortError::EndpointReadEmpty => (2, None),
            PortError::EndpointInvalidSize => (3, Some(" - Endpoint Invalid Size: Transfer size > 1 byte.\r\n")),
            PortError::MailboxInvalidAccess => (
                10,
                Some(" - Mailbox Invalid Access: Invalid host read/write access.\r\n"),
            ),
            PortError::MailboxOverrunUnderrun => (
                11,
                Some(" - Mailbox Overrun/Underrun: Write overrun or read underrun.\r\n"),
            ),
            PortError::MailboxSizeOverflow => (
                12,
                Some(" - Mailbox Size Overflow: Request size exceeds mailbox boundary.\r\n"),
            ),
            PortError::MailboxRamBusError => (13, Some(" - Mailbox RAM/Bus Error: AHB/RAM access error.\r\n")),
            PortError::MasterFromHostFailed => (20, None),
            PortError::MasterOverrunUnderrun => (21, None),
            PortError::MasterEraseFailed => (22, Some(" - Flash erase failed.\r\n")),
            PortError::MasterBusError => (23, None),
        };
        uprint!(tx, "Port {} error code {}\r\n", pe.port, code);
        if let Some(explain) = explain {
            uprint!(tx, "{}", explain);
        }
    }

    if Some(pe.port) == espi.oob_port() {
        if pe.read {
            uprint!(tx, "OOB sent over.\r\n");
        }
        if pe.write {
            let mut buf = [0u8; 256];
            if let Ok(len) = espi.read_oob(&mut buf) {
                uprint!(tx, "OOB received ({} bytes): ", len);
                for byte in &buf[..len] {
                    uprint!(tx, "{:02X} ", byte);
                }
                uprint!(tx, "\r\n");
            }
        }
        return;
    }

    if Some(pe.port) == espi.saf_port() {
        if let Some(req) = espi.flash_request(&pe) {
            handle_flash_request(espi, tx, flash, queue, req).await;
        }
        return;
    }

    match pe.port {
        // Port 1: ACPI endpoint.
        1 => {
            if pe.write {
                let (idx, data) = espi.endpoint_data(pe.port);
                uprint!(
                    tx,
                    "Received endpoint message: idx = 0x{:X}, datain = 0x{:X}\r\n",
                    idx,
                    data
                );
                if idx == 0 {
                    espi.write_endpoint_data(pe.port, data);
                    uprint!(tx, "Endpoint data ready: 0x{:X}\r\n", data);
                }
            }
            if pe.read {
                let (idx, _) = espi.endpoint_data(pe.port);
                uprint!(tx, "Endpoint data read: idx = 0x{:X}\r\n", idx);
            }
        }
        // Port 2: ACPI index/data.
        2 => {
            if pe.spec0 {
                let (idx, data) = espi.endpoint_data(pe.port);
                uprint!(
                    tx,
                    "Received Index-data message: idx = 0x{:X}, datain = 0x{:X}\r\n",
                    idx,
                    data
                );
            }
            if pe.read {
                let (idx, _) = espi.endpoint_data(pe.port);
                espi.write_endpoint_data(pe.port, 0xBB);
                uprint!(
                    tx,
                    "Index-data sent back: idx = 0x{:X}, dataout =0x{:X}\r\n",
                    idx,
                    0xBBu32
                );
            }
        }
        // Other ports: mailbox handling.
        _ => {
            if pe.write || pe.spec0 {
                let mut buf = [0u8; 256];
                let len = espi.read_mailbox(pe.port, &mut buf);
                uprint!(tx, "Mailbox received ({} bytes): ", len);
                for byte in &buf[..len] {
                    uprint!(tx, "{:02X} ", byte);
                }
                uprint!(tx, "\r\n");
                espi.set_mailbox_status(pe.port, MailboxStatus::WrEmpty);
            }
            if pe.read {
                if pe.spec1 {
                    uprint!(tx, "Mailbox read started.\r\n");
                }
                if pe.spec3 {
                    uprint!(tx, "Mailbox read done.\r\n");
                }
            }
        }
    }
}

// =========================================================================
// SAF virtual flash (port of ExampleFlashOps)
// =========================================================================

async fn handle_flash_request(
    espi: &mut Espi<'_>,
    tx: &mut LpuartTx<'_, Buffered>,
    flash: &mut [u8; FLASH_SIZE],
    queue: &mut ReadQueue,
    req: FlashRequest,
) {
    match req.kind {
        FlashRequestKind::Erase | FlashRequestKind::Write => {
            let addr = req.addr as usize;
            let mut length = req.length as usize;
            if addr >= FLASH_SIZE {
                espi.set_flash_op_len(OMFLEN_SAF_COMPLETION_FAIL, 0);
                espi.set_flash_completion(req.tag, SSTCL_SAF_COMPLETION, SafCompletionType::Only);
                return;
            }
            length = length.min(FLASH_SIZE - addr);
            espi.set_flash_completion(req.tag, SSTCL_SAF_REQ_ACCEPTED, SafCompletionType::Middle);

            if req.kind == FlashRequestKind::Erase {
                uprint!(tx, "[SAF] Erase addr=0x{:08X}, len={}\r\n", req.addr, length);
                flash[addr..addr + length].fill(0xFF);
                espi.set_flash_op_len(OMFLEN_SAF_COMPLETION_NO_DATA, length as u32);
                espi.set_flash_completion(req.tag, SSTCL_SAF_COMPLETION, SafCompletionType::Middle);
            } else {
                let window = espi.flash_window();
                // Write payload starts after the 4 address bytes.
                flash[addr..addr + length].copy_from_slice(&window[4..4 + length]);
                espi.set_flash_op_len(OMFLEN_SAF_COMPLETION_NO_DATA, length as u32);
                espi.set_flash_completion(req.tag, SSTCL_SAF_COMPLETION, SafCompletionType::Middle);

                uprint!(
                    tx,
                    "[SAF] Write addr=0x{:08X}, len={}, first bytes(Hex)=",
                    req.addr,
                    length
                );
                for byte in flash[addr..addr + length].iter().take(8) {
                    uprint!(tx, "{:02X} ", byte);
                }
                uprint!(tx, "\r\n");
            }
        }
        FlashRequestKind::Read => {
            // Promira WAIT_STATE workaround: skip the spurious length-0 pull.
            if req.length == 0 {
                return;
            }
            let max_payload = espi.flash_max_payload();

            if req.read_start {
                let addr = req.addr as usize;
                if addr >= FLASH_SIZE {
                    espi.set_flash_op_len(OMFLEN_SAF_COMPLETION_FAIL, 0);
                    espi.set_flash_completion(req.tag, SSTCL_SAF_COMPLETION, SafCompletionType::Only);
                    return;
                }
                let length = (req.length as usize).min(FLASH_SIZE - addr) as u32;

                espi.set_flash_completion(req.tag, SSTCL_SAF_REQ_ACCEPTED, SafCompletionType::Middle);

                // Split the read into payload-sized completions.
                queue.tag = req.tag;
                queue.cur = 0;
                queue.tot = 0;
                let mut remaining = length;
                let mut offset = 0u32;
                while remaining > 0 && queue.tot < SAF_RX_SPLIT_MAX {
                    let trans_len = remaining.min(max_payload);
                    let rx_type = if length <= max_payload {
                        SafCompletionType::Only
                    } else if remaining == length {
                        SafCompletionType::First
                    } else if remaining == trans_len {
                        SafCompletionType::Last
                    } else {
                        SafCompletionType::Middle
                    };
                    queue.item[queue.tot] = (req.addr + offset, trans_len, rx_type);
                    queue.tot += 1;
                    offset += trans_len;
                    remaining -= trans_len;
                }
            }

            if queue.cur < queue.tot {
                let (addr, len, rx_type) = queue.item[queue.cur];
                let window = espi.flash_window();
                window[..len as usize].copy_from_slice(&flash[addr as usize..(addr + len) as usize]);
                espi.set_flash_op_len(OMFLEN_SAF_COMPLETION_WITH_DATA, len);
                espi.set_flash_completion(queue.tag, SSTCL_SAF_COMPLETION, rx_type);
                queue.cur += 1;
            }

            // Like the SDK example, report the request's own address/length
            // (not the per-chunk values), with the window's first bytes.
            uprint!(
                tx,
                "[SAF] Read addr=0x{:08X}, len={}, first bytes(Hex)=",
                req.addr,
                req.length
            );
            let window = espi.flash_window();
            for byte in window.iter().take((req.length as usize).min(8)) {
                uprint!(tx, "{:02X} ", byte);
            }
            uprint!(tx, "\r\n");
        }
    }
}

// =========================================================================
// Console commands (port of the SDK example's interactive CLI)
// =========================================================================

const VW_FLAG_NAMES: [(&str, VWireOut); 12] = [
    ("dswpwrokrst", VWireOut::DswPwrokRst),
    ("booterrn", VWireOut::BootErrn),
    ("bootdone", VWireOut::BootDone),
    ("e2p", VWireOut::E2p),
    ("susackn", VWireOut::SusAckN),
    ("hostrstack", VWireOut::HostRstAck),
    ("rcinn", VWireOut::Rcinn),
    ("smin", VWireOut::Smin),
    ("scin", VWireOut::Scin),
    ("pmen", VWireOut::Pmen),
    ("wakenscin", VWireOut::WakenScin),
    ("oobrstack", VWireOut::OobRstAck),
];

async fn print_help(tx: &mut LpuartTx<'_, Buffered>) {
    uprint!(tx, "\r\nInteractive commands:\r\n");
    uprint!(tx, " show_config               -- Show eSPI configuration\r\n");
    uprint!(tx, " status                    -- Show eSPI status flags\r\n");
    uprint!(tx, " espi_cap                  -- Show eSPI capabilities\r\n");
    uprint!(tx, " espi_cfg                  -- Show eSPI configurations\r\n");
    uprint!(tx, " wirero                    -- Show vwire read\r\n");
    uprint!(tx, " wirewo                    -- Show vwire write\r\n");
    uprint!(tx, " send_vw_mask <hexmask>    -- Apply VW by mask (32-bit hex)\r\n");
    uprint!(
        tx,
        " send_vw_flag <name> <val> -- Set VW flag by name (val may be multi-bit)\r\n"
    );
    uprint!(tx, " vw_flags                  -- List available VW flag names\r\n");
    uprint!(
        tx,
        " send_oob <hexbytes>       -- Send OOB payload (hex, e.g. AA55 or 0xAA 0x55)\r\n"
    );
    uprint!(tx, " push_irq <num>            -- Push IRQ (0-255) to host\r\n");
    uprint!(tx, " reset_p80                 -- Reset Port 80 counter\r\n");
    uprint!(tx, " help                      -- Help\r\n\r\n");
}

async fn print_vw_flag_list(tx: &mut LpuartTx<'_, Buffered>) {
    uprint!(tx, "Available VWire flags (send_vw_flag <name> <val>):\r\n");
    for (name, wire) in &VW_FLAG_NAMES {
        let multi = *wire == VWireOut::E2p;
        uprint!(
            tx,
            "  {:<12} (bits={}){}\r\n",
            name,
            if multi { 8 } else { 1 },
            if multi { " [multi-bit]" } else { "" }
        );
    }
}

async fn print_espi_config(espi: &Espi<'_>, tx: &mut LpuartTx<'_, Buffered>) {
    let raw = espi.host_config();
    // Decode through the PAC's typed ESPICFG accessors instead of magic shifts.
    let cfg = hal::pac::espi::ESPICFG(raw);
    uprint!(tx, "\n--- Current eSPI Configuration (0x{:08X}) ---\r\n", raw);

    uprint!(
        tx,
        "  IO Mode: {}\r\n",
        match cfg.SPIMOD().to_bits() {
            0 => "Single SPI",
            1 => "Dual SPI",
            2 => "Quad SPI",
            _ => "Reserved",
        }
    );

    uprint!(
        tx,
        "  SPI Speed: {}\r\n",
        match cfg.SPISPD().to_bits() {
            0 => "<=20 MHz",
            1 => "<=25 MHz",
            2 => "<=33 MHz",
            3 => "<=50 MHz",
            4 => "<=66 MHz",
            _ => "Reserved",
        }
    );

    uprint!(
        tx,
        "  CRC Checking: {}\r\n",
        if cfg.CRC() { "Enabled" } else { "Disabled" }
    );
    uprint!(
        tx,
        "  Alert Pin: {}\r\n",
        if cfg.ALERT() { "Dedicated Pin" } else { "MISO" }
    );
    if cfg.ALERT() {
        uprint!(
            tx,
            "  Alert Type: {}\r\n",
            if cfg.ALERTOD() { "Open Drain" } else { "Push-Pull" }
        );
    }

    uprint!(tx, "  Channels:\r\n");
    uprint!(
        tx,
        "    Ch0 (Memory): {}\r\n",
        if cfg.MEMENA() { "Enabled" } else { "Disabled" }
    );
    uprint!(
        tx,
        "    Ch1 (VWire):  {}\r\n",
        if cfg.VWOK() { "Enabled" } else { "Disabled" }
    );
    uprint!(
        tx,
        "    Ch2 (OOB):    {}\r\n",
        if cfg.OOBOK() { "Enabled" } else { "Disabled" }
    );
    uprint!(
        tx,
        "    Ch3 (Flash):  {}\r\n",
        if cfg.FLSHOK() { "Enabled" } else { "Disabled" }
    );

    uprint!(tx, "  Max Payload Sizes:\r\n");
    if cfg.MEMENA() {
        uprint!(tx, "    Memory: {} bytes\r\n", 64u32 << cfg.MEMSZ().to_bits());
    }
    if cfg.OOBOK() {
        uprint!(tx, "    OOB:    {} bytes\r\n", 64u32 << cfg.OOBSZ().to_bits());
    }
    if cfg.FLSHOK() {
        uprint!(tx, "    Flash:  {} bytes\r\n", 64u32 << cfg.FLASHSZ().to_bits());
        uprint!(
            tx,
            "  Flash Erase: {}\r\n",
            match cfg.FLSHERA().to_bits() {
                0 => "Disabled",
                1 => "4 KB",
                2 => "64 KB",
                3 => "4 KB & 64 KB",
                4 => "128 KB",
                5 => "256 KB",
                _ => "Reserved",
            }
        );
    }

    uprint!(
        tx,
        "  SAF Support: {}\r\n",
        if cfg.SAF().to_bits() != 0 { "Yes" } else { "No" }
    );
    uprint!(
        tx,
        "  Bus Master: {}\r\n",
        if cfg.BUSMOK() { "Enabled" } else { "Disabled" }
    );
}

fn parse_u32(s: &str) -> Option<u32> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16).ok()
    } else {
        s.parse().ok()
    }
}

/// Parse hex bytes ("AA55", "AA 55", "0xAA 0x55") into `out`.
fn parse_hex_bytes(s: &str, out: &mut [u8]) -> Option<usize> {
    let mut len = 0;
    for token in s.split_whitespace() {
        let mut token = token
            .strip_prefix("0x")
            .or_else(|| token.strip_prefix("0X"))
            .unwrap_or(token);
        while !token.is_empty() {
            if token.len() < 2 || len >= out.len() {
                return None;
            }
            out[len] = u8::from_str_radix(&token[..2], 16).ok()?;
            len += 1;
            token = &token[2..];
        }
    }
    Some(len)
}

async fn handle_command(espi: &mut Espi<'_>, tx: &mut LpuartTx<'_, Buffered>, input: &str) {
    let mut parts = input.trim().splitn(2, ' ');
    let cmd = parts.next().unwrap_or("");
    let args = parts.next().unwrap_or("").trim();

    match cmd {
        "help" | "h" => print_help(tx).await,
        "show_config" => print_espi_config(espi, tx).await,
        "status" => uprint!(tx, "eSPI status: 0x{:08X}\r\n", espi.status()),
        "espi_cap" => uprint!(tx, "eSPI capabilities: 0x{:08X}\r\n", espi.capabilities()),
        "espi_cfg" => uprint!(tx, "eSPI configuration: 0x{:08X}\r\n", espi.host_config()),
        "wirero" => uprint!(tx, "vwire read: 0x{:08X}\r\n", espi.vwires().0),
        "wirewo" => uprint!(tx, "vwire write: 0x{:08X}\r\n", espi.vwires_out_raw()),
        "send_vw_mask" => match parse_u32(args) {
            Some(mask) => {
                espi.send_vwire_mask(mask);
                uprint!(tx, "VW sends as mask 0x{:08X}\r\n", mask);
            }
            None => uprint!(tx, "Missing mask\r\n"),
        },
        "send_vw_flag" => {
            let mut argv = args.split_whitespace();
            let (name, val) = (argv.next(), argv.next().and_then(parse_u32));
            match (name, val) {
                (Some(name), Some(val)) => {
                    // Accept full SDK constant names too, like the C example.
                    let short = name.strip_prefix("kESPI_VWireWr_").unwrap_or(name);
                    let lookup = VW_FLAG_NAMES
                        .iter()
                        .find(|(flag_name, _)| flag_name.eq_ignore_ascii_case(short));
                    match lookup {
                        Some((_, wire)) => {
                            // 0 = kStatus_Success, 3 = kStatus_Busy, as the C
                            // example prints the raw status_t value.
                            let result = espi.send_vwire(*wire, val as u8);
                            uprint!(
                                tx,
                                "\r\nESPI_SendVWire({}, val={}) -> {}\r\n",
                                name,
                                val,
                                if result.is_ok() { 0 } else { 3 }
                            );
                        }
                        None => {
                            uprint!(tx, "Unknown VW flag name: {}\r\n", name);
                            print_vw_flag_list(tx).await;
                        }
                    }
                }
                _ => {
                    uprint!(tx, "Usage: send_vw_flag <name> <val>\r\n");
                    print_vw_flag_list(tx).await;
                }
            }
        }
        "vw_flags" => print_vw_flag_list(tx).await,
        "send_oob" => {
            let max_oob = espi.oob_port().map(|p| espi.mailbox_size(p)).unwrap_or(0).min(256);
            let mut buf = [0u8; 256];
            match parse_hex_bytes(args, &mut buf) {
                Some(0) | None if args.is_empty() => uprint!(tx, "No OOB data provided\r\n"),
                Some(0) => uprint!(tx, "No OOB data parsed\r\n"),
                Some(len) if len > max_oob => uprint!(tx, "OOB data too long (max {} bytes)\r\n", max_oob),
                Some(len) => {
                    let result = espi.send_oob(&buf[..len], true);
                    uprint!(
                        tx,
                        "ESPI_SendOOB -> {} (len={})\r\n",
                        if result.is_ok() { 0 } else { -1 },
                        len
                    );
                }
                None => uprint!(tx, "Invalid hex input (use two chars per byte, e.g. AA BB)\r\n"),
            }
        }
        "push_irq" => match parse_u32(args) {
            Some(irq) if irq <= 255 => espi.push_irq(irq as u8),
            Some(irq) => uprint!(tx, "Invalid IRQ number: {} (must be 0-255)\r\n", irq),
            None => uprint!(tx, "Missing IRQ number (0-255)\r\nUsage: push_irq <num>\r\n"),
        },
        "reset_p80" => {
            espi.reset_port80_counter();
            uprint!(tx, "Port 80 counter reset.\r\n");
        }
        _ => uprint!(tx, "Unknown command. Type 'help' for help.\r\n"),
    }
}

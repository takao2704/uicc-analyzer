#![no_std]
#![no_main]

mod atr;
mod clk_monitor;
mod io_capture;
mod rst_monitor;

use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver as UsbDriver, InterruptHandler as UsbInterruptHandler};
use embassy_rp::bind_interrupts;
use embassy_time::{Duration, Instant, Timer};
use panic_halt as _;

#[cfg(feature = "pico2w")]
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
#[cfg(feature = "pico2w")]
use embassy_rp::peripherals::{DMA_CH0, PIO0};
#[cfg(feature = "pico2w")]
use embassy_rp::pio::{InterruptHandler as PioInterruptHandler, Pio};
#[cfg(feature = "pico2w")]
use static_cell::StaticCell;

use crate::atr::{AtrMachine, AtrState};
use crate::clk_monitor::ClkMonitor;
use crate::io_capture::{IoCapture, IoSample};
use crate::rst_monitor::{RstLevel, RstMonitor};

const NO_SIGNAL_TIMEOUT_US: u64 = 3_000_000;
const NO_CLK_AFTER_RST_TIMEOUT_US: u64 = 1_500_000;
const NO_ATR_IO_TIMEOUT_US: u64 = 2_000_000;
const IDLE_STATUS_REPEAT_US: u64 = 2_000_000;
const HEARTBEAT_INTERVAL_US: u64 = 2_000_000;
const LOOP_PERIOD_US: u64 = 100;

#[cfg(feature = "pico2w")]
bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
});

#[cfg(not(feature = "pico2w"))]
bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
});

#[derive(Clone, Copy, PartialEq, Eq)]
enum LedMode {
    Idle,
    WaitForAtr,
    Active,
    AlertNoSignal,
}

#[embassy_executor::task]
async fn logger_task(driver: UsbDriver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[cfg(feature = "pico2w")]
#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

fn now_us() -> u64 {
    Instant::now().as_micros()
}

fn log_line(now_us: u64, msg: &str) {
    let ms = now_us / 1_000;
    let frac = now_us % 1_000;
    log::info!("[{}.{:03} ms] {}", ms, frac, msg);
}

#[cfg(feature = "pico2w")]
async fn led_set(led: &mut cyw43::Control<'static>, on: bool) {
    led.gpio_set(0, on).await;
}

#[cfg(not(feature = "pico2w"))]
async fn led_set(led: &mut Output<'_>, on: bool) {
    let level = if on { Level::High } else { Level::Low };
    led.set_level(level);
}

fn led_is_on(mode: LedMode, now_us: u64) -> bool {
    match mode {
        LedMode::Idle => now_us % 1_000_000 < 120_000,
        LedMode::WaitForAtr => now_us % 400_000 < 200_000,
        LedMode::Active => true,
        LedMode::AlertNoSignal => {
            let phase = now_us % 1_200_000;
            phase < 80_000 || (200_000..280_000).contains(&phase)
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let usb_driver = UsbDriver::new(p.USB, Irqs);
    spawner.spawn(logger_task(usb_driver)).unwrap();
    Timer::after_millis(50).await;

    #[cfg(feature = "pico2w")]
    let mut led = {
        let pwr = Output::new(p.PIN_23, Level::Low);
        let cs = Output::new(p.PIN_25, Level::High);
        let mut pio = Pio::new(p.PIO0, Irqs);
        let spi = PioSpi::new(
            &mut pio.common,
            pio.sm0,
            RM2_CLOCK_DIVIDER,
            pio.irq0,
            cs,
            p.PIN_24,
            p.PIN_29,
            p.DMA_CH0,
        );

        static CYW43_STATE: StaticCell<cyw43::State> = StaticCell::new();
        let state = CYW43_STATE.init(cyw43::State::new());
        let (_net_device, mut control, runner) =
            cyw43::new(state, pwr, spi, cyw43_firmware::CYW43_43439A0).await;
        spawner.spawn(cyw43_task(runner)).unwrap();
        control.init(cyw43_firmware::CYW43_43439A0_CLM).await;
        control
            .set_power_management(cyw43::PowerManagementMode::PowerSave)
            .await;
        control
    };

    #[cfg(not(feature = "pico2w"))]
    let mut led = Output::new(p.PIN_25, Level::Low);

    let sim_clk = Input::new(p.PIN_2, Pull::None);
    let sim_rst = Input::new(p.PIN_3, Pull::None);
    let sim_io = Input::new(p.PIN_4, Pull::None);

    let mut rst_monitor = RstMonitor::new();
    let mut clk_monitor = ClkMonitor::new();
    let mut io_capture = IoCapture::new();
    let mut atr = AtrMachine::new();

    let mut saw_clk_log = false;
    let boot_us = now_us();
    let mut saw_bus_activity = false;
    let mut no_clk_warned = false;
    let mut no_atr_warned = false;
    let mut rst_initialized = false;
    let mut rst_released_at_us: Option<u64> = None;
    let mut wait_atr_since_us: Option<u64> = None;
    let mut atr_io_seen = false;
    let mut atr_io_edge_count: u8 = 0;
    let mut io_level_prev = sim_io.is_high();
    let mut last_no_signal_log_us: Option<u64> = None;
    let mut last_heartbeat_us = boot_us;
    let mut led_on = false;
    led_set(&mut led, false).await;

    log_line(boot_us, "boot");
    log_line(boot_us, "uicc-analyzer ready (GPIO2=CLK, GPIO3=RST, GPIO4=IO)");
    #[cfg(feature = "pico2w")]
    log_line(boot_us, "onboard LED active (CYW43 GPIO0)");
    #[cfg(not(feature = "pico2w"))]
    log_line(boot_us, "onboard LED active (GPIO25)");
    log_line(boot_us, "waiting for SIM activity");

    loop {
        let now = now_us();

        let rst_high = sim_rst.is_high();
        if let Some(edge) = rst_monitor.update(rst_high) {
            if !rst_initialized {
                rst_initialized = true;
            }

            match edge.level {
                RstLevel::Low => log_line(now, "RST=LOW"),
                RstLevel::High => log_line(now, "RST=HIGH"),
            }

            if let Some(state) = atr.on_rst_transition(edge.level) {
                match state {
                    AtrState::ResetAsserted => {
                        saw_clk_log = false;
                        io_capture.stop();
                        rst_released_at_us = None;
                        wait_atr_since_us = None;
                        atr_io_seen = false;
                        atr_io_edge_count = 0;
                        no_clk_warned = false;
                        no_atr_warned = false;
                    }
                    AtrState::WaitForClock => {
                        rst_released_at_us = Some(now);
                        wait_atr_since_us = None;
                        atr_io_seen = false;
                        atr_io_edge_count = 0;
                        no_clk_warned = false;
                        no_atr_warned = false;
                        log_line(now, "RST released, checking CLK");
                    }
                    _ => {}
                }
            }
        }

        let clk_level = sim_clk.is_high();
        clk_monitor.sample(now, clk_level);
        if !saw_clk_log && clk_monitor.clock_detected() {
            saw_clk_log = true;
            saw_bus_activity = true;
            log_line(now, "CLK detected");
            if atr.on_clk_activity(true) == Some(AtrState::WaitForAtr) {
                io_capture.start_wait_for_atr();
                wait_atr_since_us = Some(now);
                atr_io_seen = false;
                atr_io_edge_count = 0;
                no_atr_warned = false;
                log_line(now, "waiting for ATR");
            }
        }

        let io_level = sim_io.is_high();
        if io_level != io_level_prev {
            io_level_prev = io_level;
            if wait_atr_since_us.is_some() && !atr_io_seen && atr_io_edge_count < u8::MAX {
                atr_io_edge_count = atr_io_edge_count.saturating_add(1);
                if atr_io_edge_count >= 2 {
                    atr_io_seen = true;
                    saw_bus_activity = true;
                    log_line(now, "IO activity detected while waiting for ATR");
                }
            }
        }

        let sample = if io_level {
            IoSample::High
        } else {
            IoSample::Low
        };
        io_capture.feed_sample(now, sample);

        if !saw_bus_activity && now.saturating_sub(boot_us) >= NO_SIGNAL_TIMEOUT_US {
            let should_log = match last_no_signal_log_us {
                None => true,
                Some(prev) => now.saturating_sub(prev) >= IDLE_STATUS_REPEAT_US,
            };
            if should_log {
                last_no_signal_log_us = Some(now);
                log_line(
                    now,
                    "no signal activity yet (RST/CLK/IO). check SIM power/wiring/connection",
                );
            }
        }

        if now.saturating_sub(last_heartbeat_us) >= HEARTBEAT_INTERVAL_US {
            last_heartbeat_us = now;
            if saw_bus_activity {
                log_line(now, "heartbeat: monitoring bus");
            } else {
                log_line(now, "heartbeat: idle, waiting for SIM activity");
            }
        }

        if let Some(released_at) = rst_released_at_us {
            if !saw_clk_log
                && !no_clk_warned
                && now.saturating_sub(released_at) >= NO_CLK_AFTER_RST_TIMEOUT_US
            {
                no_clk_warned = true;
                log_line(
                    now,
                    "RST released but CLK not detected. SIM may be absent or not driven",
                );
            }
        }

        if let Some(wait_since) = wait_atr_since_us {
            if !atr_io_seen
                && !no_atr_warned
                && now.saturating_sub(wait_since) >= NO_ATR_IO_TIMEOUT_US
            {
                no_atr_warned = true;
                log_line(
                    now,
                    "CLK detected but no IO activity for ATR yet. SIM absent/idle possible",
                );
            }
        }

        let led_mode = if !saw_bus_activity && now.saturating_sub(boot_us) >= NO_SIGNAL_TIMEOUT_US {
            LedMode::AlertNoSignal
        } else if !saw_bus_activity {
            LedMode::Idle
        } else if wait_atr_since_us.is_some() && !atr_io_seen {
            LedMode::WaitForAtr
        } else {
            LedMode::Active
        };

        let next_led_on = led_is_on(led_mode, now);
        if next_led_on != led_on {
            led_on = next_led_on;
            led_set(&mut led, led_on).await;
        }

        Timer::after(Duration::from_micros(LOOP_PERIOD_US)).await;
    }
}

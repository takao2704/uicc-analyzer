#![no_std]
#![no_main]
#![allow(static_mut_refs)]

mod atr;
mod clk_monitor;
mod io_capture;
mod rst_monitor;
mod usb_log;

use cortex_m_rt::entry;
use embedded_hal::digital::v2::InputPin;
use hal::Clock;
use panic_halt as _;
use rp2040_hal as hal;
use usb_device::bus::UsbBusAllocator;

use crate::atr::{AtrMachine, AtrState};
use crate::clk_monitor::ClkMonitor;
use crate::io_capture::{IoCapture, IoSample};
use crate::rst_monitor::{RstLevel, RstMonitor};
use crate::usb_log::UsbLogger;

#[link_section = ".boot2"]
#[used]
pub static BOOT2_FIRMWARE: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

#[entry]
fn main() -> ! {
    let mut pac = hal::pac::Peripherals::take().unwrap();
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let sio = hal::Sio::new(pac.SIO);

    let clocks = hal::clocks::init_clocks_and_plls(
        12_000_000u32,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let sim_clk = pins.gpio2.into_floating_input();
    let sim_rst = pins.gpio3.into_floating_input();
    let sim_io = pins.gpio4.into_floating_input();

    let sys_hz = clocks.system_clock.freq().to_Hz();

    static mut USB_BUS: Option<UsbBusAllocator<hal::usb::UsbBus>> = None;
    let usb_bus = unsafe {
        USB_BUS = Some(UsbBusAllocator::new(hal::usb::UsbBus::new(
            pac.USBCTRL_REGS,
            pac.USBCTRL_DPRAM,
            clocks.usb_clock,
            true,
            &mut pac.RESETS,
        )));
        USB_BUS.as_ref().unwrap()
    };

    let mut logger = UsbLogger::new(usb_bus);
    let mut rst_monitor = RstMonitor::new();
    let mut clk_monitor = ClkMonitor::new();
    let mut io_capture = IoCapture::new();
    let mut atr = AtrMachine::new();

    let mut saw_clk_log = false;

    logger.log(timer.get_counter().ticks(), "boot");

    loop {
        logger.poll();

        let now = timer.get_counter().ticks();

        let rst_high = sim_rst.is_high().unwrap_or(false);
        if let Some(edge) = rst_monitor.update(rst_high) {
            match edge.level {
                RstLevel::Low => logger.log(now, "RST=LOW"),
                RstLevel::High => logger.log(now, "RST=HIGH"),
            }

            if let Some(state) = atr.on_rst_transition(edge.level) {
                match state {
                    AtrState::ResetAsserted => {
                        saw_clk_log = false;
                        io_capture.stop();
                    }
                    AtrState::WaitForClock => logger.log(now, "RST released, checking CLK"),
                    _ => {}
                }
            }
        }

        let clk_level = sim_clk.is_high().unwrap_or(false);
        clk_monitor.sample(now, clk_level);
        if !saw_clk_log && clk_monitor.clock_detected() {
            saw_clk_log = true;
            logger.log(now, "CLK detected");
            if atr.on_clk_activity(true) == Some(AtrState::WaitForAtr) {
                io_capture.start_wait_for_atr();
                logger.log(now, "waiting for ATR");
            }
        }

        // Placeholder sampling hook. This module will be upgraded to PIO/DMA later.
        let io_level = sim_io.is_high().unwrap_or(false);
        let sample = if io_level {
            IoSample::High
        } else {
            IoSample::Low
        };
        io_capture.feed_sample(now, sample);

        cortex_m::asm::delay(sys_hz / 20_000);
    }
}

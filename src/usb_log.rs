use core::fmt::Write;

use heapless::String;
use usb_device::bus::UsbBus;
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
use usbd_serial::SerialPort;

pub struct UsbLogger<'a, B: UsbBus> {
    serial: SerialPort<'a, B>,
    usb_dev: UsbDevice<'a, B>,
}

impl<'a, B: UsbBus> UsbLogger<'a, B> {
    pub fn new(usb_bus: &'a UsbBusAllocator<B>) -> Self {
        let serial = SerialPort::new(usb_bus);
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x1209, 0x0001))
            .device_class(usbd_serial::USB_CLASS_CDC)
            .build();

        Self { serial, usb_dev }
    }

    pub fn poll(&mut self) {
        let _ = self.usb_dev.poll(&mut [&mut self.serial]);
    }

    pub fn log(&mut self, now_us: u64, msg: &str) {
        let ms = now_us / 1_000;
        let frac = now_us % 1_000;

        let mut line: String<128> = String::new();
        let _ = write!(line, "[{}.{:03} ms] {}\r\n", ms, frac, msg);

        // Best-effort write: this is debug output, so dropping bytes is acceptable in v1.
        let _ = self.serial.write(line.as_bytes());
    }
}

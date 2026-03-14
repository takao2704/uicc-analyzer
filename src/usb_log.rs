use core::fmt::Write;

use heapless::{String, Vec};
use usb_device::bus::UsbBus;
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
use usbd_serial::SerialPort;

pub struct UsbLogger<'a, B: UsbBus> {
    serial: SerialPort<'a, B>,
    usb_dev: UsbDevice<'a, B>,
    pending: Option<PendingWrite>,
}

struct PendingWrite {
    data: Vec<u8, 128>,
    offset: usize,
}

impl<'a, B: UsbBus> UsbLogger<'a, B> {
    pub fn new(usb_bus: &'a UsbBusAllocator<B>) -> Self {
        let serial = SerialPort::new(usb_bus);
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x1209, 0x0001))
            .device_class(usbd_serial::USB_CLASS_CDC)
            .build();

        Self {
            serial,
            usb_dev,
            pending: None,
        }
    }

    pub fn poll(&mut self) {
        let _ = self.usb_dev.poll(&mut [&mut self.serial]);
        self.flush_pending();
    }

    pub fn log(&mut self, now_us: u64, msg: &str) {
        let ms = now_us / 1_000;
        let frac = now_us % 1_000;

        let mut line: String<128> = String::new();
        let _ = write!(line, "[{}.{:03} ms] {}\r\n", ms, frac, msg);

        self.flush_pending();
        if self.pending.is_some() {
            return;
        }
        self.write_bytes(line.as_bytes());
    }

    fn flush_pending(&mut self) {
        if let Some(mut pending) = self.pending.take() {
            match self.serial.write(&pending.data[pending.offset..]) {
                Ok(written) if written > 0 => {
                    pending.offset = pending.offset.saturating_add(written);
                    if pending.offset < pending.data.len() {
                        self.pending = Some(pending);
                    }
                }
                _ => {
                    self.pending = Some(pending);
                }
            }
        }
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        match self.serial.write(bytes) {
            Ok(written) if written >= bytes.len() => {}
            Ok(written) => {
                let mut data: Vec<u8, 128> = Vec::new();
                let _ = data.extend_from_slice(bytes);
                self.pending = Some(PendingWrite {
                    data,
                    offset: written,
                });
            }
            Err(_) => {
                let mut data: Vec<u8, 128> = Vec::new();
                let _ = data.extend_from_slice(bytes);
                self.pending = Some(PendingWrite { data, offset: 0 });
            }
        }
    }
}

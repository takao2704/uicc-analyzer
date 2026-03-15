#!/usr/bin/env python3
"""Simple host logger for the Pico USB CDC output."""

from __future__ import annotations

import argparse
from contextlib import nullcontext
from datetime import datetime

import serial
from serial.tools import list_ports
from serial.tools.list_ports_common import ListPortInfo


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Read and print UICC analyzer logs")
    parser.add_argument("port", nargs="?", help="Serial port (e.g. /dev/ttyACM0, /dev/cu.usbmodem*, COM3)")
    parser.add_argument("--baud", type=int, default=115200, help="Baud rate (default: 115200)")
    parser.add_argument("--save", help="Optional file path to append received lines")
    parser.add_argument("--list", action="store_true", help="List available serial ports and exit")
    return parser.parse_args()


def list_serial_ports() -> list[ListPortInfo]:
    return sorted(list_ports.comports(), key=lambda p: p.device)


def is_preferred_port(port: ListPortInfo) -> bool:
    device = port.device.lower()
    description = (port.description or "").lower()
    manufacturer = (port.manufacturer or "").lower()
    hwid = (port.hwid or "").lower()
    searchable = f"{device} {description} {manufacturer} {hwid}"

    # Avoid auto-selecting host debug consoles that are usually unrelated.
    if "debug-console" in searchable:
        return False

    return (
        "usbmodem" in searchable
        or "usbserial" in searchable
        or "ttyacm" in searchable
        or "ttyusb" in searchable
        or device.upper().startswith("COM")
        or port.vid == 0x2E8A  # Raspberry Pi / Pico devices
    )


def pick_default_port() -> str | None:
    ports = list_serial_ports()

    # Prefer USB CDC-like ports by common naming conventions and VID.
    preferred = [p for p in ports if is_preferred_port(p)]
    if preferred:
        return preferred[0].device
    return None


def main() -> None:
    args = parse_args()

    if args.list:
        ports = list_serial_ports()
        if not ports:
            print("No serial ports found")
            return
        for p in ports:
            print(f"{p.device}\t{p.description}\t{p.hwid}")
        return

    port = args.port or pick_default_port()
    if not port:
        ports = list_serial_ports()
        if not ports:
            print("No serial port found. Connect the board and run with --list to inspect ports.")
            return
        print("No preferred USB CDC port found. Specify a port explicitly, for example:")
        print("  python3 tools/serial_logger.py /dev/cu.usbmodemXXXX --baud 115200")
        print("Detected ports:")
        for p in ports:
            print(f"  {p.device}\t{p.description}")
        return

    try:
        output_context = open(args.save, "a", encoding="utf-8") if args.save else nullcontext(None)
        with output_context as out_file, serial.Serial(port, args.baud, timeout=1) as ser:
            # embassy_usb_logger may wait for host control lines.
            ser.dtr = True
            ser.rts = True
            # Nudge OUT endpoint once so both CDC directions are active.
            ser.write(b"\r\n")
            ser.flush()
            print(f"Connected to {port} @ {args.baud} baud")
            while True:
                raw = ser.readline()
                if not raw:
                    continue
                line = raw.decode("utf-8", errors="replace").rstrip()
                stamped = f"{datetime.now().isoformat(timespec='seconds')} {line}"
                print(stamped)
                if out_file:
                    out_file.write(stamped + "\n")
                    out_file.flush()
    except KeyboardInterrupt:
        print("\nStopped by user")


if __name__ == "__main__":
    main()

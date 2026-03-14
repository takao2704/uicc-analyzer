#!/usr/bin/env python3
"""Simple host logger for the Pico USB CDC output."""

from __future__ import annotations

import argparse
from datetime import datetime

import serial
from serial.tools import list_ports


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Read and print UICC analyzer logs")
    parser.add_argument("port", nargs="?", help="Serial port (e.g. /dev/ttyACM0, /dev/cu.usbmodem*, COM3)")
    parser.add_argument("--baud", type=int, default=115200, help="Baud rate (default: 115200)")
    parser.add_argument("--save", help="Optional file path to append received lines")
    parser.add_argument("--list", action="store_true", help="List available serial ports and exit")
    return parser.parse_args()


def list_serial_ports() -> list[str]:
    return [p.device for p in list_ports.comports()]


def pick_default_port() -> str | None:
    ports = list_serial_ports()

    # Prefer USB CDC-like ports by common naming conventions.
    preferred = [
        p
        for p in ports
        if (
            "usbmodem" in p.lower()
            or "usbserial" in p.lower()
            or "ttyacm" in p.lower()
            or "ttyusb" in p.lower()
            or p.upper().startswith("COM")
        )
    ]
    if preferred:
        return preferred[0]
    return ports[0] if ports else None


def main() -> None:
    args = parse_args()

    if args.list:
        ports = list_serial_ports()
        if not ports:
            print("No serial ports found")
            return
        for p in ports:
            print(p)
        return

    port = args.port or pick_default_port()
    if not port:
        print("No serial port found. Connect the board and run with --list to inspect ports.")
        return

    out_file = open(args.save, "a", encoding="utf-8") if args.save else None

    try:
        with serial.Serial(port, args.baud, timeout=1) as ser:
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
    finally:
        if out_file:
            out_file.close()


if __name__ == "__main__":
    main()

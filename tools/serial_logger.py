#!/usr/bin/env python3
"""Simple host logger for the Pico USB CDC output."""

from __future__ import annotations

import argparse
from datetime import datetime

import serial


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Read and print UICC analyzer logs")
    parser.add_argument("port", help="Serial port (e.g. /dev/ttyACM0 or COM3)")
    parser.add_argument("--baud", type=int, default=115200, help="Baud rate (default: 115200)")
    parser.add_argument("--save", help="Optional file path to append received lines")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    out_file = open(args.save, "a", encoding="utf-8") if args.save else None

    try:
        with serial.Serial(args.port, args.baud, timeout=1) as ser:
            print(f"Connected to {args.port} @ {args.baud} baud")
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

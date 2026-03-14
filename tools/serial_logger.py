#!/usr/bin/env python3
"""Simple USB serial logger for the Pico SIM passive analyzer."""

import argparse
import datetime as dt
import sys

import serial


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Read and optionally save analyzer logs")
    parser.add_argument("--port", required=True, help="Serial port (e.g. /dev/ttyACM0 or COM5)")
    parser.add_argument("--baud", type=int, default=115200, help="Baud rate (default: 115200)")
    parser.add_argument("--save", help="Optional file path to append logs")
    parser.add_argument("--timestamp-host", action="store_true", help="Prefix each line with host timestamp")
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    sink = open(args.save, "a", encoding="utf-8") if args.save else None

    try:
        with serial.Serial(args.port, args.baud, timeout=1) as ser:
            print(f"Connected to {args.port} @ {args.baud}")
            while True:
                raw = ser.readline()
                if not raw:
                    continue

                line = raw.decode("utf-8", errors="replace").rstrip("\r\n")
                if args.timestamp_host:
                    host_ts = dt.datetime.now().isoformat(timespec="milliseconds")
                    output = f"{host_ts} {line}"
                else:
                    output = line

                print(output)
                if sink:
                    sink.write(output + "\n")
                    sink.flush()

    except KeyboardInterrupt:
        print("\nStopped by user")
        return 0
    except serial.SerialException as exc:
        print(f"Serial error: {exc}", file=sys.stderr)
        return 1
    finally:
        if sink:
            sink.close()


if __name__ == "__main__":
    raise SystemExit(main())

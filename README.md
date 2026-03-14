# uicc-analyzer

Passive SIM/UICC-to-modem protocol analyzer prototype for Raspberry Pi Pico (RP2040).

## Scope (Prototype v1)

This repository implements a **passive observation-only** foundation:

- Monitor `RST` transitions with timestamps.
- Detect `CLK` activity.
- Capture a first, software-based sample of `IO` activity.
- Build a pipeline for ATR detection and text logging over USB CDC.

Out of scope in this prototype:

- MITM / traffic modification
- SIM emulation
- APDU decoding (future stage)

## Architecture Proposal

### Firmware layers

1. **Signal Monitor Layer** (`main.c`)
   - Configures GPIO inputs for `CLK`, `RST`, and `IO`.
   - Handles interrupt callbacks for `RST` and `CLK` edge observation.
   - Performs simple polling capture for `IO` in the prototype stage.

2. **Timing Layer** (`main.c`)
   - Uses RP2040 microsecond timer (`time_us_64`) to generate relative millisecond timestamps from boot.

3. **Logging Layer** (`logger.c` / `logger.h`)
   - Human-readable text log helpers.
   - USB CDC transport via TinyUSB stdio (`stdio_usb_init`).

4. **ATR Capture Pipeline (prototype skeleton)**
   - Arms ATR capture on `RST` rising edge.
   - Collects candidate bits from IO polling loop and packs into bytes.
   - Emits ATR candidate bytes in hex once enough bytes are accumulated.

### Runtime flow

- Boot -> initialize USB serial + GPIO + interrupts -> print boot line.
- On `RST` edge -> log transition and arm/reset ATR buffer as needed.
- On `CLK` edges -> detect and log first activity event after reset.
- Main loop -> poll IO, append candidate bits while ATR capture is armed.
- Once candidate bytes exist -> print `ATR: <hex bytes>` log line.

## GPIO Mapping (default)

- `GP2` -> `SIM_CLK` input
- `GP3` -> `SIM_RST` input
- `GP4` -> `SIM_IO` input
- `GND` -> shared ground
- USB -> host logging interface

## Build (Pico SDK)

Prerequisites:

- `PICO_SDK_PATH` set to a Pico SDK checkout.
- ARM GCC toolchain installed.

Example:

```bash
cd firmware
mkdir -p build && cd build
cmake ..
make -j
```

## PC Logger

Use `tools/serial_logger.py` to display and optionally save serial logs.

```bash
python3 tools/serial_logger.py --port /dev/ttyACM0 --baud 115200 --save sim.log
```

## Development Roadmap

### Stage 1 — RST Logger
- Done in skeleton: IRQ-based `RST` transition logs with timestamps.
- Success: reliable `RST=LOW/HIGH` events.

### Stage 2 — Clock Detection
- Done in skeleton: first `CLK detected` event and edge counter.
- Next: optional frequency estimate output.

### Stage 3 — IO Raw Capture
- Done in skeleton: software polling of `IO` while ATR capture is armed.
- Next: improve bit sampling quality.

### Stage 4 — ATR Capture
- Done in skeleton: candidate byte packing and ATR hex line output.
- Next: robust frame alignment and stop/parity validation.

### Stage 5 — Stability Improvements
- Planned: PIO-based synchronized sampling, parity checks, timeout recovery.

See `docs/pio_capture_proposal.md` for the PIO capture design.

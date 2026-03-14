# PIO Capture Design Proposal (CLK-Synchronized IO)

## Goal

Improve ATR reliability by sampling `SIM_IO` synchronized to `SIM_CLK` using RP2040 PIO, while keeping the analyzer strictly passive.

## Proposed PIO Strategy

### Inputs

- `SIM_CLK` on GP2
- `SIM_IO` on GP4
- `SIM_RST` stays on CPU GPIO IRQ (GP3)

### PIO state machine concept

1. Wait for clock edge (`wait 1 pin` / `wait 0 pin` sequence).
2. Sample IO pin near stable region of each bit period.
3. Shift sampled bits into ISR.
4. Push packed words to RX FIFO for CPU consumption.

### Data path

- **PIO SM**: bit sampling and packing.
- **DMA (optional in later step)**: move FIFO data to RAM ring buffer.
- **Core 0**: framing (start/parity/stop), ATR byte reconstruction.
- **Core 1 (optional)**: logging and USB text output.

## Framing recommendation for SIM asynchronous bytes

For T=0-style async framing:

- Start bit: `0`
- 8 data bits: LSB first
- Even parity bit
- Stop bit: `1`

Decoder should:

- search for valid start bit transitions
- validate stop and parity
- flag framing/parity errors in logs

## Why this improves reliability

- Sampling tied to observed SIM clock reduces drift versus CPU polling.
- PIO timing is deterministic and minimizes interrupt latency effects.
- FIFO buffering smooths short USB logging stalls.

## Incremental implementation steps

1. Add PIO program that samples IO on selected CLK phase.
2. Validate bitstream against known ATR pattern.
3. Add byte-framing + parity checks in CPU.
4. Add overflow/error counters to logs.
5. Optional: DMA ring buffer and multicore split.

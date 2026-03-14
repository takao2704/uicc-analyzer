#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <elf-path>" >&2
  exit 2
fi

ELF_PATH="$1"
UF2_PATH="${ELF_PATH}.rp2350.uf2"

if ! command -v elf2flash >/dev/null 2>&1; then
  echo "error: elf2flash is not installed. run: cargo install elf2flash" >&2
  exit 1
fi

if [[ ! -f "$ELF_PATH" ]]; then
  echo "error: ELF not found: $ELF_PATH" >&2
  exit 1
fi

echo "Converting ELF to RP2350 UF2..."
elf2flash convert --board rp2350 "$ELF_PATH" "$UF2_PATH"

VOLUME=""
for candidate in /Volumes/RP2350 /Volumes/RPI-RP2 /Volumes/RPI-RP2350; do
  if [[ -d "$candidate" ]]; then
    VOLUME="$candidate"
    break
  fi
done

if [[ -z "$VOLUME" ]]; then
  echo "error: RP2350 UF2 volume not found. Put board in BOOTSEL mode and retry." >&2
  exit 1
fi

echo "Copying UF2 to $VOLUME..."
cp "$UF2_PATH" "$VOLUME"/
sync

echo "Ejecting $VOLUME..."
diskutil eject "$VOLUME" >/dev/null 2>&1 || true

echo "Deploy complete."

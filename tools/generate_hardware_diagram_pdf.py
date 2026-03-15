#!/usr/bin/env python3
"""Generate a simple circuit diagram PDF for the modem-UICC monitor wiring."""

from __future__ import annotations

from pathlib import Path


PAGE_W = 595
PAGE_H = 842


def escape_pdf_text(text: str) -> str:
    return text.replace("\\", "\\\\").replace("(", "\\(").replace(")", "\\)")


class PdfCanvas:
    def __init__(self) -> None:
        self.ops: list[str] = []

    def line(self, x1: float, y1: float, x2: float, y2: float) -> None:
        self.ops.append(f"{x1:.2f} {y1:.2f} m {x2:.2f} {y2:.2f} l S")

    def rect(self, x: float, y: float, w: float, h: float) -> None:
        self.ops.append(f"{x:.2f} {y:.2f} {w:.2f} {h:.2f} re S")

    def text(self, x: float, y: float, size: float, text: str) -> None:
        t = escape_pdf_text(text)
        self.ops.append(f"BT /F1 {size:.2f} Tf 1 0 0 1 {x:.2f} {y:.2f} Tm ({t}) Tj ET")

    def circle_filled(self, x: float, y: float, r: float) -> None:
        # Approximate a circle using 4 cubic Bezier segments.
        k = 0.5522847498 * r
        self.ops.append(
            f"{x + r:.2f} {y:.2f} m "
            f"{x + r:.2f} {y + k:.2f} {x + k:.2f} {y + r:.2f} {x:.2f} {y + r:.2f} c "
            f"{x - k:.2f} {y + r:.2f} {x - r:.2f} {y + k:.2f} {x - r:.2f} {y:.2f} c "
            f"{x - r:.2f} {y - k:.2f} {x - k:.2f} {y - r:.2f} {x:.2f} {y - r:.2f} c "
            f"{x + k:.2f} {y - r:.2f} {x + r:.2f} {y - k:.2f} {x + r:.2f} {y:.2f} c f"
        )

    def build_stream(self) -> bytes:
        stream = "\n".join(self.ops) + "\n"
        return stream.encode("ascii")


def build_diagram_stream() -> bytes:
    c = PdfCanvas()

    # Title
    c.text(45, 804, 15, "uicc-analyzer hardware circuit (monitoring-only)")
    c.text(45, 787, 10, "Purpose: passive tap on modem<->UICC bus (series R on tap branch)")

    # Blocks
    modem_x, modem_y, modem_w, modem_h = 25, 640, 120, 105
    uicc_x, uicc_y, uicc_w, uicc_h = 185, 640, 120, 105
    sh_x, sh_y, sh_w, sh_h = 350, 605, 110, 165
    rp_x, rp_y, rp_w, rp_h = 470, 620, 110, 140

    c.rect(modem_x, modem_y, modem_w, modem_h)
    c.rect(uicc_x, uicc_y, uicc_w, uicc_h)
    c.rect(sh_x, sh_y, sh_w, sh_h)
    c.rect(rp_x, rp_y, rp_w, rp_h)

    c.text(modem_x + 28, modem_y + 88, 10, "MODEM")
    c.text(modem_x + 13, modem_y + 73, 8, "host side")
    c.text(uicc_x + 35, uicc_y + 88, 10, "UICC / SIM")
    c.text(uicc_x + 18, uicc_y + 73, 8, "card side")
    c.text(sh_x + 9, sh_y + 145, 10, "AE-LLCNV8 (FXMA108)")
    c.text(sh_x + 30, sh_y + 130, 8, "OE: active low")
    c.text(rp_x + 3, rp_y + 118, 10, "RP2350 (Pico 2/2W)")
    c.text(rp_x + 16, rp_y + 103, 8, "MCU domain (3.3V)")

    c.text(115, 758, 8, "modem <-> UICC main bus")

    # Main bus and tap branches
    tap_x = 165
    r_x_base = uicc_x + uicc_w + 6  # keep resistor block clearly outside UICC/SIM block
    rows = [
        ("CLK", "GPIO2 (CLK_MON)", "A1", "B1", 710, 690),
        ("RST", "GPIO3 (RST_MON)", "A2", "B2", 680, 660),
        ("IO", "GPIO4 (IO_MON)", "A3", "B3", 650, 630),
    ]

    for idx, (sig, gpio, pin_a, pin_b, bus_y, tap_y) in enumerate(rows):
        branch_x = tap_x + (idx * 4)
        r_x = r_x_base + (idx * 4)

        # Main modem-UICC line (no resistor inline).
        c.text(modem_x + 8, bus_y + 8, 8, f"SIM_{sig}")
        c.line(modem_x + modem_w, bus_y, uicc_x, bus_y)
        c.circle_filled(branch_x, bus_y, 2.2)

        # Tap branch from main line to analyzer (with series resistor on branch only).
        # Route tap branch outside the UICC block (below it), then up to [R].
        route_y = uicc_y - 14 - (idx * 12)
        c.line(branch_x, bus_y, branch_x, route_y)
        r_w = 24
        r_h = 10
        # Near [R], route as: horizontal -> vertical bend -> short horizontal into [R].
        # Keep this bend on the right side of UICC/SIM block.
        leg_x = r_x - 3
        c.line(branch_x, route_y, leg_x, route_y)
        c.line(leg_x, route_y, leg_x, tap_y)
        c.line(leg_x, tap_y, r_x, tap_y)
        c.rect(r_x, tap_y - 5, r_w, r_h)
        c.text(r_x + 5, tap_y - 1, 7, "[R]")
        c.text(r_x - 1, tap_y + 11, 7, "10k-22k")

        c.line(r_x + r_w, tap_y, sh_x, tap_y)
        c.text(sh_x + 7, tap_y + 8, 8, pin_a)

        # Level shifter B-side to RP2350 input.
        c.line(sh_x + sh_w, tap_y, rp_x, tap_y)
        c.line(rp_x, tap_y, rp_x + 66, tap_y)
        c.text(sh_x + sh_w - 18, tap_y + 8, 8, pin_b)
        c.text(rp_x + 6, tap_y + 8, 8, gpio)

    # Supplies
    rail_y = 592
    vcca_x = sh_x + 35
    vccb_x = sh_x + 90
    c.text(vcca_x - 13, sh_y + 8, 8, "VCCA")
    c.text(vccb_x - 11, sh_y + 8, 8, "VCCB")
    c.line(vcca_x, sh_y, vcca_x, rail_y)
    c.line(vccb_x, sh_y, vccb_x, rail_y)

    c.text(uicc_x + 10, rail_y - 4, 9, "SIM_VCC")
    c.line(uicc_x + 56, rail_y, vcca_x, rail_y)
    c.text(rp_x + 74, rail_y - 4, 9, "3V3")
    c.line(vccb_x, rail_y, rp_x + 71, rail_y)

    gnd_y = 572
    c.line(40, gnd_y, 560, gnd_y)
    c.text(265, gnd_y + 8, 8, "GND common")

    # Notes
    c.text(45, 530, 10, "Notes:")
    c.text(55, 514, 9, "- Series resistor is on the tap branch, not on modem-UICC main bus")
    c.text(55, 499, 9, "- Keep tap wires short; add low-cap TVS if needed")
    c.text(55, 484, 9, "- Never power the SIM side from analyzer")

    c.text(45, 452, 9, "See docs/hardware_circuit_design.md for full design notes.")
    return c.build_stream()


def write_pdf(path: Path, stream: bytes) -> None:
    # Minimal, standards-compliant PDF with one page and Helvetica font.
    objects: list[bytes] = []
    objects.append(b"<< /Type /Catalog /Pages 2 0 R >>")
    objects.append(b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>")
    objects.append(
        (
            f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {PAGE_W} {PAGE_H}] "
            "/Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>"
        ).encode("ascii")
    )
    objects.append(
        b"<< /Length " + str(len(stream)).encode("ascii") + b" >>\nstream\n" + stream + b"endstream"
    )
    objects.append(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")

    out = bytearray()
    out.extend(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n")

    offsets: list[int] = [0]
    for i, obj in enumerate(objects, start=1):
        offsets.append(len(out))
        out.extend(f"{i} 0 obj\n".encode("ascii"))
        out.extend(obj)
        out.extend(b"\nendobj\n")

    xref_pos = len(out)
    out.extend(f"xref\n0 {len(objects) + 1}\n".encode("ascii"))
    out.extend(b"0000000000 65535 f \n")
    for off in offsets[1:]:
        out.extend(f"{off:010d} 00000 n \n".encode("ascii"))

    out.extend(
        f"trailer\n<< /Size {len(objects) + 1} /Root 1 0 R >>\nstartxref\n{xref_pos}\n%%EOF\n".encode("ascii")
    )

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(out)


def main() -> None:
    out = Path("docs/hardware_circuit_diagram.pdf")
    write_pdf(out, build_diagram_stream())
    print(f"wrote {out}")


if __name__ == "__main__":
    main()

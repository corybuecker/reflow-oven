---
name: kicad
description: KiCad 9 PCB design expertise covering layout, routing, DRC rules, design review, and pcbnew Python scripting. Use when working with .kicad_pcb or .kicad_sch files, debugging DRC errors, writing KiCad automation scripts, reviewing PCB layouts, or asking about via sizing, trace widths, decoupling placement, ground planes, net labels, or fab output.
---

# SKILL: KiCad 9 — PCB Layout, DRC & Design Review

## When to Load This Skill

Load this skill when the user asks about any of the following:
- PCB layout, routing, or trace placement in KiCad 9
- DRC (Design Rule Check) errors, warnings, or custom constraints
- Review of a `.kicad_pcb` or `.kicad_sch` file for correctness
- Writing or debugging KiCad Python scripts (pcbnew API)
- Via sizing, clearance, copper pour, or zone fill questions
- Gerber/fabrication prep (layer stack, drill files, fab notes)
- Footprint placement strategy (decoupling caps, thermal relief, keepouts)
- Net tie, net label scoping, or hierarchical sheet issues
- Silkscreen / reference designator placement
- Component value notation (IEC 60062) on silkscreen or BOM

---

## KiCad 9 File Format Basics

KiCad 9 uses **S-expression** text files for all design data.

| File | Purpose |
|---|---|
| `*.kicad_pcb` | PCB layout (board outline, copper, silkscreen, fab, courtyard, etc.) |
| `*.kicad_sch` | Schematic sheet |
| `*.kicad_pro` | Project file (DRC rules path, net class config) |
| `*.kicad_dru` | Custom design rules (human-editable constraint overrides) |
| `*.kicad_sym` | Symbol library |
| `*.kicad_mod` | Footprint library entry |

### Reading `.kicad_pcb` S-expressions

Key top-level nodes:
```
(kicad_pcb
  (version ...)
  (setup ...)          ; board setup: grid, DRC constraints
  (net ...)            ; net declarations
  (footprint ...)      ; placed components
  (segment ...)        ; copper traces
  (via ...)            ; vias
  (zone ...)           ; copper pours / fills
  (gr_line ...)        ; graphical lines (board outline = Edge.Cuts)
  (gr_text ...)        ; graphical text (silkscreen notes)
)
```

Each `(footprint ...)` contains:
- `(layer "F.Cu")` — placement side
- `(at X Y ANGLE)` — position
- `(pad N thru_hole/smd ...)` with `(net NET_ID "NET_NAME")`
- `(fp_text reference ...)` and `(fp_text value ...)`

Each `(segment ...)` contains:
- `(start X Y)` `(end X Y)` `(width W)` `(layer "F.Cu")` `(net NET_ID)`

Each `(via ...)` contains:
- `(at X Y)` `(size DRILL_DIA)` `(drill DRILL_DIA)` `(layers "F.Cu" "B.Cu")` `(net NET_ID)`

---

## PCB Layout — Best Practices for 2-Layer Boards

### Layer Usage

| Layer | Purpose |
|---|---|
| `F.Cu` | Component side copper |
| `B.Cu` | Bottom copper (often ground plane) |
| `F.Silkscreen` / `B.Silkscreen` | Reference designators, labels |
| `F.Fab` / `B.Fab` | Assembly drawings (not printed on board) |
| `F.Courtyard` / `B.Courtyard` | Keepout for placement collisions |
| `F.Mask` / `B.Mask` | Solder mask openings |
| `Edge.Cuts` | Board outline — must be a **closed contour** |

### Placement Strategy

**General order:**
1. Place connectors and mounting holes at board edges first (they constrain everything else).
2. Place ICs next, oriented to minimize trace crossing.
3. Place decoupling capacitors **immediately adjacent** to each VDD/AVDD pin — aim for <1mm trace from cap pad to IC power pin, cap GND pad via-stitched directly to ground plane.
4. Place crystals / oscillators close to their load caps and the MCU clock pins; keep them away from high-frequency switching nodes.
5. Place bulk caps near power entry points.
6. Route remaining passives last.

**Decoupling cap placement rules (critical):**
- Cap should be between the power source and the IC pin, not after it.
- Via from cap GND pad should go straight to the ground plane — do not daisy-chain GND traces between caps.
- For ICs with multiple VDD pins (e.g., AVDD and VDD), each pin gets its own cap.
- Typical values: 100 nF (high-freq bypass) in parallel with 1–10 µF (bulk), placed in that order from IC pin outward.

**Thermal considerations:**
- ICs with exposed pads (QFN, DFN): connect exposed pad to ground plane with an array of thermal vias (e.g., 0.3 mm drill, 0.6 mm annular ring, on a ~1.2 mm grid) filled or tented.
- Avoid placing bypass caps on top of the thermal via array — wastes thermal path.

### Routing Guidelines

**Trace widths (general starting points for 1 oz copper):**

| Signal type | Min width | OSH Park minimum |
|---|---|---|
| Low-speed signal (<1 MHz) | 0.15 mm | 0.152 mm (6 mil) |
| High-speed / I2C / SPI | 0.2 mm | 0.152 mm (6 mil) |
| Power (< 500 mA) | 0.3 mm | 0.152 mm (6 mil) |
| Power (500 mA – 2 A) | 0.5–1.0 mm | 0.152 mm (6 mil) |
| High current (> 2 A) | Use PCB trace width calculator: W = (I / (k × ΔT^0.44))^(1/0.725) / thickness | — |

> **OSH Park Two-Layer:** minimum trace width and spacing are both **6 mil (0.152 mm)**. Trace-to-trace clearance minimum is also 6 mil. Board edge keepout is **15 mil (0.381 mm)**.

**Via sizing:**

| Parameter | OSH Park Two-Layer | JLC/generic standard |
|---|---|---|
| Via drill minimum | 0.254 mm (10 mil) | 0.3 mm |
| Via annular ring minimum | 0.127 mm (5 mil) | 0.15 mm |
| Via pad minimum (drill + 2× ring) | 0.508 mm | 0.6 mm |
| Signal via (recommended) | 0.3 mm drill / 0.56 mm pad | 0.3 mm drill / 0.6 mm pad |
| Power via (recommended) | 0.4 mm drill / 0.66 mm pad | 0.4–0.5 mm drill / 0.8 mm pad |
| Micro-via | Not available on standard 2-layer | Not available on standard 2-layer |

> OSH Park finish is **ENIG (gold)**, soldermask is **purple**, copper weight is **1 oz**, board thickness is **1.6 mm**. No selection options — these are fixed for the two-layer service.

**90° corners:** Avoid hard 90° bends — use 45° chamfers or curved arcs. In KiCad 9 the interactive router defaults to 45°.

**Ground plane:**
- On a 2-layer board, dedicate `B.Cu` to a solid ground plane (copper zone, no thermal relief on SMD pads connected to GND).
- Use `(zone (fill (thermal_gap 0) (thermal_bridge_width 0)))` for solid fill on GND.
- Add stitching vias (~5 mm grid) around the board perimeter and under ICs.

**Differential pairs:**
- Route as a matched-length pair using KiCad's differential pair router (shortcut: `/`).
- Keep gap constant; avoid splitting the pair around vias.

---

## DRC — Design Rules & Common Issues

### Understanding the `.kicad_dru` File

Custom rules override global DRC settings. Syntax (KiCad 9):

```lisp
(version 1)

(rule "min_via_drill"
  (constraint hole_size (min 0.3mm))
)

(rule "power_trace_width"
  (constraint track_width (min 0.5mm))
  (condition "A.NetClass == 'Power'")
)

(rule "courtyard_clearance"
  (constraint courtyard_clearance (min 0.1mm))
)

(rule "silkscreen_text_size"
  (constraint text_height (min 0.8mm))
  (constraint text_thickness (min 0.15mm))
  (layer "F.Silkscreen" "B.Silkscreen")
)
```

Conditions can reference:
- `A.NetName`, `B.NetName`
- `A.NetClass`, `B.NetClass`
- `A.Type` (e.g., `'Track'`, `'Via'`, `'Pad'`, `'Zone'`)
- `A.Layer`

### Common DRC Violations & Fixes

| DRC Error | Likely Cause | Fix |
|---|---|---|
| `Clearance violation` | Trace/pad too close to adjacent copper | Reroute or increase clearance in DRC rules |
| `Hole too small` | Via drill below fab minimum | Change via drill to ≥ 0.3 mm |
| `Footprint courtyard missing` | No `F.Courtyard` on footprint | Edit footprint, add courtyard rectangle |
| `Courtyard overlap` | Components placed too close | Increase spacing; check 3D viewer |
| `Unconnected items` | Missing ratsnest connection | Reroute or add missing trace/via |
| `Silkscreen clipped by solder mask` | Silkscreen text over exposed pad | Move reference off pad area |
| `Pad not connected to zone` | Zone not filled, or pad inside zone but no via | Refill zones (B key), check thermal relief |
| `Net conflict` | Two nets with same name on different hierarchical scopes | Fix net label scoping in schematic |
| `Duplicate reference` | Two components share a reference (e.g., R1, R1) | Re-annotate (`Tools → Annotate Schematic`) |
| `Board outline not closed` | Gap in Edge.Cuts contour | Zoom in, snap endpoints on Edge.Cuts |

### Net Label Scoping Rules (KiCad 9 Schematic)

This is a common source of silent connectivity errors:

- **Local net labels** (plain labels): scoped to the **current sheet only**. Two sheets can each have a `VDD` label without being connected.
- **Global labels** (`PWR_FLAG`, explicit global labels): visible across all sheets. Use `Add Global Label` for cross-sheet power nets.
- **Power symbols** (from power library): act as implicit global labels. `VDD`, `GND`, `+3V3` from the power library are global by design.
- **Hierarchical labels + sheet pins**: used to explicitly pass nets between a parent sheet and a child hierarchical sheet.

**Rule of thumb:** If a net must cross a hierarchical sheet boundary, use either a power symbol or a hierarchical label/pin pair. Never rely on matching local label names across sheets.

---

## Python Scripting — pcbnew API (KiCad 9)

### Environment Setup

KiCad 9 ships its own Python interpreter. Run scripts via:
- **KiCad Scripting Console:** `Tools → Scripting Console` inside pcbnew
- **CLI (footprint/board automation):** `python3` with `PYTHONPATH` pointing to KiCad's site-packages

```bash
# Linux/macOS
export PYTHONPATH=/usr/lib/kicad/lib/python3/dist-packages
python3 my_script.py

# macOS (Homebrew or App)
export PYTHONPATH=/Applications/KiCad/KiCad.app/Contents/Frameworks/Python.framework/Versions/Current/lib/python3.x/site-packages
```

### Loading a Board

```python
import pcbnew

board = pcbnew.LoadBoard("my_project.kicad_pcb")
```

### Iterating Footprints

```python
for fp in board.GetFootprints():
    ref = fp.GetReference()
    pos = fp.GetPosition()
    layer = fp.GetLayerName()
    print(f"{ref}: ({pcbnew.ToMM(pos.x):.3f}, {pcbnew.ToMM(pos.y):.3f}) on {layer}")
```

### Iterating Tracks and Vias

```python
for track in board.GetTracks():
    if track.GetClass() == "PCB_VIA":
        via = track
        print(f"Via at ({pcbnew.ToMM(via.GetX()):.2f}, {pcbnew.ToMM(via.GetY()):.2f}), "
              f"drill={pcbnew.ToMM(via.GetDrillValue()):.2f}mm")
    else:
        print(f"Track: width={pcbnew.ToMM(track.GetWidth()):.3f}mm, "
              f"net={track.GetNetname()}, layer={track.GetLayerName()}")
```

### Unit Conversions

KiCad 9 stores everything in **nanometers** internally.

```python
pcbnew.FromMM(1.0)    # → 1000000 (nm)
pcbnew.ToMM(1000000)  # → 1.0 (mm)

# Always convert when setting or reading positions/sizes:
fp.SetPosition(pcbnew.VECTOR2I(pcbnew.FromMM(10.0), pcbnew.FromMM(20.0)))
```

### Moving / Placing a Footprint

```python
fp = board.FindFootprintByReference("C1")
fp.SetPosition(pcbnew.VECTOR2I(pcbnew.FromMM(15.5), pcbnew.FromMM(22.0)))
fp.SetOrientationDegrees(90)
```

### Creating a Via

```python
via = pcbnew.PCB_VIA(board)
via.SetPosition(pcbnew.VECTOR2I(pcbnew.FromMM(10), pcbnew.FromMM(10)))
via.SetDrill(pcbnew.FromMM(0.3))
via.SetWidth(pcbnew.FromMM(0.6))
via.SetLayerPair(pcbnew.F_Cu, pcbnew.B_Cu)
via.SetNet(board.FindNet("GND"))
board.Add(via)
```

### Running DRC Programmatically

```python
drc = pcbnew.DRC_ENGINE()
drc.InitEngine(board)
# KiCad 9 DRC API is limited from scripting console;
# prefer running DRC via GUI or kicad-cli for full results
```

### Using kicad-cli (Command Line Interface — KiCad 9)

KiCad 9 ships `kicad-cli` for headless operations:

```bash
# Export Gerbers
kicad-cli pcb export gerbers \
  --output ./gerbers/ \
  --layers F.Cu,B.Cu,F.Silkscreen,B.Silkscreen,F.Mask,B.Mask,Edge.Cuts \
  my_project.kicad_pcb

# Export drill files
kicad-cli pcb export drill \
  --output ./gerbers/ \
  --format excellon \
  my_project.kicad_pcb

# Run DRC and output report
kicad-cli pcb drc \
  --output drc_report.json \
  --format json \
  my_project.kicad_pcb

# Export BOM from schematic
kicad-cli sch export bom \
  --output bom.csv \
  my_project.kicad_sch
```

### Saving Changes

```python
board.Save("my_project.kicad_pcb")         # overwrite
board.SaveAs("my_project_modified.kicad_pcb")  # save copy
pcbnew.Refresh()  # update GUI if running inside scripting console
```

---

## Design Review — Analysis Checklist

When the user asks Claude to review a PCB design (by sharing file content, screenshots, or describing their layout), work through this checklist systematically and report findings by severity.

### Severity Levels
- 🔴 **Critical** — Will likely cause board failure or fab rejection
- 🟠 **Major** — Functional risk; should fix before ordering
- 🟡 **Minor** — Best practice violation; low risk but worth noting
- 🟢 **Pass** — Meets requirements

### Power & Decoupling
- [ ] Every IC VDD/AVDD pin has a dedicated 100 nF cap within 1 mm
- [ ] Bulk capacitance present near power entry
- [ ] Decoupling cap GND pad connects directly to ground plane via (not daisy-chained)
- [ ] Power traces sized for expected current (use trace width calculator)
- [ ] No long power traces sharing a width with signal traces

### Ground Plane
- [ ] Solid ground plane on B.Cu (for 2-layer)
- [ ] No large unintentional splits or slots in ground plane under high-frequency circuits
- [ ] Stitching vias present at board perimeter (~5 mm spacing)
- [ ] Thermal relief disabled on SMD GND pads (use solid fill)
- [ ] Thermal vias present under exposed-pad ICs (QFN, DFN)

### Via Geometry
- [ ] All vias meet fab minimum drill (OSH Park: ≥ 0.254 mm / 10 mil; generic: ≥ 0.3 mm)
- [ ] Annular ring ≥ 0.127 mm for OSH Park (pad radius − drill radius); ≥ 0.15 mm generic
- [ ] No via-in-pad on fine-pitch SMD unless via is filled/capped (call out on fab notes)
- [ ] Signal vias consistent size; power vias larger (0.4–0.5 mm drill)

### Trace Routing
- [ ] No unrouted nets (ratsnest clear)
- [ ] No 90° corners (use 45° or arcs)
- [ ] No acute angles (<45°) — acid traps
- [ ] Differential pairs routed together, matched length
- [ ] High-speed signals not routed parallel to each other for long runs (crosstalk)
- [ ] Return path not interrupted under high-speed traces

### Clearances
- [ ] Trace-to-trace clearance ≥ fab minimum (OSH Park: 0.152 mm / 6 mil; generic: 0.1–0.15 mm)
- [ ] Trace-to-board-edge clearance ≥ 0.381 mm (OSH Park 15 mil keepout) / ≥ 0.3 mm (generic)
- [ ] Copper-to-board-edge clearance ≥ 0.381 mm (OSH Park) / ≥ 0.3 mm (generic)
- [ ] Pad-to-pad clearance on fine-pitch ICs verified

### Board Outline
- [ ] Edge.Cuts is a **closed contour** (no gaps)
- [ ] All mechanical holes present (mounting, connectors)
- [ ] Board dimensions match spec

### Silkscreen
- [ ] Reference designators readable (height ≥ 0.8 mm, thickness ≥ 0.15 mm)
- [ ] No silkscreen over exposed pads or vias (causes DRC warning, solder issues)
- [ ] Component polarity indicators present (diodes, electrolytic caps, ICs pin 1)
- [ ] Connector pin 1 / orientation marked
- [ ] IEC 60062 notation used for component values where applicable

### Fabrication Notes
- [ ] Layer stack documented (e.g., 2-layer, 1 oz Cu, ENIG vs HASL)
- [ ] Controlled impedance callouts present if required
- [ ] Silkscreen color, board color specified if non-default

---

## Common Pitfalls & Lessons Learned

### I2C Pull-Up Placement
Net labels for I2C pull-up resistors must be placed **at the resistor pad connected to the bus**, not at the power supply end. Placing the label on the VDD end shorts the bus to VDD. Verify: resistor top pin → VDD power symbol; resistor bottom pin → net label matching the IC SDA/SCL pad net.

### TVS Diode Orientation
TVS diodes in clamping circuits: cathode to the protected signal/rail, anode to GND. Reversed wiring (anode to VDD) creates a forward-bias path that will source current from VDD instead of clamping the signal.

### In-Circuit Resistance Measurement
Measuring resistance in-circuit will read lower than the component value because parallel paths through other components create alternate current paths. Always power off and isolate or measure out-of-circuit for accurate readings.

### Zone Fill After Changes
Any time copper zones (ground planes) are modified or new traces are added, **refill zones** before running DRC (KiCad shortcut: `B`). Unfilled zones show ratsnest connections that appear unrouted and generate false DRC errors.

### Hierarchical Sheet Net Scoping
Local net labels do not cross hierarchical sheet boundaries. If a net appears connected on paper (same label name, different sheets) but DRC reports an unconnected net, the fix is to use a global label or hierarchical label/pin pair — not to add the same local label to both sheets.

### STATUS Register / Input Range Bit (MCP9601 and similar ADCs)
Some thermocouple/ADC ICs have an input range protection bit in a STATUS register that blocks temperature output if the differential input voltage falls outside a protected range (e.g., MCP9601 bit 4: VSENSE must be within 10–19% of VDD). Validate hardware biasing meets the IC's input range requirements; add this check to bring-up procedures.

---

## Quick Reference — KiCad 9 Keyboard Shortcuts (pcbnew)

| Action | Shortcut |
|---|---|
| Route track | `X` |
| Route differential pair | `\` |
| Add via (while routing) | `V` |
| Refill all zones | `B` |
| Run DRC | `Inspect → Design Rules Checker` |
| Inspect net highlight | `` ` `` (backtick) |
| Open properties | `E` |
| Flip to other side | `F` |
| Mirror | `M` (with item selected) |
| Rotate 90° | `R` |
| Select connected tracks | `U` |
| Select entire net | `I` |
| 3D Viewer | `Alt+3` |
| Scripting Console | `Tools → Scripting Console` |

---

## Output Format Guidelines

### When producing a design review (text analysis):
- Lead with a **summary table** of findings by severity
- Group findings by category (Power, Ground, Vias, Routing, Silkscreen, etc.)
- For each finding: state what was found, why it matters, and the specific fix
- End with a fabrication readiness verdict: **Ready / Fix Required / Major Rework**

### When producing Python scripts:
- Always include unit conversion comments (`# mm → nm`)
- Include a `board.Save()` call with a distinct output filename to avoid overwriting
- Add a dry-run mode (`DRY_RUN = True`) that prints changes without applying them
- Print a summary at the end: components moved, vias added, nets modified

### When producing `.kicad_dru` custom rules:
- Include a comment block at the top with the rule's intent, affected nets/layers, and the fab it targets (e.g., JLCPCBStandard)
- Validate that rule conditions reference valid KiCad 9 property names (`A.NetName`, not `net_name`)
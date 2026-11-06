# Component Category Taxonomy & Parametric Schema

Type: grilling
Status: resolved
Blocked by: none

## Question

What is the exact taxonomy of audio electronics component categories and their structured parametric attributes, how are electrical values normalized and searched (e.g., `4k7` vs `4.7kΩ` vs `4700`), and how are physical storage locations (bins, drawers, binders) modeled?

## Answer

### 1. Component Category Taxonomy
Nine built-in core categories (plus `Other / Custom`), each with a `subtype` field:
1. **Resistors** (Fixed, trimmer/preset, arrays/networks)
2. **Capacitors** (Film, ceramic, aluminum electrolytic, tantalum, mica, trimmer)
3. **Semiconductors — Discrete** (Diodes, Zeners, Schottky, LEDs, BJTs, JFETs, MOSFETs, matched pairs/quads)
4. **Integrated Circuits (ICs)** (Op-amps, logic, delay/BBD/PT2399, voltage regulators, specialized audio/synth ICs, optocouplers/vactrols)
5. **Potentiometers & Encoders** (Rotary pots, slide pots, trimmers, rotary encoders)
6. **Switches, Jacks & Connectors** (Footswitches, toggle/push/rotary/DIP switches, 1/4" & 3.5mm audio jacks, DC power jacks, headers/sockets)
7. **Inductors, Transformers & Magnetics** (Audio transformers, power transformers, inductors/chokes, wah inductors)
8. **Enclosures, Knobs & Mechanical Hardware** (Enclosures, knobs, standoffs, fasteners, PCBs/stripboard, wire)
9. **Modules & Sub-Assemblies** (MCU boards like Daisy Seed / Teensy / Pico, display modules, pre-assembled daughterboards, reverb bricks like Belton BTDR)

### 2. Hybrid Parametric Schema (`components` table)
- **First-class indexed SQL columns**:
  - `id`, `name`, `category`, `subtype`, `package_or_mounting` (e.g., `Through-Hole`, `SMD 0805`, `DIP-8`, `TO-92`, `16mm PCB`, `Module`)
  - `raw_spec_text` (stores the exact user-entered or BOM-imported specification string verbatim, preserving combined details like `100kA 16mm` or `4k7 1% 1/4W`)
  - `canonical_value_display` (formatted canonical string, e.g., `4.7kΩ`, `47nF`)
  - Normalized numeric columns for exact comparison and range sorting: `resistance_ohms` (`REAL`), `capacitance_pf` (`REAL`), `inductance_uh` (`REAL`), `voltage_rating_volts` (`REAL`)
  - `low_stock_threshold` (`INTEGER`), `datasheet_url` (`TEXT`), `notes` (`TEXT`)
- **Validated `specs_json` column** (queried via SQLite `json_extract`):
  - **Resistors**: `tolerance`, `power_rating_watts`, `composition`
  - **Capacitors**: `dielectric`, `tolerance`, `polarized`, `lead_spacing_mm`
  - **Semiconductors — Discrete**: `part_code`, `polarity_or_channel`, `key_spec` (e.g., Zener V, LED color/size, measured $h_{FE}$ / $V_{GS(off)}$)
  - **Integrated Circuits (ICs)**: `part_code`, `function`, `channel_or_pin_count`
  - **Potentiometers & Encoders**: `taper` (`A`, `B`, `C`, `W`), `body_size`, `shaft_type`, `gangs`, `has_switch`, `center_detent`
  - **Switches, Jacks & Connectors**: `configuration` (`3PDT`, `SPDT On-On`, `TRS Switched`, etc.), `actuation`, `size_or_pitch`
  - **Inductors, Transformers & Magnetics**: `value_or_ratio`, `current_or_power_rating`
  - **Enclosures, Knobs & Mechanical Hardware**: `size_or_form`, `finish_or_color`, `shaft_fit`
  - **Modules & Sub-Assemblies**: `model_or_platform`, `interface_or_voltage`, `form_factor_or_pinout`

### 3. Electrical Value Normalization & Verbatim Preservation
- A deterministic Rust parser parses shorthand (`4k7`, `4.7k`, `4700R`, `47n`, `0.047uF`, `4n7`, `100kA`, `B50k`) on both component/BOM entry and search queries into canonical numeric columns (`resistance_ohms`, `capacitance_pf`, `inductance_uh`, `voltage_rating_volts`) and extracted parametric attributes (`taper`, `tolerance`, `power_rating_watts`).
- Capacitance values in the `1nF–999nF` range format canonically as `nF` by default.
- The original `raw_spec_text` is always preserved and displayed alongside the canonicalized value so extra nuances in the spec string are never lost.

### 4. Hierarchical `Storage Location` & Multi-Location `Stock Lot`
- **`Storage Location`** (`storage_locations` table): Self-referential tree (`id`, `name`, `kind` [`Crate`, `Box`, `Binder`, `Drawer`, `Bag`, `Other`], `parent_id`, `notes`). Moving a bag or box updates `parent_id` and automatically updates the full location breadcrumb path for all nested child locations, **Stock Lots**, and **Kit Builds**.
- **`Stock Lot`** (`stock_lots` table): Tracks a physical stash of a **Component** at a specific **Storage Location** (`id`, `component_id`, `location_id`, `quantity`, `supplier`, `supplier_part_no`, `lot_notes`).
  - A **Component**'s total quantity on hand is `SUM(stock_lots.quantity)`.
  - Allows duplicate bags of the same component to live in multiple crates/bags simultaneously.
  - Supports direct inline `+1 / -1` adjustment when a component has a single lot, per-lot selection when multiple lots exist, and a **Merge Lots** operation when physically combining duplicate bags into one location.

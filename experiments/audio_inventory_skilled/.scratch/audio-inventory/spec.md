# Audio Electronics Inventory & DIY Kit Build Tracker Specification

Type: spec
Status: resolved
Labels: ready-for-agent

## Problem Statement

A hobbyist audio electronics builder who builds guitar pedals, synthesizer modules, and amplifiers accumulates hundreds or thousands of electronic **Components** stored across nested physical containers (crates, bags, boxes, binders, and drawers), with duplicate stashes of the same part frequently living in multiple containers. At the same time, the builder tracks multiple **Kit Builds** across different stages—from wishlisted schematics to active kitting and bench debugging.

Without a purpose-built tool, the builder faces several recurring friction points:

1. **Inconsistent electrical shorthand and parametric search**: Schematics, supplier invoices, and personal spreadsheets write the same value in incompatible ways (`4k7`, `4.7kΩ`, `4700R`, or `0.047uF` vs. `47nF` vs. `47n`), making text-only search unreliable while rigid forms lose nuances from the original specification string (`100kA 16mm PCB` or `4k7 1% 1/4W Metal Film`).
2. **Physical container dispersion and duplicate bags**: Parts live inside nested **Storage Locations** (e.g., an anti-static bag inside a category bag inside a crate) that get rearranged as a unit, and duplicate bags of the same **Component** often exist in multiple places until physically consolidated.
3. **Ambiguity between drawer counts, reserved stock, and kitted project boxes**: When planning multiple **Kit Builds**, it is hard to know whether **Available Stock** covers all planned builds, which parts are merely `Reserved` in place inside their storage drawers, which parts have been physically `Picked` into a **Kit Build**'s project box (and which specific **Stock Lot** they came from if the build is later shelved), and which parts came `Included in Kit` from a partial kit vendor.
4. **Real-world BOM substitutions and split allocations**: During kitting or bench assembly, a builder often substitutes an equivalent part (e.g., `2N5457` for `J201`) or splits a multi-quantity **BOM Line** across two different **Stock Lots** or **Components** without wanting to overwrite the original schematic specification.
5. **Tedious BOM entry and multi-build supplier ordering**: Manually entering supplier order histories or KiCad/spreadsheet **BOMs** one line at a time is slow, and calculating combined **Net Shortfall** across multiple queued **Kit Builds** (plus low-stock bench staples) to produce supplier quick-order files for Mouser, DigiKey, Tayda, or LCSC is error-prone.
6. **Fragmented bench documentation and troubleshooting notes**: During assembly and debugging, builders need an interactive per-placement soldering checklist (`R1`, `R2`, `C4`) organized by board profile height alongside schematic PDFs, drill templates, voltage measurements, and modifications.

## Solution

Build a local-first, single-user web application powered by a single self-contained Rust binary (`axum` + `maud` compile-time HTML/HTMX + embedded local `turso` SQLite database) that manages the builder's **Component** catalog, hierarchical **Storage Locations**, multi-location **Stock Lots**, **Kit Builds** and their **BOM Lines**, staged CSV imports, live multi-build **Shopping Lists**, and active bench checklists and **Bench Log Entries**.

The application provides:

- **Deterministic Spec Normalization with Verbatim Preservation**: A shared spec parser normalizes electrical shorthand (`4k7` → `4700 Ω`, `0.047uF` → `47nF`, `100kB` → `100kΩ` + `B` taper) on both catalog/BOM entry and live search queries, while always preserving and displaying the verbatim `raw_spec_text` alongside structured category attributes.
- **Self-Referential Storage Hierarchy & Multi-Lot Balances**: A recursive tree of **Storage Locations** (`Crate`, `Box`, `Binder`, `Drawer`, `Bag`, `Other`) that updates nested breadcrumb paths automatically when a container is moved, paired with **Stock Lots** that track per-location quantities, inline single-lot steppers, multi-lot inspection, and 1-click lot merging.
- **5-State BOM Fulfillment & Reversible Kitting**: Every **BOM Line** tracks its schematic specification, an optional catalog **Component** link (with substitution notes and 1-click line splitting), and a 5-state **Fulfillment Status** (`Unallocated`, `Reserved` in place, `Picked` into the **Kit Build**'s project box with `source_lot_id` tracking, `Installed`, and `Included in Kit`).
- **Two-Step Staged CSV/Paste Imports**: Interactive staging and reconciliation workflows for both **Component**/**Stock Lot** batches (with `Exact Match`, `Near / Ambiguous Match`, and `No Match` classification) and **Kit Build** **BOMs** (with automatic reference-designator category inference and identical-spec row grouping).
- **Live Multi-Build Shopping List & Order Receiving**: Real-time aggregation of unfulfilled **BOM Line** demand across selected **Kit Builds** before subtracting **Available Stock** to compute **Net Shortfall**, optional low-stock restock top-ups, Full and Supplier Quick-Order CSV/clipboard exports, and 1-click receiving of arrived lines directly into **Stock Lots**.
- **Dense HTMX Workbench & Active Bench View**: A fast, keyboard- and mouse-friendly tabular UI with a right-hand context Inspector Drawer, native `<dialog>` substitution modal, stage-grouped soldering checklist with tappable reference-designator chips, local schematic/photo attachment storage, timestamped **Bench Log Entries**, and 1-click `.zip` backup/restore.

## User Stories

### Component Catalog & Parametric Search

1. As a hobbyist builder, I want to catalog **Components** across nine built-in audio electronics categories (`Resistors`, `Capacitors`, `Semiconductors — Discrete`, `Integrated Circuits (ICs)`, `Potentiometers & Encoders`, `Switches, Jacks & Connectors`, `Inductors, Transformers & Magnetics`, `Enclosures, Knobs & Mechanical Hardware`, and `Modules & Sub-Assemblies`, plus `Other / Custom`) with a subtype and mounting/package field, so that every passive part, active device, hardware item, and MCU/reverb module has an appropriate home.
2. As a hobbyist builder, I want to enter a **Component** specification using natural electronics shorthand (such as `4k7 1% 1/4W Metal Film`, `0.047uF 63V Box Film 5mm`, or `100kA 16mm PCB`) and have the app automatically extract canonical numeric values (`4700 Ω`, `47nF`, `100kΩ`) and structured attributes while keeping my exact `raw_spec_text` intact, so that I never lose original part nuances when values are normalized.
3. As a hobbyist builder, I want capacitance values between `1nF` and `999nF` to format canonically as `nF` (e.g., `0.047uF` and `47000pF` both display canonically as `47nF`), so that audio capacitor values are consistent across the catalog and **BOMs**.
4. As a hobbyist builder, I want to type shorthand queries (`4k7`, `4700R`, `4.7k`, `0.047uF`, `47n`, `100kB`, `TL072`) into a debounced catalog search bar and immediately see a live parser interpretation badge showing how my query was parsed, so that I can verify the exact electrical value and attributes being matched.
5. As a hobbyist builder, I want to filter **Components** by category and dynamically filter on category-specific parametric attributes (such as tolerance, wattage, and composition for Resistors; dielectric, voltage rating, polarization, and lead spacing for Capacitors; taper, body size, and shaft type for Potentiometers; part code and channel/pin count for ICs; configuration and actuation for Switches & Jacks; or model/platform and interface voltage for Modules & Sub-Assemblies), so that I can quickly narrow down parts during circuit design or kitting.
6. As a hobbyist builder, I want to record a `low_stock_threshold`, `datasheet_url`, and general notes on any **Component**, so that I am warned when bench staples run low and can open datasheets in one click.

### Hierarchical Storage Locations & Multi-Location Stock Lots

7. As a hobbyist builder, I want to organize my physical containers as a self-referential tree of **Storage Locations** with a container kind (`Crate`, `Box`, `Binder`, `Drawer`, `Bag`, `Other`), so that I can model real-world nesting like `Crate 2 > Passives Bag > Film Caps Sub-bag` or `Binder A > Op-Amps Page`.
8. As a hobbyist builder, I want to move a **Storage Location** to a different parent container and have the full location breadcrumb path automatically update for all nested child **Storage Locations**, **Stock Lots**, and **Kit Builds**, so that rearranging my workshop bags or boxes requires only a single update.
9. As a hobbyist builder, I want to filter the **Component** catalog by any **Storage Location** and see all **Components** stored inside that container or any of its nested child containers, so that I can audit what is inside a specific crate or binder.
10. As a hobbyist builder, I want a **Component**'s quantity on hand to be tracked as one or more **Stock Lots** at specific **Storage Locations** (with optional lot-level supplier, supplier part number, and notes), so that I can accurately represent duplicate bags of the same part living in different crates or drawers.
11. As a hobbyist builder, I want to see both **Available Stock** (`total on hand minus Reserved in-place quantities`) and total physical quantity on hand for every **Component** in the catalog table, so that I know how many parts are physically in my drawers versus already spoken for by planned **Kit Builds**.
12. As a hobbyist builder, I want an inline `− / qty / +` stepper directly in the catalog row when a **Component** lives in a single **Stock Lot**, so that I can adjust single-location part counts in one click without opening a modal or secondary panel.
13. As a hobbyist builder, I want clicking a **Component** with multiple **Stock Lots** to open the right-hand Inspector Drawer listing every **Storage Location** lot with its own `− / qty / +` stepper, so that I never accidentally deduct from the wrong physical bag or drawer.
14. As a hobbyist builder, I want a 1-click **"Merge Duplicate Stock Lots"** action in the Inspector Drawer (and automatic lot merging whenever stock is moved or received into a **Storage Location** that already holds a lot for that **Component**), so that physically combining two bags of the same resistor cleanly consolidates their quantities into a single **Stock Lot**.

### Kit Builds, BOM Matching & Stock Allocation

15. As a hobbyist builder, I want to create and track **Kit Builds** across six lifecycle statuses (`Wishlist / Planned`, `Sourcing / Waiting on Parts`, `Ready to Build (Kitted)`, `On the Bench (In Progress)`, `Boxing / Troubleshooting`, and `Completed`, plus `Shelved / Abandoned`), so that I can see the state of every pedal, synth module, or amp project on my bench.
16. As a hobbyist builder, I want to assign a physical **Storage Location** to a **Kit Build** (such as `Project Box #4` inside `Crate 1`), so that I know where the PCB, enclosure, and picked parts for that build are physically stored.
17. As a hobbyist builder, I want each **BOM Line** on a **Kit Build** to store its schematic specification (`raw_spec_text`, `category`, normalized electrical values, `quantity_required`, `reference_designators`, and notes) alongside an optional link to a catalog **Component**, so that I can enter or import a **BOM** before every part exists in my catalog.
18. As a hobbyist builder, I want newly created or imported **BOM Lines** to automatically link to a catalog **Component** when a single exact parametric match exists, and to stay `Unlinked` with ranked candidate matches (plus a 1-click **"Create Component from BOM Line"** button) when multiple or zero catalog **Components** match, so that unambiguous parts link automatically while ambiguous specs never silently pick the wrong part.
19. As a hobbyist builder, I want to mark any **BOM Line** as `Included in Kit` when a partial kit vendor supplies the PCB, pre-drilled enclosure, or rare IC inside the kit box, so that the line counts as ready for building without linking to or deducting from my catalog **Stock Lots**.
20. As a hobbyist builder, I want to transition a **BOM Line** to `Reserved` (soft-reserved in place) to hold `quantity_allocated` against my catalog stock without physically removing parts from their storage drawers, so that **Available Stock** for other builds reflects the reservation while physical **Stock Lot** counts still match what is sitting in my drawers.
21. As a hobbyist builder, I want to transition a **BOM Line** to `Picked` when I physically pull parts from a **Stock Lot** and place them into the **Kit Build**'s project box, automatically deducting `quantity_allocated` from the chosen `source_lot_id` and recording `source_lot_id` on the **BOM Line**, so that my storage drawer counts decrease immediately and the app remembers which lot the parts came from.
22. As a hobbyist builder, I want to un-pick a `Picked` **BOM Line** (or transition a **Kit Build** to `Shelved / Abandoned` and confirm returning parts) to automatically return `Picked` quantities back to their original `source_lot_id` and release `Reserved` quantities, while leaving `Installed` (soldered) parts consumed, so that abandoning or shelving a kitted build cleanly restores my inventory without manual math.
23. As a hobbyist builder, I want batch kitting actions (`"Reserve All Available"` and `"Pick All Reserved to Project Box"`) at the top of a **Kit Build**'s **BOM**, so that I can allocate or kit an entire build in one click.
24. As a hobbyist builder, I want row-level **Fulfillment Status** changes on a **BOM Line** to update both that table row and the **Kit Build**'s header readiness summary (`N/M Lines Ready · K Shortfalls`) immediately via HTMX out-of-band swap, so that I always see live build readiness without full page reloads.

### Substitutions, Partial Allocations & Kit Build Cloning

25. As a hobbyist builder, I want to substitute any catalog **Component** onto a **BOM Line** via a native modal dialog and record an optional `substitution_note` without overwriting the schematic's original `raw_spec_text`, so that both the original schematic callout (e.g., `J201`) and the actual part used (e.g., `2N5457 — pinout rotated`) remain visible side by side.
26. As a hobbyist builder, I want to partially allocate a **BOM Line** (`0 < quantity_allocated < quantity_required`) when I only have some of the required parts on hand, so that the allocated portion is held for my build while the remaining shortfall (`quantity_required - quantity_allocated`) automatically flows to the **Shopping List**.
27. As a hobbyist builder, I want a 1-click **"Split BOM Line"** action in the Inspector Drawer that divides a multi-quantity **BOM Line** (e.g., `4x 100nF: C1, C2, C3, C4`) by quantity and reference designators into two flat **BOM Lines** (e.g., `2x C1, C2` and `2x C3, C4`), so that I can fulfill half of the placements from one **Stock Lot** or substitute **Component** and half from another without complex nested sub-tables.
28. As a hobbyist builder, I want to clone an existing **Kit Build** via a 1-click **"Clone Kit Build"** action that copies its metadata, external links, shared schematic/build-doc attachment references, and all **BOM Lines** (preserving `raw_spec_text`, `reference_designators`, `quantity_required`, linked `component_id`, `substitution_note`, and `Included in Kit` flags) while resetting status to `Wishlist / Planned`, clearing allocations (`Unallocated`), and starting with an empty **Bench Log Entry** stream, so that I can build a second copy of a pedal or dual-channel synth module effortlessly.

### Active Bench Checklist, Attachments & Bench Log Entries

29. As a hobbyist builder working on an active build, I want an **Active Bench Checklist** view that groups **BOM Lines** by assembly stage / component profile height (`Stage 1: Low-Profile Resistors & Diodes`, `Stage 2: IC Sockets & Film Capacitors`, `Stage 3: Electrolytics, Pots & Off-Board Hardware`) with an overall placement progress bar (`X / Y Placements Soldered`), so that I can populate the PCB in proper low-to-high soldering order.
30. As a hobbyist builder, I want each **BOM Line** on the Active Bench Checklist to render individual tappable reference-designator chips (`[○ R1]`, `[✓ R2]`, `[○ R5]`), so that I can check off each placement as I solder it onto the board and have the **BOM Line** automatically transition to `Installed` (deducting from stock if not already `Picked`) once its final designator is checked.
31. As a hobbyist builder, I want to attach local files (`Schematic PDF`, `Build Doc / Drill Template`, `Photos`) and external project/forum URLs to a **Kit Build** and access them from a pinned panel beside the bench checklist, so that my schematics and drill guides are always right next to my soldering checklist.
32. As a hobbyist builder, I want to record timestamped **Bench Log Entries** categorized as `Measurement / Voltage Reading`, `Troubleshooting`, `Modification`, or `Note` (optionally with attached bench photos) in a chronological stream on the **Kit Build**, so that IC pin voltages, audio-probe observations, and mod notes are preserved for future reference.

### CSV Import & Shopping List Workflows

33. As a hobbyist builder, I want to import **Components** and **Stock Lots** by uploading a CSV file or pasting tabular data, choosing a default target **Storage Location** and optional default `Supplier`, and mapping columns using auto-detected supplier presets (Mouser, DigiKey, Tayda, LCSC) or saved custom presets, so that I can ingest supplier order histories and existing spreadsheets quickly.
34. As a hobbyist builder, I want the **Component** CSV import to show an interactive staging and reconciliation preview that classifies every parsed row as `Exact Match` (defaulting to incrementing/adding a **Stock Lot** on the existing **Component**), `Near / Ambiguous Match` (letting me pick an existing candidate **Component** or create a new one), or `No Match` (creating a new **Component** + initial **Stock Lot**), so that imports never create silent duplicate **Components**.
35. As a hobbyist builder, I want to import a **Kit Build** **BOM** via CSV upload or clipboard paste (in either `Append` or `Replace Unallocated Lines` mode), with automatic category inference from reference designator prefixes (`R`→Resistors, `C`→Capacitors, `D`/`Q`→Semiconductors, `U`/`IC`→ICs, `RV`/`VR`→Potentiometers, `SW`/`J`→Switches & Jacks, `L`/`T`→Inductors & Transformers) when a category column is absent, so that raw KiCad or pedal build-doc tables import cleanly.
36. As a hobbyist builder, I want an **"Automatic Designator Grouping"** toggle (enabled by default) during **BOM** CSV import that combines separate single-designator rows sharing an identical `Category + Spec + Package` into a single **BOM Line** with summed `quantity_required` and merged `reference_designators` (e.g., `R1, R2, R5`), so that per-placement CAD exports become compact, readable **BOM Lines**.
37. As a hobbyist builder, I want a live **Shopping List** view where I can select any combination of **Kit Builds** (such as `Sourcing / Waiting on Parts` and chosen `Wishlist / Planned` builds) and have the app aggregate unallocated demand across all selected builds by linked `component_id` (or normalized spec when unlinked) *before* subtracting **Available Stock** to compute **Net Shortfall**, so that shared **Available Stock** is never double-counted across multiple builds.
38. As a hobbyist builder, I want an **"Include catalog low-stock restocking"** toggle on the **Shopping List** that adds any catalog **Component** whose post-build remaining stock falls below its `low_stock_threshold` and tops up the recommended order quantity to reach that threshold, so that I can replenish bench staples in the same supplier order.
39. As a hobbyist builder, I want to expand any aggregated row on the **Shopping List** to see the exact contributing **Kit Builds** and `reference_designators`, inline-edit the `Order Quantity` (to round up for minimum order quantities or spares), and select or override the target `Supplier` and `Supplier Part #`, so that I have full control over the final order quantities and vendor split.
40. As a hobbyist builder, I want to export the **Shopping List** either as a **Full Shopping List CSV** or as a supplier-filtered **Supplier Quick-Order CSV / Clipboard format** (`Supplier Part #, Order Quantity, Customer Reference`), so that I can paste or upload my order directly into Mouser, DigiKey, Tayda, or LCSC cart tools.
41. As a hobbyist builder, I want a **"Receive Selected Lines into Stock"** action directly on the **Shopping List** that creates or increments **Stock Lots** at a chosen **Storage Location** (and automatically creates and links catalog **Components** for any previously unlinked **BOM Lines**), so that when a shipment arrives I can close the loop in a few clicks.

### Data Persistence & Backup

42. As a hobbyist builder running the app locally, I want all SQLite database state (`inventory.db`) and uploaded build attachments (`attachments/`) to live in a single unified data directory with a 1-click **Download Backup (`.zip`)** action and matching restore workflow, so that backing up or migrating my entire workshop inventory is trivial.

## Implementation Decisions

### 1. Technology Stack & Self-Contained Binary Architecture (ADR 0001)
- **HTTP Server & Routing**: Built as a single Cargo crate (library + binary) using `axum` on `tokio`, with `tower-http` handling static asset serving, multipart uploads (CSV imports and build attachments), and request tracing.
- **Compile-Time Server-Rendered HTML & HTMX Partials**: All full pages and HTMX fragments are authored in pure Rust using `maud` HTML macro functions returning `maud::Markup`. Route handlers inspect `HX-Request` headers (or dedicated partial routes) to return either a complete document layout or a targeted HTML fragment (plus out-of-band `hx-swap-oob` fragments where needed).
- **Zero-Node Static Frontend**: Vendored `htmx.min.js` and a standalone CSS stylesheet (using modern CSS custom properties, `@container` responsive table queries, `color-scheme: light dark`, and native `<dialog closedby="any">` modals) are embedded/served directly by the Rust binary with no Node.js, npm, or bundler build step.
- **Embedded Local Database**: Uses the native Rust `turso` crate (`turso::Builder::new_local`) backed by a local file in production and `:memory:` in automated tests. Every connection enables `PRAGMA foreign_keys = ON;`. Numbered SQL migration files are embedded into the binary via `include_str!` and applied automatically at startup inside a transaction tracked by a `_schema_migrations` table.
- **Unified Local Data Directory & Zip Backup/Restore**: Configurable via `--data-dir` or `AUDIO_INVENTORY_DATA_DIR` (defaulting to `./data`), containing `inventory.db` and `attachments/<kit_build_id>/<uuid>-<sanitized_filename>`. A `GET /settings/backup.zip` route streams a consistent snapshot of the database and attachments directory as a single `.zip` archive, paired with a restore workflow that unpacks a backup archive into the data directory.

### 2. Deep Domain Module Boundaries
The crate is organized around five deep domain modules with concise public interfaces, topped by thin HTTP/HTML and filesystem adapters:
1. **`spec_parser` (Pure Rust module)**:
   - Exposes `parse_spec(raw_text, category_hint) -> ParsedSpec` and `infer_category_from_designators(designators) -> Option<Category>`.
   - Encapsulates electrical shorthand parsing (`4k7`, `4.7k`, `4700R`, `0.047uF`, `47n`, `4n7`, `100kA`, `B50k`), canonical display formatting (including formatting `1nF–999nF` capacitances as `nF`), unit normalization (`resistance_ohms`, `capacitance_pf`, `inductance_uh`, `voltage_rating_volts`), parametric attribute extraction (`tolerance`, `power_rating_watts`, `taper`, `package_or_mounting`), and designator-prefix category inference.
2. **`catalog` (Turso-backed module)**:
   - Owns **Components**, the self-referential **Storage Location** tree, and **Stock Lots**.
   - Encapsulates dynamic `json_extract(specs_json, ...)` parametric queries, recursive CTE (`WITH RECURSIVE`) location breadcrumbs and subtree filtering, **Available Stock** calculation, single-lot and multi-lot stock delta adjustments, and automatic/manual **Stock Lot** merging whenever duplicate `(component_id, location_id)` lots are combined.
3. **`builds` (Turso-backed module)**:
   - Owns **Kit Builds**, **BOM Lines**, per-designator bench placement tracking, **Bench Log Entries**, and attachment metadata.
   - Encapsulates the atomic 5-state **Fulfillment Status** state machine (`Unallocated`, `Reserved`, `Picked`, `Installed`, `Included in Kit`), `source_lot_id` deduction and return, batch kitting (`Reserve All Available`, `Pick All Reserved`), 1-click **BOM Line** splitting, component substitution preserving `raw_spec_text`, shelved-build stock recovery, and **Kit Build** cloning.
4. **`imports` (Turso-backed module)**:
   - Owns two-step staged CSV/paste imports (`import_batches` and `import_rows`), column-mapping presets (Mouser, DigiKey, Tayda, LCSC, plus user-saved mappings), automatic designator grouping for identical-spec BOM rows, and `Exact Match` / `Near / Ambiguous Match` / `No Match` catalog reconciliation prior to atomic commit.
5. **`shopping_list` (Turso-backed module)**:
   - Computes live multi-build **Net Shortfall** across selected **Kit Builds**, optional low-stock restock top-ups, inline order quantity and supplier overrides, Full and Supplier Quick-Order CSV/clipboard generation, and direct receiving of arrived lines into **Stock Lots** (including auto-creating/linking catalog **Components** for previously unlinked **BOM Lines**).
6. **`web` & `attachments` (Adapters)**:
   - Thin `axum` route handlers, `maud` view functions, and local attachment file I/O + `.zip` archive streaming.

### 3. Hybrid Parametric Catalog & Storage Schema
- **Nine Built-in Component Categories** (plus `Other / Custom`):
  1. `Resistors` (`specs_json`: `tolerance`, `power_rating_watts`, `composition`)
  2. `Capacitors` (`specs_json`: `dielectric`, `tolerance`, `polarized`, `lead_spacing_mm`)
  3. `Semiconductors — Discrete` (`specs_json`: `part_code`, `polarity_or_channel`, `key_spec`)
  4. `Integrated Circuits (ICs)` (`specs_json`: `part_code`, `function`, `channel_or_pin_count`)
  5. `Potentiometers & Encoders` (`specs_json`: `taper`, `body_size`, `shaft_type`, `gangs`, `has_switch`, `center_detent`)
  6. `Switches, Jacks & Connectors` (`specs_json`: `configuration`, `actuation`, `size_or_pitch`)
  7. `Inductors, Transformers & Magnetics` (`specs_json`: `value_or_ratio`, `current_or_power_rating`)
  8. `Enclosures, Knobs & Mechanical Hardware` (`specs_json`: `size_or_form`, `finish_or_color`, `shaft_fit`)
  9. `Modules & Sub-Assemblies` (`specs_json`: `model_or_platform`, `interface_or_voltage`, `form_factor_or_pinout`)
- **`components` Table**: First-class indexed columns for `id`, `name`, `category`, `subtype`, `package_or_mounting`, `raw_spec_text`, `canonical_value_display`, `resistance_ohms` (`REAL`), `capacitance_pf` (`REAL`), `inductance_uh` (`REAL`), `voltage_rating_volts` (`REAL`), `low_stock_threshold` (`INTEGER`), `datasheet_url`, `notes`, and validated `specs_json` (`TEXT`).
- **`storage_locations` Table**: Self-referential hierarchy (`id`, `name`, `kind` [`Crate`, `Box`, `Binder`, `Drawer`, `Bag`, `Other`], `parent_id`, `notes`).
- **`stock_lots` Table**: Per-location balance (`id`, `component_id`, `location_id`, `quantity`, `supplier`, `supplier_part_no`, `lot_notes`). Enforces at most one active lot per `(component_id, location_id)` pair by automatically summing quantities and merging notes when stock is moved, received, or merged into an existing location for that **Component**.

### 4. BOM Matching, 5-State Fulfillment Machine & Kit Build Lifecycle
- **`kit_builds` Table**: Tracks `id`, `name`, `project_type`, `status` (`Wishlist / Planned`, `Sourcing / Waiting on Parts`, `Ready to Build (Kitted)`, `On the Bench (In Progress)`, `Boxing / Troubleshooting`, `Completed`, `Shelved / Abandoned`), optional `location_id` (referencing `storage_locations`), `description`, and timestamps.
- **`bom_lines` Table**: Tracks `id`, `kit_build_id`, `raw_spec_text`, `canonical_value_display`, `category`, normalized numeric columns, `package_or_mounting`, `reference_designators`, `installed_designators_json` (tracking checked per-designator chips on the bench checklist), `quantity_required`, `quantity_allocated`, `fulfillment_status`, optional `component_id`, optional `source_lot_id`, `substitution_note`, and `notes`.
- **Fulfillment Status State Machine Rules**:
  ```
  Unallocated ──(reserve)──► Reserved (in-place: reduces Available Stock; Stock Lot qty unchanged)
       │                        │
       └────────(pick)──────────┴──► Picked (deducts quantity_allocated from source_lot_id; records source_lot_id)
                                       │
                                  (solder/install)
                                       ▼
                                   Installed (permanently consumed; deducts from source_lot_id if coming from Unallocated/Reserved)

  Included in Kit (fulfilled by kit vendor; never links to or deducts from catalog Stock Lots)
  ```
  - Transitioning from `Picked` back to `Reserved` or `Unallocated` (or shelving a build and choosing to return kitted parts) increments `quantity_allocated` back onto `source_lot_id` (or re-creates the lot at the **Component**'s default/last location if the lot was deleted).
  - **Available Stock** for a catalog **Component** is strictly computed as:
    $$\text{Available Stock} = \sum \text{stock\_lots.quantity} - \sum_{\text{status = Reserved}} \text{bom\_lines.quantity\_allocated}$$
    (because `Picked` and `Installed` lines have already been deducted from `stock_lots.quantity`).
- **1-Click BOM Line Split**: Splitting a **BOM Line** with `quantity_required > 1` takes a split quantity $k$ ($1 \le k < \text{quantity\_required}$) and designator partition, decrements the original line to $\text{quantity\_required} - k$, and inserts a sibling **BOM Line** with quantity $k$ and the partitioned designators.

### 5. Multi-Build Shopping List & Net Shortfall Formula
- Across any user-selected set of **Kit Builds**, every non-kit **BOM Line** (`fulfillment_status NOT IN ('Included in Kit', 'Installed', 'Picked')`) contributes:
  $$\text{unallocated\_needed} = \max(0, \text{quantity\_required} - \text{quantity\_allocated})$$
- Lines are grouped across all selected builds by `component_id` when linked, or by normalized `(category, canonical_value_display, package_or_mounting)` when unlinked.
- **Net Shortfall** is calculated on the combined demand before subtracting **Available Stock**:
  $$\text{Net Shortfall} = \max\left(0, \sum \text{unallocated\_needed} - \text{Available Stock}\right)$$
- When the **"Include catalog low-stock restocking"** toggle is enabled, any catalog **Component** where $(\text{Available Stock} - \sum \text{unallocated\_needed}) < \text{low\_stock\_threshold}$ is included with a recommended order quantity topped up by $\text{low\_stock\_threshold} - (\text{Available Stock} - \sum \text{unallocated\_needed})$.

### 6. Workbench UI & HTMX Interaction Patterns (Validated in Prototype)
- **Layout**: Full-width dense sticky workbench tables (`1fr`) paired with a right-hand 340px context **Inspector Drawer** (`#inspector`), topped by surface navigation tabs (`Component Catalog`, `Kit Build BOM`, `Active Bench Checklist`, `Shopping List`, `CSV Import`).
- **Catalog Search & Inline Steppers**:
  - Search input uses `hx-get="/catalog" hx-trigger="input changed delay:150ms, search" hx-push-url="true"` and renders a live `spec_parser` interpretation badge beside the input.
  - Single-lot **Components** render an inline `− / qty / +` stepper directly inside the table cell (`hx-post="/catalog/lots/{id}/delta"` swapping the component row); multi-lot **Components** render an `"N Stock Lots ▸"` button that opens per-lot steppers and the 1-click **"Merge Duplicate Stock Lots"** action inside the right-hand Inspector Drawer.
- **BOM Table & Native Substitution Modal**:
  - Inline status buttons (`Unallocated`, `Reserved`, `Picked`, `Installed`) trigger row-level `hx-patch` requests that swap the `#bom-row-{id}` `<tr>` and update the header readiness counter (`hx-swap-oob="true"`).
  - Component substitution opens a native `<dialog closedby="any">` modal (`showModal()`) that links any chosen catalog **Component** and saves a `substitution_note` while displaying the original schematic `raw_spec_text` untouched.
- **Active Bench Checklist**:
  - Groups **BOM Lines** into 3 assembly profile height stages and renders tappable per-designator buttons (`[○ R1]`, `[✓ R2]`) that toggle placement state via HTMX and auto-transition the row to `Installed` when the final designator is checked, alongside a pinned right-hand panel for local attachments and timestamped **Bench Log Entries**.

## Testing Decisions

### Philosophy: External Behavior at the Highest Domain Seam
Good tests verify **observable external behavior and domain invariants through public interfaces**, never internal implementation details or mock SQL expectations. Because `axum` route handlers and `maud` templates are a thin rendering layer over the domain modules, the primary testing seam is the **public Rust interface of the five deep domain modules (`spec_parser`, `catalog`, `builds`, `imports`, `shopping_list`) backed by real, ephemeral in-memory Turso databases (`turso::Builder::new_local(":memory:")`) with the production SQL migrations applied**.

### Modules to Be Tested
1. **`spec_parser` (Pure unit tests)**:
   - Shorthand electrical value parsing and canonical display formatting (`4k7` → `4700.0 Ω` / `4.7kΩ`, `0.047uF` → `47000.0 pF` / `47nF`, `4n7` → `4700.0 pF` / `4.7nF`, `100kA` → `100000.0 Ω` + taper `A`, `B50k` → `50000.0 Ω` + taper `B`).
   - Extraction of tolerances (`1%`, `5%`), power ratings (`1/4W`, `0.25W`), voltage ratings (`63V`, `16V`), and packages (`0805`, `DIP-8`, `16mm PCB`) while preserving `raw_spec_text` verbatim.
   - Reference-designator prefix category inference (`R1, R2` → `Resistors`, `C1` → `Capacitors`, `Q1, D1` → `Semiconductors — Discrete`, `U1, IC2` → `Integrated Circuits (ICs)`, `RV1, VR1` → `Potentiometers & Encoders`, `SW1, J1` → `Switches, Jacks & Connectors`).
2. **`catalog` (Integration tests against `:memory:` Turso)**:
   - Creating nested **Storage Locations**, moving a parent container, and verifying updated recursive breadcrumb paths and subtree queries.
   - Creating **Components** across categories, querying by normalized value ranges and `specs_json` attributes, adjusting single and multiple **Stock Lots**, verifying **Available Stock** vs. total on hand, and merging duplicate **Stock Lots** into a single lot.
3. **`builds` (Integration tests against `:memory:` Turso)**:
   - Creating **Kit Builds** and **BOM Lines**, verifying automatic single-match `component_id` linking vs. ambiguous candidate listing.
   - Exercising all 5 **Fulfillment Status** transitions (`Unallocated`, `Reserved`, `Picked`, `Installed`, `Included in Kit`): verifying that `Reserved` reduces **Available Stock** without changing `stock_lots.quantity`, `Picked` deducts from `source_lot_id` and records `source_lot_id`, un-picking or shelving a build returns `Picked` stock to `source_lot_id` and releases `Reserved` stock, and checking the final reference designator chip on the bench checklist transitions the line to `Installed`.
   - Verifying 1-click **BOM Line** splitting (quantities and designators partition cleanly), component substitution (preserving `raw_spec_text` and recording `substitution_note`), **Bench Log Entry** creation, and **Kit Build** cloning (copying BOM lines and shared attachment references while resetting status, allocations, and logs).
4. **`imports` (Integration tests against `:memory:` Turso)**:
   - Staging **Component** CSV imports and verifying `Exact Match`, `Near / Ambiguous Match`, and `No Match` classification, user overrides, and atomic commit into **Components** and **Stock Lots**.
   - Staging **Kit Build** **BOM** CSV/paste imports with automatic designator grouping (`R1`, `R2` combined into `2x R1, R2`) and category inference.
5. **`shopping_list` (Integration tests against `:memory:` Turso)**:
   - Verifying multi-build **Net Shortfall** math when two or more **Kit Builds** compete for the same **Available Stock** (both for linked `component_id` lines and unlinked normalized specs), verifying the low-stock restock top-up calculation, verifying Full and Supplier Quick-Order CSV output formatting, and verifying **"Receive Selected Lines into Stock"** updates **Stock Lots** and links previously unlinked **BOM Lines**.
6. **`web` & Backup Smoke Tests (`tower::ServiceExt::oneshot` + tempdir)**:
   - Smoke testing full-page vs. `HX-Request` partial responses (including `hx-swap-oob` readiness counters) and verifying that exporting `GET /settings/backup.zip` and restoring into a fresh temporary data directory preserves both SQLite rows and attachment files byte-for-byte.

### Prior Art
- All domain tests follow standard Rust `#[tokio::test]` integration testing patterns using a shared `test_db()` helper that initializes `turso::Builder::new_local(":memory:")`, enables `PRAGMA foreign_keys = ON;`, and runs the embedded SQL migrations before each test.

## Out of Scope

- **Multi-user accounts, authentication, permissions, and cloud hosting**: This application is strictly a single-user, local-first personal workbench tool.
- **Serial manufacturing and separate Project Template vs. Production Run entities**: Builds are modeled as a single **Kit Build** concept with 1-click **Clone Kit Build** support when building a second unit.
- **Live third-party supplier API integrations or web scraping**: Real-time Mouser, DigiKey, Tayda, or LCSC API queries are out of scope; supplier workflows use CSV/clipboard import and export plus stored supplier part numbers and URLs.
- **EDA schematic/PCB layout capture or SPICE simulation**: Schematics and drill templates are stored as attached PDFs/images and external links.
- **Multi-currency accounting, tax ledgering, or commercial unit-cost COGS tracking**: Focus is on physical stock quantities, build readiness, bench assembly, and supplier order quantities.

## Further Notes

- **Domain Glossary**: All module names, database columns, and UI labels must adhere strictly to [`GLOSSARY.md`](../../GLOSSARY.md) (**Component**, **Storage Location**, **Stock Lot**, **Kit Build**, **BOM**, **BOM Line**, **Fulfillment Status**, **Available Stock**, **Bench Log Entry**, **Shopping List**, **Net Shortfall**).
- **Architecture Decision Record**: See [`docs/adr/0001-rust-htmx-turso-stack.md`](../../docs/adr/0001-rust-htmx-turso-stack.md).
- **UI Reference Prototype**: See [`.scratch/audio-inventory/prototype-ui.html`](./prototype-ui.html) (Variant A selected) and the resolved design tickets in [`.scratch/audio-inventory/decisions/`](./decisions/) alongside the implementation tickets in [`.scratch/audio-inventory/issues/`](./issues/).

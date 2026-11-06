# Core UI Views & HTMX Interaction Prototype

Type: prototype
Status: resolved
Blocked by: 01, 02, 04

## Question

How should the core server-rendered HTMX views look and behave—specifically fast parametric component search and inline stock adjustment, the Kit Build BOM readiness & part allocation table, and the active bench build checklist view?

## Assets

- [Interactive HTMX UI Prototype (Variants A, B, C)](../prototype-ui.html)

## Answer

### 1. Primary Workbench Layout (**Variant A — Full-Width Dense Workbench Table + Right-Hand Inspector Drawer**)
- **Top Surface Navigation**: Horizontal surface tabs (`Component Catalog & Parametric Search`, `Kit Build BOM & Stock Allocation`, `Active Bench Checklist & Voltage Log`, plus `Shopping List` and `CSV Import`) above a two-column workspace: a **full-width sticky data table** (`1fr`) paired with a **340px right-hand context Inspector Drawer** (`hx-target="#inspector"`).
- **Responsive & Accessible Data Tables**: All data tables use semantic `<caption>`, sticky `<thead>`, and sticky row header `<th scope="row">` inside focusable `role="region"` containers with `@container (width < 600px)` stacked-card responsive fallbacks and automatic `color-scheme: light dark` theming.

### 2. Component Catalog, Live Parametric Search & Multi-Lot Inline Stock Adjustment
- **Debounced HTMX Parametric Search (`hx-get="/catalog" hx-trigger="input changed delay:150ms, search" hx-push-url="true"`)**:
  - Typing shorthand (`4k7`, `0.047uF`, `47n`, `100kB`, `TL072`) updates the catalog results fragment and renders a live **`spec_parser` interpretation badge** (e.g., `spec_parser: Value: 4.7kΩ (4700 Ω) · Tolerance: 1%`) right beside the search bar so the user can immediately verify how the query was normalized.
  - Selecting a **Component** category dynamically reveals category-specific `specs_json` filter controls (e.g., Dielectric & Voltage for Capacitors; Taper & Body Size for Potentiometers), alongside a **Storage Location** breadcrumb filter.
  - Each row displays both the normalized `canonical_value_display` (`4.7kΩ`) and the verbatim `raw_spec_text` (`4k7 1% 1/4W Metal Film`), plus an **Available Stock** badge (`Available / Total On Hand` with reserved count callout and low-stock warning).
- **Single-Lot Inline Stepper vs. Multi-Lot Inspector Drawer**:
  - When a **Component** has a **single Stock Lot**, an inline `− / qty / +` stepper is rendered directly in the table row (`hx-post="/catalog/lots/{id}/delta"`, swapping `#comp-row-{id}`).
  - When a **Component** has **multiple Stock Lots** across different **Storage Locations**, the row cell displays a `"N Stock Lots (Total) ▸"` button that loads the right-hand **Inspector Drawer**, where each individual **Storage Location** lot has its own `− / qty / +` stepper and a 1-click **"Merge Duplicate Stock Lots"** action.

### 3. Kit Build BOM Readiness, Allocation & Kitting Table
- **Row-Level HTMX Fulfillment Transitions + Out-of-Band Summary**:
  - Each **BOM Line** `<tr>` displays `reference_designators`, schematic `raw_spec_text`, linked catalog **Component** (or `Unlinked` / `Included in Kit`), `quantity_allocated / quantity_required` (with a `Shortfall: N` badge when under-allocated and current catalog **Available Stock**), `source_lot_id` location path, and **Fulfillment Status** badge.
  - Inline 1-click status buttons (`Unallocated`, `Reserved`, `Picked`, `Installed`) trigger `hx-patch="/builds/{kit_id}/bom/{bom_id}/status"`, swapping the `#bom-row-{id}` `<tr>` and updating the build's header readiness summary (`N/M Lines Ready · K Shortfalls`) via `hx-swap-oob="true"`.
  - Top-level **Batch Kitting Actions** (`"Reserve All Available"` and `"Pick All Reserved to Project Box"`) allow kitting an entire build in one click.
- **Right-Hand BOM Line Inspector & Native `<dialog>` Substitution Modal**:
  - Clicking any **BOM Line** populates the right-hand **Inspector Drawer** with compatible catalog candidates (when `Unlinked`), `source_lot_id` picker, **1-Click Split BOM Line** (`POST /builds/{kit_id}/bom/{bom_id}/split`, dividing multi-quantity rows and reference designators into two flat **BOM Lines**), and **"Substitute Catalog Component"**.
  - Component substitution opens a native `<dialog closedby="any">` modal (`showModal()`) that links any chosen catalog **Component** and records a `substitution_note` while preserving the schematic's original `raw_spec_text` verbatim.

### 4. Active Bench Build Checklist & Bench Log View
- **Stage-Grouped Assembly Table with Tappable Reference Designator Chips**:
  - Groups **BOM Lines** by assembly profile height / stage (`Stage 1: Low-Profile Resistors & Diodes`, `Stage 2: IC Sockets & Film Capacitors`, `Stage 3: Electrolytics, Pots & Off-Board Hardware`) with an overall placement `<progress>` bar (`X / Y Placements Soldered`).
  - Each **BOM Line** renders interactive per-designator chip buttons (`[○ R1]`, `[✓ R2]`, `[○ R5]`, `hx-patch="/builds/{kit_id}/bench/{bom_id}/designator/{des}"`) so the builder can tap individual placements as they are soldered onto the PCB. Checking the final designator on a line (or clicking `"Mark All Installed"`) automatically transitions the **BOM Line**'s **Fulfillment Status** to `Installed`.
- **Pinned Right-Hand Bench Log & Attachments Panel**:
  - Displays quick links to the build's local attachments (`Schematic PDF`, `Build Doc / Drill Template`, `Photos`) above an inline form (`hx-post="/builds/{kit_id}/bench-logs"`) and chronological stream of timestamped **Bench Log Entries** (`Measurement / Voltage Reading`, `Troubleshooting`, `Modification`, `Note`).

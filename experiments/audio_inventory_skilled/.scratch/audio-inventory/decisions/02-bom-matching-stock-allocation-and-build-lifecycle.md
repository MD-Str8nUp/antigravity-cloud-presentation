# BOM Matching, Stock Allocation, and Build Lifecycle

Type: grilling
Status: resolved
Blocked by: 01

## Question

How do BOM Lines on a Kit Build link to and match against Component inventory (strict link to a specific Component vs. parametric specification matching, and handling partial kits where some parts are already included in the kit box), how does stock reservation/allocation vs. permanent consumption interact with multi-location Stock Lots and the Kit Build's own Storage Location across Kit Build lifecycle stages, and what are the exact Kit Build statuses and transitions (including troubleshooting logs and build doc attachments)?

## Answer

### 1. BOM Line Component Linking, Parametric Matching & Partial Kits
- **Dual Spec + Optional `component_id` Link**:
  - Every **BOM Line** (`bom_lines` table) stores its own required specification (`raw_spec_text`, `category`, normalized numeric value/attributes, `quantity_required`, `reference_designators`, `notes`) *plus* an optional foreign key (`component_id`) to a catalog **Component**.
  - **Auto-match & Parametric Candidates**: On BOM Line creation or import, the deterministic parser parses `raw_spec_text` and automatically links `component_id` if there is a single exact parametric match in the catalog. If multiple catalog **Components** satisfy the spec (e.g., different brands or higher voltage ratings) or if none exists yet, `component_id` remains `NULL` (`Unlinked`) and the UI displays matching catalog candidates alongside a 1-click **"Create Component from BOM Line"** action.
  - **Partial Kit Support (`Included in Kit`)**: A **BOM Line** can be marked as `Included in Kit` for parts supplied inside a partial kit box (such as a PCB, pre-drilled enclosure, or rare IC), marking the line ready without linking to or deducting from catalog **Stock Lots**.

### 2. Stock Allocation, Picking into a Project Box, and Consumption across Stock Lots
- **Fulfillment Statuses on a BOM Line**:
  - `Unallocated`: No stock reserved or pulled yet. Readiness check compares `quantity_required` against **Available Stock** (`SUM(stock_lots.quantity) - SUM(reserved_in_place_quantities)`).
  - `Reserved` (Soft-Reserved in place): Reserves `quantity_allocated` for this **Kit Build** (optionally pinned to a `source_lot_id` or at the **Component** level) without physically moving parts from their drawer/binder. Reduces **Available Stock** for other builds while leaving physical **Stock Lot** counts untouched so drawer counts match reality.
  - `Picked` (Physically Kitted into the **Kit Build**'s **Storage Location**): When parts are pulled from a drawer/binder and placed into the **Kit Build**'s project box/bag, the app immediately deducts `quantity_allocated` from the chosen `source_lot_id` and records `source_lot_id` on the **BOM Line** so 1 click can return the parts to their original **Stock Lot** if un-picked or if the build is shelved/abandoned.
  - `Installed` (Soldered/Consumed on the build): Marks the parts permanently installed on the PCB/enclosure during bench work (deducting from `source_lot_id` at transition time if transitioning directly from `Unallocated` or `Reserved`).
  - `Included in Kit`: Fulfilled by the kit vendor; never touches catalog **Stock Lots**.

### 3. Substitutions & Partial Quantities on a BOM Line
- **Full-Line Substitutions**: Linking a **BOM Line** to *any* catalog `component_id` (even with a different value, rating, or part number, e.g., `2N5457` in place of `J201`) never overwrites the original schematic `raw_spec_text`. An optional `substitution_note` records the substitution reason so the schematic spec and actual part used remain visible side by side.
- **Partial Allocation & 1-Click Split**:
  - Each **BOM Line** tracks `quantity_allocated` (`0 <= quantity_allocated <= quantity_required`), allowing partial reservation/picking of stock on hand while the shortfall (`quantity_required - quantity_allocated`) automatically flows to the shopping list.
  - To fulfill a single multi-quantity **BOM Line** (e.g., `4x 100nF: C1, C2, C3, C4`) using two different **Components** or **Stock Lots**, a 1-click **"Split BOM Line"** action divides the row by quantity and reference designators into two flat **BOM Lines** rather than requiring a nested many-to-many sub-table on every line.

### 4. Kit Build Lifecycle Statuses, Attachments, and Bench Logs
- **Six Lifecycle Statuses** (plus `Shelved / Abandoned`):
  1. `Wishlist / Planned`: Queued idea; checks stock readiness and optionally contributes shortages to shopping lists.
  2. `Sourcing / Waiting on Parts`: Active project awaiting part orders.
  3. `Ready to Build (Kitted)`: All required parts are available/reserved or physically `Picked` into the **Kit Build**'s **Storage Location**.
  4. `On the Bench (In Progress)`: Active assembly; drives the bench checklist view (`Installed` progress across **BOM Lines**).
  5. `Boxing / Troubleshooting`: PCB populated, awaiting enclosure off-board wiring or undergoing bench debugging.
  6. `Completed` (and `Shelved / Abandoned`): Finished build (or parked/abandoned; transitioning to `Shelved / Abandoned` prompts to release `Reserved` parts and return `Picked` parts back to their `source_lot_id`, while leaving `Installed` parts consumed unless manually salvaged).
- **Attachments & External Links (`kit_attachments` table)**: Stores external project/forum URLs plus locally stored files (`Schematic PDF`, `Build Doc / Drill Template`, `Photos`) in a local attachments directory.
- **Timestamped Bench & Troubleshooting Log (`kit_log_entries` table)**: Alongside the build's markdown overview, stores chronological entries typed as `Note`, `Measurement / Voltage Reading`, `Modification`, or `Troubleshooting`.

### 5. Cloning a Kit Build
- **"Clone Kit Build" Action**:
  - **Copied**: Build name (prompted or suffixed `(Copy)`), category/type, description, external URLs, schematic/build-doc attachments (referencing the same on-disk files without duplicating blobs), and all **BOM Lines** including `raw_spec_text`, `reference_designators`, `quantity_required`, linked `component_id`, `substitution_note`, and `Included in Kit` flag.
  - **Reset**: Lifecycle status resets to `Wishlist / Planned`, `location_id` is cleared/prompted, all non-kit **BOM Lines** reset to `Unallocated` (`quantity_allocated = 0`, `source_lot_id = NULL`), and `kit_log_entries` / build-specific photos start empty.

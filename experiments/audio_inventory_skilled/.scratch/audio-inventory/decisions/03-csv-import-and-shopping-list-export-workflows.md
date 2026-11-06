# CSV Import & Shopping List Export Workflows

Type: grilling
Status: resolved
Blocked by: 02

## Question

What are the exact workflows and column/field mappings for importing Components and Kit Build BOMs via CSV, how are duplicate or near-match components reconciled during import, and how are missing/shortfall parts across one or multiple Kit Builds aggregated and exported as supplier shopping lists?

## Answer

### 1. Component & Stock Lot CSV Import Workflow and Reconciliation
- **Two-Step Staged Import**:
  1. **Upload & Column Mapping**: Accepts a CSV file upload or pasted tabular text, selects a default target **Storage Location** and optional default `Supplier` for the batch, and maps columns (`raw_spec_text` / Value, `category`, `subtype`, `package_or_mounting`, `quantity`, `location` path/name, `supplier`, `supplier_part_no`, `datasheet_url`, `notes`). Includes automatic header detection for common supplier exports (Mouser, DigiKey, Tayda, LCSC) and user-saved mapping presets.
  2. **Interactive Staging & Reconciliation Preview**: Runs the deterministic spec parser on every row before committing to the database and classifies each row into one of three states (overridable per row or in bulk):
     - **Exact Match** (matched by `supplier + supplier_part_no` or by normalized `category` + electrical value + `package_or_mounting` + key attributes): Default action is **Add / Increment Stock Lot** on the existing **Component** at the target **Storage Location** (incrementing the lot quantity if a lot at that location already exists, or creating a new **Stock Lot** if stored in a different location).
     - **Near / Ambiguous Match** (e.g., matching category and value where CSV omits secondary specs or multiple catalog **Components** match): Flagged for 1-click user selection between existing candidate **Components** or **Create as New Component**.
     - **No Match**: Default action is **Create New Component + Initial Stock Lot** (auto-creating any new **Storage Location** in the path when requested).

### 2. Kit Build BOM CSV Import & Designator Grouping
- **Flexible Input & Column Mapping**: Supports CSV upload or clipboard paste into a new or existing **Kit Build** (with **Append** or **Replace Unallocated Lines** modes). Maps `raw_spec_text` (Value / Part Name), `reference_designators`, `quantity_required` (defaults to designator count or `1`), `category` (automatically inferred from reference designator prefixes such as `R`→Resistors, `C`→Capacitors, `D`/`Q`→Semiconductors, `U`/`IC`→ICs, `RV`/`VR`→Potentiometers, `SW`/`J`→Switches & Jacks when unmapped), `package_or_mounting`, `notes`, and optional `Included in Kit` flag.
- **Automatic Designator Grouping Toggle**: On by default during import; combines separate single-designator rows sharing an identical `Category + Spec + Package` (common in KiCad/Eagle exports) into a single **BOM Line** with summed `quantity_required` and merged `reference_designators` (e.g., `R1, R2`).
- **Staging Preview with Catalog Auto-Match**: Displays each parsed **BOM Line** alongside its auto-linked catalog **Component** (when a single exact parametric match exists), current **Available Stock**, or candidate matches when ambiguous/unlinked.

### 3. Multi-Build Shopping List Aggregation & Net Shortfall Calculation
- **Shared-Stock Aggregation Across Selected Builds**:
  - For all selected **Kit Builds** (e.g., `Sourcing / Waiting on Parts` and chosen `Wishlist / Planned` builds), each non-kit **BOM Line** contributes `unallocated_needed = quantity_required - quantity_allocated`.
  - Lines are grouped across all selected builds by linked `component_id` (or, when `component_id IS NULL`, by normalized `category + canonical_value_display + package_or_mounting` / `raw_spec_text`).
  - **Net Shortfall** is computed across the combined demand before subtracting **Available Stock**:
    $$\text{Net Shortfall} = \max(0, \sum \text{unallocated\_needed} - \text{Available Stock})$$
    (with `Available Stock = 0` for unlinked specifications).
- **Optional Low-Stock Restock Toggle**: An **"Include catalog low-stock restocking"** toggle adds any catalog **Component** whose post-build remaining stock (`Available Stock - sum(unallocated_needed)`) falls below its `low_stock_threshold`, topping up the recommended order quantity to reach `low_stock_threshold`.
- **Per-Build & Designator Traceability**: Each aggregated row shows total demand, **Available Stock**, **Net Shortfall**, and an expandable breakdown of the contributing **Kit Builds** and `reference_designators`.

### 4. Shopping List Overrides, Supplier Export Formats, and Receiving Parts
- **Live Computed View with Inline Overrides**: The **Shopping List** is a live computed view (not a separate stateful Purchase Order entity). Before exporting, users can inline-edit the **Order Quantity** (initialized to **Net Shortfall**, allowing rounding up for spares/minimums) and choose/edit `Supplier` and `Supplier Part #` (pre-filled from existing **Stock Lot** metadata on the linked **Component**).
- **Export Formats**:
  - **Full Shopping List CSV**: Exports all columns (`Category`, `Spec / Component Name`, `Package`, `Supplier`, `Supplier Part #`, `Available Stock`, `Net Shortfall`, `Order Quantity`, `Kit Builds & Reference Designators`, `Notes`), optionally filtered or grouped by supplier.
  - **Supplier Quick-Order / Cart Format (CSV & Clipboard Copy)**: Filters by a chosen supplier (Mouser, DigiKey, Tayda, LCSC, etc.) and outputs `Supplier Part #, Order Quantity, Customer Reference (Kit Build names)` for direct paste or upload into supplier BOM/cart tools.
- **Closing the Loop on Arrival**: Received orders can either be imported via the **Component & Stock Lot CSV Import** workflow or received directly from the **Shopping List** via a **"Receive Selected Lines into Stock"** action that creates/increments **Stock Lots** at a target **Storage Location** (and auto-creates/links catalog **Components** for any previously unlinked **BOM Lines**).

# 03: Kit Builds, BOM Line Auto-Matching & 5-State Stock Fulfillment

**What to build:** End-to-end **Kit Build** lifecycle tracking and **BOM Line** management, including automatic single-match catalog **Component** linking (with ranked candidate suggestions and 1-click **"Create Component from BOM Line"** when unlinked), the atomic 5-state **Fulfillment Status** state machine (`Unallocated`, `Reserved` in place, `Picked` into the **Kit Build**'s container with `source_lot_id` tracking, `Installed`, and `Included in Kit`), row-level HTMX transitions with out-of-band readiness header counters (`hx-swap-oob`), batch kitting actions (`"Reserve All Available"`, `"Pick All Reserved to Project Box"`), and stock recovery when un-picking or shelving a build.

**Blocked by:** 02: Component Catalog, Parametric Search & Multi-Lot Stock Adjustments

**Status:** resolved

- [x] A builder can create and update **Kit Builds** across all six lifecycle statuses (`Wishlist / Planned`, `Sourcing / Waiting on Parts`, `Ready to Build (Kitted)`, `On the Bench (In Progress)`, `Boxing / Troubleshooting`, `Completed`, plus `Shelved / Abandoned`) and assign an optional physical **Storage Location** (e.g., `Project Box #4`).
- [x] Adding a **BOM Line** parses `raw_spec_text` via `spec_parser`, automatically links `component_id` if a single exact parametric match exists in the catalog, and leaves `component_id` `Unlinked` with ranked compatible catalog candidates (plus a 1-click **"Create Component from BOM Line"** action) in the right-hand Inspector Drawer when zero or multiple catalog **Components** match.
- [x] Each **BOM Line** transitions atomically across the 5 **Fulfillment Status** states:
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
- [x] **Available Stock** for any catalog **Component** is strictly computed as $\sum \text{stock\_lots.quantity} - \sum_{\text{status = Reserved}} \text{bom\_lines.quantity\_allocated}$, and is reflected live in both the **Component** catalog and **Kit Build** **BOM** views.
- [x] Transitioning a `Picked` **BOM Line** back to `Reserved` or `Unallocated` (or transitioning a **Kit Build** to `Shelved / Abandoned` and confirming part return) restores `Picked` `quantity_allocated` back onto `source_lot_id` and releases `Reserved` quantities, while leaving `Installed` lines consumed.
- [x] Row-level **Fulfillment Status** changes on a **BOM Line** swap the updated `<tr>` and immediately update the **Kit Build** header readiness summary (`N/M Lines Ready · K Shortfalls`) via HTMX `hx-swap-oob="true"`, and top-level batch buttons (`"Reserve All Available"` and `"Pick All Reserved to Project Box"`) allocate or pick all eligible lines in one click.
- [x] Integration tests against `:memory:` Turso verify automatic single-match vs. ambiguous **BOM Line** linking, all 5 **Fulfillment Status** transitions, **Available Stock** vs. physical `stock_lots.quantity` invariants, batch kitting, and stock recovery on un-pick and shelved builds.

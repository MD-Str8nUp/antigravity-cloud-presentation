## Destination

A complete functional and architectural specification (`spec.md` alongside [`GLOSSARY.md`](../../GLOSSARY.md) and ADRs in `docs/adr/`) for the single-user audio electronics inventory and DIY kit build web application, with all domain, UX, and technical decisions resolved so implementation tickets can be cut directly.

## Notes

- **Domain**: Personal hobbyist audio electronics component inventory and DIY kit build tracker (single-user, not serial manufacturing). Consult [`GLOSSARY.md`](../../GLOSSARY.md) in every session.
- **Skills**: Every session resolving a ticket should consult `grilling` and `domain-modeling` (and `prototype` for prototype tickets, `codebase-design` when designing module boundaries).
- **Standing constraints**:
  - **User interaction preference**: Always ask clarifying questions **one at a time**.
  - **Tech stack** ([`0001-rust-htmx-turso-stack.md`](../../docs/adr/0001-rust-htmx-turso-stack.md)): Rust backend, server-rendered HTML with HTMX frontend, and a local Turso database.
  - **Issue tracker**: Local markdown under `.scratch/audio-inventory/`. Recommend running `/setup-matt-pocock-skills` if you want to formalize repo-wide skill docs in `docs/agents/`.

## Decisions so far

<!-- the index: one line per closed ticket, enough to judge relevance, then zoom the link for the detail the ticket holds -->

- [Component Category Taxonomy & Parametric Schema](./decisions/01-component-category-taxonomy-and-parametric-schema.md): 9 built-in categories (including Modules & Sub-Assemblies), hybrid SQL + `specs_json` schema preserving verbatim `raw_spec_text` alongside normalized numeric values, self-referential `Storage Location` hierarchy, and multi-location `Stock Lot` balances with lot merging.
- [BOM Matching, Stock Allocation, and Build Lifecycle](./decisions/02-bom-matching-stock-allocation-and-build-lifecycle.md): Dual BOM Line spec + optional `component_id` link with `Included in Kit` support, 5-state `Fulfillment Status` (`Unallocated`, `Reserved` in place, `Picked` into kit location with `source_lot_id` tracking, `Installed`, `Included in Kit`), preserved schematic spec + substitution notes + 1-click line splitting, 6-stage hobbyist lifecycle with local attachments and timestamped `Bench Log Entry` stream, and kit build cloning.
- [CSV Import & Shopping List Export Workflows](./decisions/03-csv-import-and-shopping-list-export-workflows.md): Two-step staged CSV/paste import with column mapping presets and interactive exact/near/new match reconciliation for both Components/Stock Lots and Kit Build BOMs (with reference-designator category inference and identical-spec row grouping), plus a live multi-build Shopping List that aggregates unallocated demand before subtracting Available Stock, supports low-stock restock top-ups, exports Full and Supplier Quick-Order CSV/clipboard formats, and receives arrived lines directly into Stock Lots.
- [Rust + HTMX + Local Turso Crate & App Architecture](./decisions/04-rust-htmx-turso-crate-and-app-architecture.md): Single Cargo crate using `axum` + `tokio` + `tower-http`, `maud` compile-time HTML functions for full-page and HTMX fragment rendering, vendored `htmx.min.js` + standalone CSS (zero Node/npm), native Rust `turso` crate (`turso::Builder::new_local`) with embedded startup SQL migrations, unified `data/` directory (`inventory.db` + `data/attachments/`) with 1-click `.zip` backup/restore, and 5 deep domain modules (`spec_parser`, `catalog`, `builds`, `imports`, `shopping_list`) tested directly against `:memory:` Turso instances.
- [Core UI Views & HTMX Interaction Prototype](./decisions/05-core-ui-views-and-htmx-interaction-prototype.md): Variant A full-width dense sticky workbench tables paired with a right-hand context Inspector Drawer (`hx-target="#inspector"`), debounced `delay:150ms` parametric search with live `spec_parser` interpretation badges, inline single-lot `− / +` steppers vs. drawer-based multi-lot adjustment/merging, row-level HTMX BOM status transitions (`hx-swap-oob` readiness counter) with batch kitting, 1-click BOM line splitting, and native `<dialog>` component substitution, plus a stage-grouped Active Bench Checklist with tappable per-designator (`[R1]`, `[R2]`) soldering chips beside a pinned Bench Log & Attachments panel ([prototype](../audio-inventory/prototype-ui.html)).
- [Audio Electronics Inventory & DIY Kit Build Tracker Specification (`ready-for-agent`)](./spec.md): Complete synthesized specification (`Status: open`, `Labels: ready-for-agent`, also tracked at [`decisions/06-audio-electronics-inventory-and-kit-build-spec.md`](./decisions/06-audio-electronics-inventory-and-kit-build-spec.md)) covering 42 user stories, 6 architectural/domain implementation decision sections, and the `:memory:` Turso domain-module testing seam.

## Not yet specified

<!-- Spec published to `.scratch/audio-inventory/spec.md` with `ready-for-agent` label. -->

## Out of scope

- **Multi-user accounts, cloud hosting, and authentication**: Explicitly single-user local tool.
- **Serial manufacturing / separate Project Template vs. Production Run entities**: Kept as a single **Kit Build** concept (with optional cloning) for a hobbyist builder.
- **Live third-party supplier API integrations (Mouser, DigiKey, Tayda, LCSC live scraping/API calls)**: Initial scope uses manual entry and CSV import/export while storing supplier part numbers/URLs as fields.

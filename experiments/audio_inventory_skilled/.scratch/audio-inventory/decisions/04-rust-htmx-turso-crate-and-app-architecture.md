# Rust + HTMX + Local Turso Crate & App Architecture

Type: grilling
Status: resolved
Blocked by: none

## Question

Which specific Rust crates and architectural boundaries will power the application (HTTP router, HTML templating library, CSS/styling approach, local Turso/libSQL driver and migration strategy, and local filesystem storage for schematic/build-doc attachments), and how should the domain modules be structured?

## Answer

### 1. HTTP Server, HTML Templating & Styling Stack
- **HTTP Server & Routing**: `axum` (on `tokio`) with `tower-http` for static asset serving, multipart file uploads (CSV imports and build attachments), and request tracing.
- **Compile-Time HTML & HTMX Partials**: `maud` HTML macro functions in pure Rust. Every full page and HTMX partial (such as a single **BOM Line** `<tr>`, an inline **Stock Lot** stepper, or an **Available Stock** badge) is a composable Rust function returning `maud::Markup`, allowing route handlers to return either a full document or a targeted HTMX fragment (`HX-Request`) with compile-time type safety and zero separate `.html` template files to keep in sync.
- **Zero-Node Frontend Assets**: Vendored `htmx.min.js` and a standalone static CSS stylesheet (dense tabular workbench layout using CSS custom properties) embedded/served directly by the Rust binary, requiring **no Node.js/npm build step**.

### 2. Local Turso Database Driver, Migrations & Query Execution
- **Database Crate**: The native Rust `turso` crate (`https://crates.io/crates/turso`), initialized via `turso::Builder::new_local(&db_path).build().await?` for local file persistence (and `turso::Builder::new_local(":memory:")` in tests).
- **Connection Pragmas**: Every connection enables `PRAGMA foreign_keys = ON;` on startup.
- **Embedded SQL Migrations**: Numbered SQL files in `migrations/*.sql` embedded into the binary via `include_str!` and applied automatically at startup inside a transaction, tracked in a `_schema_migrations` table.
- **Query Strategy**: Explicit parameterized SQL queries inside the domain modules, paired with a lightweight dynamic `WHERE`-clause builder for `json_extract(specs_json, ...)` parametric filtering, recursive CTEs (`WITH RECURSIVE`) for **Storage Location** hierarchy traversal/breadcrumbs, and multi-build **Shopping List** aggregation.

### 3. Local Data Directory, Attachment Storage & Backup/Restore
- **Unified Data Directory (`DATA_DIR`, defaulting to `./data`, configurable via `--data-dir` or `AUDIO_INVENTORY_DATA_DIR`)**:
  - `data/inventory.db` (and associated WAL/SHM files) for the local `turso` database.
  - `data/attachments/<kit_build_id>/<uuid>-<sanitized_filename>` on disk for uploaded schematic PDFs, build guides, and bench photos, with file metadata (`id`, `kit_build_id`, `bench_log_entry_id`, `original_filename`, `mime_type`, `relative_path`, `size_bytes`, `created_at`) stored in an `attachments` table.
- **1-Click Backup & Restore**:
  - In-app `GET /settings/backup.zip` endpoint that streams a consistent snapshot of `inventory.db` together with `data/attachments/` as a single `.zip` archive download.
  - Matching restore/CLI workflow that unpacks a backup `.zip` into `DATA_DIR`.

### 4. Crate Layout, Deep Domain Modules & Test Seam
- **Single Cargo Package (Library + Binary)** structured around 5 deep modules with small interfaces and a thin HTTP/HTML adapter:
  1. **`spec_parser` (Pure Rust module)**: Exposes `parse_spec(raw_text, category_hint) -> ParsedSpec` and `infer_category_from_designators(designators) -> Option<Category>`. Encapsulates unit normalization (`4k7` → `4700.0 Ω`, `100nF` → `1e-7 F`), tolerance/wattage/voltage/package extraction, and reference-designator rules. Reused across manual catalog entry, BOM creation, CSV import staging, and Shopping List grouping.
  2. **`catalog` (Turso-backed module)**: Owns **Components**, the recursive **Storage Location** hierarchy, and **Stock Lots**. Encapsulates dynamic `specs_json` parametric queries, recursive CTE location paths (`Crate A > Op-Amps Bag`), **Available Stock** calculation, inline stock deltas, and automatic **Stock Lot** merging when moving/receiving stock into a `(component_id, location_id)` that already exists.
  3. **`builds` (Turso-backed module)**: Owns **Kit Builds**, **BOM Lines**, **Bench Log Entries**, and the atomic 5-state **Fulfillment Status** state machine (`Unallocated`, `Reserved`, `Picked` with `source_lot_id` tracking and automatic **Stock Lot** deduction/return, `Installed`, `Included in Kit`), 1-click BOM line splitting on partial pick/substitution, and kit build cloning.
  4. **`imports` (Turso-backed module)**: Owns the two-step staged CSV/paste workflow (`import_batches` / `import_rows`), supplier column-mapping presets, identical-spec designator grouping, and Exact / Near / New catalog reconciliation prior to atomic commit.
  5. **`shopping_list` (Turso-backed module)**: Computes live multi-build **Net Shortfall** (summing unallocated demand across selected **Kit Builds** before subtracting **Available Stock**, plus optional low-stock restock top-ups), exports Full and Supplier Quick-Order CSV/clipboard formats, and receives arrived orders directly into **Stock Lots**.
  6. **`web` & `attachments` (HTTP / `maud` / filesystem adapters)**: Thin `axum` route handlers and `maud` view functions rendering full pages or targeted HTMX partials, plus local attachment storage and `.zip` backup streaming.
- **Testing Seam**: Domain and integration tests exercise `catalog`, `builds`, `imports`, and `shopping_list` directly through their public module interfaces against ephemeral in-memory `turso::Builder::new_local(":memory:")` instances with real migrations applied (zero SQL mocks).

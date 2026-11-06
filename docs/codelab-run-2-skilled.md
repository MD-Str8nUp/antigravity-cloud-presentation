# Codelab Script — Run 2: The Skilled Run (`audio_inventory_skilled`)

This document chronicles **Run 2** (**Skilled Run**) of the three-run Antigravity Cloud Website Experiment and serves as the script for the **Skilled Run** **Codelab**.

In this run, we start in a fresh empty directory (`~/Demos/audio-inventory-experiment/audio_inventory_skilled`), install Matt Pocock's engineering workflow skills (`npx skills@latest add mattpocock/skills`), and give `agy` the **exact same opening prompt** as the **Baseline Run**—only this time prefixed with `/wayfinder`:

```text
/wayfinder I'd like to build a web application to track my audio electronics components inventory and DIY kit builds.
```

Instead of jumping straight from a 4-question chat interview into a 2,500-line unverified one-shot JavaScript build, the installed skills guide `agy` and the operator through a disciplined, multi-session engineering lifecycle—with `/clear` between every stage so each session relies on **durable repository artifacts** (`GLOSSARY.md`, `docs/adr/`, `.scratch/audio-inventory/map.md`, `decisions/`, `prototype-ui.html`, `spec.md`, and `issues/`) rather than ephemeral chat context.

---

## 1. Run Overview & Artifacts

| Artifact / Property | Details |
| :--- | :--- |
| **Experiment Run** | **Skilled Run** (`audio_inventory_skilled`) |
| **Installed Skills** | [`mattpocock/skills`](https://github.com/mattpocock/skills) — 38 skills installed into `.agents/skills/` (including `/wayfinder`, `/grilling`, `/domain-modeling`, `/codebase-design`, `/prototype`, `/to-spec`, `/to-tickets`, `/implement-spec`, `/tdd`, `/code-review`) |
| **Model** | Gemini 4 Argon (Medium) |
| **Terminal Cast** | [`recordings/final/02-skilled.cast`](../recordings/final/02-skilled.cast) (raw: `9h 29m 21s`, 1x idle-clamped: `2h 20m 45s`, adaptive warped playback: `16m 25s` across 111 chapter stop points) |
| **Stop-Point Spec** | [`scripts/markers/02-skilled.json`](../scripts/markers/02-skilled.json) |
| **Interactive Cast Player** | [`prototypes/02-skilled-cast-player.html`](../prototypes/02-skilled-cast-player.html) |
| **Session Traces** | [`artifacts/traces/audio_inventory_skilled/`](../artifacts/traces/audio_inventory_skilled/README.md) — **21 total traces**: 10 top-level operator sessions (`01`–`09`, `21`) separated by `/clear`, plus 11 autonomous subagent sessions (`10`–`20`) spawned during `/implement-spec` and `/code-review` |
| **Synced Application Code** | [`experiments/audio_inventory_skilled/`](../experiments/audio_inventory_skilled/) (Rust `axum` + `maud` compile-time HTML/HTMX + native Rust `turso` crate + vendored `htmx.min.js` & CSS, verified by **41 unit & integration tests**) |
| **Durable Domain & Spec Artifacts** | - [`GLOSSARY.md`](../experiments/audio_inventory_skilled/GLOSSARY.md) (11 core ubiquitous domain terms)<br>- [`docs/adr/0001-rust-htmx-turso-stack.md`](../experiments/audio_inventory_skilled/docs/adr/0001-rust-htmx-turso-stack.md)<br>- [`.scratch/audio-inventory/map.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/map.md)<br>- 5 resolved Wayfinder decision records in [`.scratch/audio-inventory/decisions/`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/decisions/)<br>- Interactive 3-variant UI prototype: [`.scratch/audio-inventory/prototype-ui.html`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/prototype-ui.html)<br>- Synthesized specification: [`.scratch/audio-inventory/spec.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/spec.md)<br>- 8 vertical-slice implementation tickets in [`.scratch/audio-inventory/issues/`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/issues/) |

---

## 2. Multi-Session Architecture & Stop-Point Timeline

The **Skilled Run** **Terminal Cast** ([`02-skilled.cast`](../recordings/final/02-skilled.cast)) contains **111 stop points** organized across 10 operator sessions (with `/clear` between every session):

| Session | Trace # | Slash Command / Prompt | Cast Stops | 1x Time Span | Warped Span | Key Deliverable Produced |
| :---: | :---: | :--- | :---: | :---: | :---: | :--- |
| **Setup** | — | `npx skills@latest add mattpocock/skills` & `agy` | **1–4** | `0:01`–`0:14` | `0:00`–`0:07` | 38 skills installed in `.agents/skills/` |
| **Session 1** | [`01`](../artifacts/traces/audio_inventory_skilled/01-7f68013b-i-d-like-to-build-a-web.md) | `/wayfinder I'd like to build a web application...` | **5–28** | `0:17`–`5:27` | `0:08`–`2:32` | [`GLOSSARY.md`](../experiments/audio_inventory_skilled/GLOSSARY.md), [`ADR 0001`](../experiments/audio_inventory_skilled/docs/adr/0001-rust-htmx-turso-stack.md), [`map.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/map.md), 5 decision tickets |
| **Session 2** | [`02`](../artifacts/traces/audio_inventory_skilled/02-1c89dce0-conversation.md) | `/wayfinder` *(Ticket 01: Taxonomy & Schema)* | **29–45** | `5:30`–`13:29` | `2:34`–`5:57` | Resolved [`01-component-category-taxonomy-and-parametric-schema.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/decisions/01-component-category-taxonomy-and-parametric-schema.md) |
| **Session 3** | [`03`](../artifacts/traces/audio_inventory_skilled/03-91bbb269-conversation.md) | `/wayfinder` *(Ticket 02: BOM & Lifecycle)* | **46–58** | `13:32`–`17:43` | `5:58`–`7:37` | Resolved [`02-bom-matching-stock-allocation-and-build-lifecycle.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/decisions/02-bom-matching-stock-allocation-and-build-lifecycle.md) |
| **Session 4** | [`04`](../artifacts/traces/audio_inventory_skilled/04-02f0a814-conversation.md) | `/wayfinder` *(Ticket 03: CSV & Shopping List)* | **59–69** | `17:46`–`21:56` | `7:38`–`9:05` | Resolved [`03-csv-import-and-shopping-list-export-workflows.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/decisions/03-csv-import-and-shopping-list-export-workflows.md) |
| **Session 5** | [`05`](../artifacts/traces/audio_inventory_skilled/05-1fdd9c16-conversation.md) | `/wayfinder` *(Ticket 04: Rust + HTMX + Turso)* | **70–80** | `21:59`–`26:55` | `9:06`–`10:47` | Resolved [`04-rust-htmx-turso-crate-and-app-architecture.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/decisions/04-rust-htmx-turso-crate-and-app-architecture.md) |
| **Session 6** | [`06`](../artifacts/traces/audio_inventory_skilled/06-c5f4c9c0-conversation.md) | `/wayfinder` *(Ticket 05: UI Prototype)* | **81–92** | `26:58`–`37:09` | `10:48`–`12:37` | [`prototype-ui.html`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/prototype-ui.html) (Variants A, B, C) & resolved [`05-core-ui-views...md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/decisions/05-core-ui-views-and-htmx-interaction-prototype.md) |
| **Session 7** | [`07`](../artifacts/traces/audio_inventory_skilled/07-99e2fb73-conversation.md) | `/to-spec` | **93–97** | `37:13`–`41:12` | `12:38`–`13:34` | Complete specification [`.scratch/audio-inventory/spec.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/spec.md) |
| **Session 8** | [`08`](../artifacts/traces/audio_inventory_skilled/08-d4487a2d-conversation.md) | `/to-tickets` | **98–102** | `41:16`–`46:29` | `13:36`–`14:32` | 8 vertical-slice implementation tickets in [`.scratch/audio-inventory/issues/`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/issues/) |
| **Session 9** | [`09`](../artifacts/traces/audio_inventory_skilled/09-980f0ef9-conversation.md) *(+ `10`–`20`)* | `/implement-spec` | **103–107** | `46:33`–`117:39` | `14:34`–`15:45` | All 8 tickets built via worktree TDD + parallel 2-axis `/code-review` (`41 passed, 0 failed`) |
| **Session 10** | [`21`](../artifacts/traces/audio_inventory_skilled/21-73c7d66f-please-test-this-site-in-a-web.md) | `please test this site in a web browser` | **108–111** | `117:44`–`140:44` | `15:48`–`16:25` | Live Chrome DevTools verification & 4 HTMX/DOM/CSS integration fixes |

---

## 3. Pre-Launch Setup: Installing `mattpocock/skills` (Stops 1–4)

### Stop 1 (`Paste Command: npx skills@latest add mattpocock/skills`) & Stop 2 (`38 Skills Installed`)
Before launching `agy`, the operator installs Matt Pocock's skill suite into the empty workspace:

```bash
npx skills@latest add mattpocock/skills
```

In ~9 seconds, the CLI downloads and installs **38 skills** into `.agents/skills/` (including `wayfinder`, `grilling`, `domain-modeling`, `codebase-design`, `prototype`, `to-spec`, `to-tickets`, `implement-spec`, `implement`, `tdd`, and `code-review`).

### Stop 3 (`Run Command: agy`) & Stop 4 (`Antigravity Prompt Ready`)
The operator starts the Antigravity CLI:

```bash
agy
```

---

## 4. Session 1: Charting the Wayfinder Map (Stops 5–28)

**Session Trace**: [`01-7f68013b-i-d-like-to-build-a-web.md`](../artifacts/traces/audio_inventory_skilled/01-7f68013b-i-d-like-to-build-a-web.md) (`10m 23s`, 45 steps)

### Stop 5 (`Submit Prompt: /wayfinder Audio Inventory`)
The operator enters the exact same prompt as Run 1, prefixed with `/wayfinder`:

```text
/wayfinder I'd like to build a web application to track my audio electronics components inventory and DIY kit builds.
```

**What `agy` does under the hood**:
- Reads `.agents/skills/grilling/SKILL.md`, `.agents/skills/domain-modeling/SKILL.md` (plus `GLOSSARY-FORMAT.md` and `ADR-FORMAT.md`), and `.agents/skills/setup-matt-pocock-skills/issue-tracker-local.md`.
- Rather than writing application code immediately, `/wayfinder` interviews the user one question at a time to establish **Destination**, **Standing Constraints**, **Ubiquitous Domain Vocabulary**, **Out-of-Scope Boundaries**, and the **Initial Decision Ticket Frontier**.

### Stops 6–27: The 11 Map-Charting Questions

| Stop Pair | Topic | Question Summary | Operator Answer |
| :---: | :--- | :--- | :--- |
| **6 → 7** | **Map Q1: Wayfinding Map Destination** | Confirm the map destination: a complete functional & architectural specification (`spec.md` + `GLOSSARY.md` + `docs/adr/`) tracked locally in `.scratch/audio-inventory/`. | `yes` |
| **8 → 9** | **Map Q2: Grilling Pace** | Should questions across wayfinding sessions be asked **one question at a time** or batched? | `one question at a time` |
| **10 → 11** | **Map Q3: Kit Build & BOM Scope** | Should the app cover both Component Inventory AND Kit Builds with BOM matching, stock readiness, part allocation/deduction, and missing-part shopping lists? | `yes` |
| **12 → 13** | **Map Q4: Structured vs. Freeform Component Schema** | Should components have audio-specific structured parametric attributes (ohms, capacitance, taper, package, footprint) rather than just a generic name + count? | `structured` |
| **14 → 15** | **Map Q5: Single-User Local vs. Multi-User** | Is this a personal, single-user workbench tool or multi-user with authentication? | `single user` |
| **16 → 17** | **Map Q6: Manual Entry & CSV Import vs. Supplier APIs** | Keep live supplier API scraping (Mouser/DigiKey/Tayda APIs) out of scope and focus on manual entry + CSV import/export while storing supplier part numbers/URLs? | `sgtm` |
| **18 → 19** | **Map Q7: Build Stages, Bench Logs & Attachments** | Should Kit Builds track multi-stage progress, build/troubleshooting notes, and schematic/build-doc links or attachments? | `yes please` |
| **20 → 21** | **Map Q8: Single Kit Build Concept vs. Separate Template** | Model a **Kit Build** as a single project concept (with a BOM and status) rather than separate "Project Template" vs. "Manufacturing Run" entities? | `single concept, I'm not a serial maufacturer, just a hobby builder` |
| **22 → 23** | **Map Q9: Tech Stack Decision vs. Standing Preference** | Do you have standing tech stack preferences, or should we open a research/decision ticket to evaluate stacks? | `I have tech preferences` |
| **24 → 25** | **Map Q10: Specify Tech Stack Preferences** | What tech stack would you like to use for the backend, frontend/UI, and local database? | `Rust backend, server-rendered HTML with HTMX frontend, and a local Turso database` |
| **26 → 27** | **Map Q11: Confirm Wayfinding Map & 5 Decision Tickets** | Presents the proposed Wayfinding Map and 5 decision tickets (`01`–`05`) for confirmation. | `lgtm` |

### Stop 28 (`Wayfinder Map Charted (GLOSSARY, ADR 0001, map.md, 5 Tickets)`)
Before ending Session 1, `agy` writes **8 durable markdown files** to disk:
1. [`GLOSSARY.md`](../experiments/audio_inventory_skilled/GLOSSARY.md) — Defines `Component`, `Storage Location`, `Kit Build`, `BOM`, `BOM Line`, and `Available Stock` (along with explicit `_Avoid_` synonyms).
2. [`docs/adr/0001-rust-htmx-turso-stack.md`](../experiments/audio_inventory_skilled/docs/adr/0001-rust-htmx-turso-stack.md) — Records the architectural decision to use Rust + server-rendered HTML/HTMX + local Turso.
3. [`.scratch/audio-inventory/map.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/map.md) — The Wayfinder map tracking destination, standing constraints, decisions, and out-of-scope items.
4. Five open Wayfinder tickets in `.scratch/audio-inventory/issues/`:
   - `01-component-category-taxonomy-and-parametric-schema.md` (`Type: grilling`, unblocked)
   - `02-bom-matching-stock-allocation-and-build-lifecycle.md` (`Type: grilling`, blocked by `01`)
   - `03-csv-import-and-shopping-list-export-workflows.md` (`Type: grilling`, blocked by `02`)
   - `04-rust-htmx-turso-crate-and-app-architecture.md` (`Type: grilling`, unblocked)
   - `05-core-ui-views-and-htmx-interaction-prototype.md` (`Type: prototype`, blocked by `01, 02, 04`)

---

## 5. Sessions 2–6: Resolving the 5 Wayfinder Decision Tickets (Stops 29–92)

Between every ticket, the operator runs `/clear` and simply types `/wayfinder`. Each new session reads `GLOSSARY.md`, `docs/adr/0001-rust-htmx-turso-stack.md`, and `.scratch/audio-inventory/map.md`, claims the next unblocked ticket on the frontier, resolves it with the operator, updates `GLOSSARY.md` and `map.md`, and stops cleanly.

---

### Session 2 — Ticket 01: Component Category Taxonomy & Parametric Schema (Stops 29–45)

**Session Trace**: [`02-1c89dce0-conversation.md`](../artifacts/traces/audio_inventory_skilled/02-1c89dce0-conversation.md) (`12m 45s`, 50 steps)  
**Decision Record**: [`decisions/01-component-category-taxonomy-and-parametric-schema.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/decisions/01-component-category-taxonomy-and-parametric-schema.md)

After `/clear` (Stop 29) and `/wayfinder` (Stop 30), `agy` claims Ticket `01` and walks through 6 domain-modeling questions:

| Stop Pair | Question Focus | Operator Answer & Resulting Domain Model |
| :---: | :--- | :--- |
| **31 → 32** | **Ticket 01 Q1: Component Category Taxonomy** (`agy` proposes 8 audio categories: Resistors, Capacitors, Discrete Semiconductors, ICs, Pots & Encoders, Switches/Jacks, Inductors/Transformers, Enclosures/Hardware) | `I'll need a ninth category for manufactured sub assemblies like MCU boards and the like.` → Adds **9. Modules & Sub-Assemblies** (e.g., Daisy Seed, Teensy, Pico, Belton BTDR reverb bricks). |
| **33 → 34** | **Ticket 01 Q2: Hybrid SQL + `specs_json` Schema** (first-class indexed columns for category, package, normalized numeric values + validated `specs_json` per category) | `sgtm` |
| **35 → 36** | **Ticket 01 Q3: Electrical Value Normalization** (how shorthand like `4k7`, `0.047uF`, `100kA` is stored and searched) | `I'd like to store the original spec string alongside a canonicalized value. A lot of the values include additional information, like variance and pot curve that are important` → Preserves verbatim `raw_spec_text` alongside `canonical_value_display` and normalized floats (`resistance_ohms`, `capacitance_pf`, `inductance_uh`, `voltage_rating_volts`). |
| **37 → 38** | **Ticket 01 Q4: Physical Storage Locations** (flat bins vs. nested containers) | `I currently have a mixture of boxes with parts like partially assembled kits, crates with bags of components, and in some cases I have crates with bags full of bagged components. Given I need to move the bags of components around it is probably worth having a heirarichal concept for storage` → Models **Storage Location** as a self-referential tree (`parent_id`) where moving a bag updates the breadcrumb path for all nested lots and kits. |
| **39 → 40** | **Ticket 01 Q5: Duplicate Bags Across Locations** (single location per component vs. multiple stashes) | `I have duplicate bags with the same (or similar) components in multiple locations, because it was easier to order more components than find the ones I already have` → Introduces multi-location **Stock Lots** (`stock_lots` table) plus a **Merge Lots** operation! |
| **41 → 42** | **Ticket 01 Q6: Stock Lot Domain Term & Metadata** (adds **Stock Lot** to `GLOSSARY.md` with per-lot supplier, part #, and notes) | `sgtm` |
| **43 → 44** | **Ticket 01 Confirm Resolution** | `yes` → Resolves Ticket 01 and updates `GLOSSARY.md` & `map.md` (Stop 45). |

> **Codelab Callout**: Look at Stops 37–40. Because `/grilling` and `/domain-modeling` probed how the builder *actually* stores parts in real life ("crates with bags full of bagged components" and "duplicate bags because it was easier to order more than find the ones I have"), the domain model gained two first-class concepts—a self-referential **Storage Location** tree and multi-location **Stock Lots** with lot merging—that never surfaced in Run 1.

---

### Session 3 — Ticket 02: BOM Matching, Stock Allocation & Build Lifecycle (Stops 46–58)

**Session Trace**: [`03-91bbb269-conversation.md`](../artifacts/traces/audio_inventory_skilled/03-91bbb269-conversation.md) (`7m 48s`, 42 steps)  
**Decision Record**: [`decisions/02-bom-matching-stock-allocation-and-build-lifecycle.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/decisions/02-bom-matching-stock-allocation-and-build-lifecycle.md)

After `/clear` (Stop 46) and `/wayfinder` (Stop 47), `agy` claims Ticket `02` (now unblocked by Ticket `01`) and resolves 5 questions:

- **Stops 48 → 49 (`Ticket 02 Q1: BOM Line Linking & Partial Kits`)**: Dual spec (`raw_spec_text`) + optional `component_id` foreign key, auto-linking single exact matches, showing ranked candidates or 1-click *"Create Component from BOM Line"* when unlinked, and an `Included in Kit` status for parts supplied inside a partial kit box (`sgtm`).
- **Stops 50 → 51 (`Ticket 02 Q2: 5-State BOM Fulfillment Status`)**: Defines the 5-state **Fulfillment Status** machine (`Unallocated`, `Reserved` in place reducing **Available Stock** without changing drawer counts, `Picked` into the **Kit Build**'s project box deducting from `source_lot_id`, `Installed` on the PCB, and `Included in Kit`) (`sgtm`).
- **Stops 52 → 53 (`Ticket 02 Q3: Substitutions, Partial Allocations & Split Line`)**: Linking a substitute **Component** (e.g., `2N5457` for `J201`) preserves `raw_spec_text` alongside a `substitution_note`, and a 1-click **"Split BOM Line"** action divides a multi-quantity line (`4x C1, C2, C3, C4`) across two different **Stock Lots** or parts (`sgtm`).
- **Stops 54 → 55 (`Ticket 02 Q4: 6-Stage Build Lifecycle & Bench Log`)**: Six hobbyist statuses (`Wishlist / Planned`, `Sourcing / Waiting on Parts`, `Ready to Build (Kitted)`, `On the Bench (In Progress)`, `Boxing / Troubleshooting`, `Completed`, plus `Shelved / Abandoned` with automatic return of `Picked` parts to `source_lot_id`), local attachments (`Schematic PDF`, `Build Doc`, `Photos`), and timestamped **Bench Log Entries** (`sgtm`).
- **Stops 56 → 57 (`Ticket 02 Q5: Cloning a Kit Build`)**: 1-click **"Clone Kit Build"** copies BOM specs, links, and shared schematic attachments while resetting fulfillment statuses to `Unallocated` (`sgtm`).
- **Stop 58 (`Ticket 02 Resolved`)**: Updates `GLOSSARY.md` (adding `Fulfillment Status` and `Bench Log Entry`), resolves Ticket 02, and updates `map.md`.

---

### Session 4 — Ticket 03: CSV Import & Shopping List Export Workflows (Stops 59–69)

**Session Trace**: [`04-02f0a814-conversation.md`](../artifacts/traces/audio_inventory_skilled/04-02f0a814-conversation.md) (`7m 23s`, 38 steps)  
**Decision Record**: [`decisions/03-csv-import-and-shopping-list-export-workflows.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/decisions/03-csv-import-and-shopping-list-export-workflows.md)

After `/clear` (Stop 59) and `/wayfinder` (Stop 60), `agy` claims Ticket `03` and resolves 4 questions:

- **Stops 61 → 62 (`Ticket 03 Q1: Staged Component CSV Import`)**: Two-step staged import with supplier presets (`Mouser`, `DigiKey`, `Tayda`, `LCSC`) and interactive `Exact Match` / `Near / Ambiguous Match` / `No Match` reconciliation before atomic commit (`yes`).
- **Stops 63 → 64 (`Ticket 03 Q2: Kit Build BOM CSV Import & Designator Grouping`)**: Infers component categories from reference designator prefixes (`R`, `C`, `D`/`Q`, `U`/`IC`, `RV`/`VR`, `SW`/`J`) and groups single-designator KiCad/Eagle rows sharing identical specs (`yes`).
- **Stops 65 → 66 (`Ticket 03 Q3: Multi-Build Shopping List Aggregation`)**: Aggregates unallocated demand across all selected **Kit Builds** *before* subtracting **Available Stock** to compute **Net Shortfall**:
  $$\text{Net Shortfall} = \max\left(0, \sum \text{unallocated\_needed} - \text{Available Stock}\right)$$
  plus an optional low-stock restock top-up toggle (`yes`). *(Notice how this explicitly prevents the multi-line/multi-build stock double-counting bug that bit Run 1!)*
- **Stops 67 → 68 (`Ticket 03 Q4: Shopping List Overrides, Export & Receiving`)**: Inline `Order Quantity` / `Supplier` overrides, Full CSV and Supplier Quick-Order CSV/Clipboard export, and 1-click *"Receive Selected Lines into Stock"* (`yes`).
- **Stop 69 (`Ticket 03 Resolved`)**: Adds `Shopping List` and `Net Shortfall` to `GLOSSARY.md` and resolves Ticket 03.

---

### Session 5 — Ticket 04: Rust + HTMX + Local Turso Architecture (Stops 70–80)

**Session Trace**: [`05-1fdd9c16-conversation.md`](../artifacts/traces/audio_inventory_skilled/05-1fdd9c16-conversation.md) (`6m 04s`, 39 steps)  
**Decision Record**: [`decisions/04-rust-htmx-turso-crate-and-app-architecture.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/decisions/04-rust-htmx-turso-crate-and-app-architecture.md)

After `/clear` (Stop 70) and `/wayfinder` (Stop 71), `agy` reads `.agents/skills/codebase-design/SKILL.md` (the deep-modules design skill) and claims Ticket `04`:

- **Stops 72 → 73 (`Ticket 04 Q1: Axum, Maud & Vendored HTMX Stack`)**: `axum` + `tokio` + `tower-http`, compile-time `maud` HTML functions for both full pages and HTMX partials, and vendored `htmx.min.js` + standalone CSS with zero Node/npm build step (`sgtm`).
- **Stops 74 → 75 (`Ticket 04 Q2: Local Turso Driver & Migrations`)**: When `agy` asks whether to use the `turso` crate or `libsql`, the operator pastes `https://crates.io/crates/turso`. `agy` runs two `search_web` queries to verify `turso::Builder::new_local` and SQLite compatibility (`json_extract`, recursive CTEs, foreign keys), then updates [`docs/adr/0001-rust-htmx-turso-stack.md`](../experiments/audio_inventory_skilled/docs/adr/0001-rust-htmx-turso-stack.md).
- **Stops 76 → 77 (`Ticket 04 Q3: Unified Data Directory & ZIP Backup`)**: Unified `DATA_DIR` (`data/inventory.db` + `data/attachments/<kit_build_id>/`) with 1-click `GET /settings/backup.zip` snapshot download and restore (`sgtm`).
- **Stops 78 → 79 (`Ticket 04 Q4: 5 Deep Domain Modules & In-Memory Tests`)**: Designs 5 deep domain modules—`spec_parser`, `catalog`, `builds`, `imports`, and `shopping_list`—tested directly against ephemeral `:memory:` Turso instances with real SQL migrations applied (`sgtm`).
- **Stop 80 (`Ticket 04 Resolved`)**: Resolves Ticket 04 and unblocks Ticket 05.

---

### Session 6 — Ticket 05: Interactive UI Prototype (`prototype-ui.html`) (Stops 81–92)

**Session Trace**: [`06-c5f4c9c0-conversation.md`](../artifacts/traces/audio_inventory_skilled/06-c5f4c9c0-conversation.md) (`50m 35s`, 52 steps)  
**Decision Record**: [`decisions/05-core-ui-views-and-htmx-interaction-prototype.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/decisions/05-core-ui-views-and-htmx-interaction-prototype.md)  
**Prototype File**: [`.scratch/audio-inventory/prototype-ui.html`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/prototype-ui.html)

After `/clear` (Stop 81) and `/wayfinder` (Stop 82), `agy` claims Ticket `05` (`Type: prototype`), reads `.agents/skills/prototype/SKILL.md` (`UI.md` and `LOGIC.md`) and `modern-web-guidance`, and builds a self-contained interactive HTML prototype:

- **Stop 83 (`UI Prototype Generation Complete`) & Stop 84 (`Ticket 05 Prototype Ready (Variants A, B, C)`)**:
  `agy` writes [`.scratch/audio-inventory/prototype-ui.html`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/prototype-ui.html) implementing **three distinct layout variants** switchable via a floating bottom bar (`?variant=a|b|c`):
  - **Variant A**: Full-width dense sticky workbench tables paired with a 340px right-hand context Inspector Drawer (`#inspector`).
  - **Variant B**: Three-pane IDE layout (persistent left Storage Location & Kit tree + center table + right inspector).
  - **Variant C**: Split-screen Kitting view (Kit BOM on left + live Catalog search on right) and high-contrast Bench HUD.
- **Stop 85 (`Request: Open Prototype in Browser`)**:
  The operator prompts:
  ```text
  please open the prototype in a browser
  ```
  `agy` runs `open .scratch/audio-inventory/prototype-ui.html` so the operator can test-drive Variants A, B, and C in Chrome.
- **Stops 86 → 87 (`Ticket 05 Q1: Overall Workbench Layout`)**: The operator selects **`A`** (**Variant A** — Full-Width Dense Workbench Table + Right-Hand Inspector Drawer).
- **Stops 88 → 89 (`Ticket 05 Q2: Parametric Search & Multi-Lot Steppers`)**: Confirms the debounced `delay:150ms` search with live `spec_parser` interpretation pills, inline `− / qty / +` steppers for single-lot components, and the Inspector Drawer for multi-lot adjustments/merging (`sgtm`).
- **Stops 90 → 91 (`Ticket 05 Q3: BOM Allocation & Active Bench Checklist`)**: Confirms row-level HTMX BOM status buttons with `hx-swap-oob` readiness counters, native `<dialog>` substitution modal, and the 3-stage Active Bench Checklist with tappable `[○ R1]` / `[✓ R2]` reference designator chips (`sgtm`).
- **Stop 92 (`Ticket 05 Resolved: All 5 Wayfinder Decisions Complete`)**: Resolves Ticket 05. Every decision on the Wayfinder map is now complete!

---

## 6. Session 7: Synthesizing the Specification (`/to-spec`) (Stops 93–97)

**Session Trace**: [`07-99e2fb73-conversation.md`](../artifacts/traces/audio_inventory_skilled/07-99e2fb73-conversation.md) (`6m 47s`, 23 steps)  
**Specification**: [`.scratch/audio-inventory/spec.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/spec.md)

- **Stop 93 (`Session Cleared`) & Stop 94 (`Submit Command: /to-spec`)**:
  In a fresh session, the operator runs:
  ```text
  /to-spec
  ```
- **Stops 95 → 96 (`Confirm Specification Outline` → `go ahead`)**:
  `agy` reads `GLOSSARY.md`, `ADR 0001`, `map.md`, and all 5 resolved decision files (`01`–`05`), presents a synthesis outline covering 42 user stories and 6 architectural decision sections, and the operator replies `go ahead`.
- **Stop 97 (`Specification Published (.scratch/audio-inventory/spec.md)`)**:
  `agy` writes [`.scratch/audio-inventory/spec.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/spec.md) marked `Labels: ready-for-agent`.

---

## 7. Session 8: Slicing Vertical Implementation Tickets (`/to-tickets`) (Stops 98–102)

**Session Trace**: [`08-d4487a2d-conversation.md`](../artifacts/traces/audio_inventory_skilled/08-d4487a2d-conversation.md) (`12m 21s`, 54 steps)  
**Tickets Directory**: [`.scratch/audio-inventory/issues/`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/issues/)

- **Stop 98 (`Session Cleared`) & Stop 99 (`Submit Command: /to-tickets`)**:
  In a fresh session, the operator runs:
  ```text
  /to-tickets
  ```
- **Stops 100 → 101 (`Confirm 8 Vertical-Slice Implementation Tickets` → `go ahead`)**:
  `agy` reads `spec.md` and `.agents/skills/to-tickets/SKILL.md`, proposes 8 vertical-slice implementation tickets with explicit blocking dependencies, and the operator confirms with `go ahead`.
- **Stop 102 (`8 Implementation Tickets Published`)**:
  `agy` archives the 6 Wayfinder decision records into `.scratch/audio-inventory/decisions/` and writes 8 actionable implementation tickets into `.scratch/audio-inventory/issues/`:
  1. [`01-workspace-foundation-spec-parser-and-storage-locations.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/issues/01-workspace-foundation-spec-parser-and-storage-locations.md) *(Blocked by: None)*
  2. [`02-component-catalog-parametric-search-and-stock-lots.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/issues/02-component-catalog-parametric-search-and-stock-lots.md) *(Blocked by: 01)*
  3. [`03-kit-builds-bom-matching-and-stock-fulfillment.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/issues/03-kit-builds-bom-matching-and-stock-fulfillment.md) *(Blocked by: 02)*
  4. [`04-bom-substitutions-partial-allocations-line-splitting-and-cloning.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/issues/04-bom-substitutions-partial-allocations-line-splitting-and-cloning.md) *(Blocked by: 03)*
  5. [`05-active-bench-checklist-attachments-and-bench-logs.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/issues/05-active-bench-checklist-attachments-and-bench-logs.md) *(Blocked by: 03)*
  6. [`06-staged-csv-and-clipboard-imports.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/issues/06-staged-csv-and-clipboard-imports.md) *(Blocked by: 03)*
  7. [`07-multi-build-shopping-list-exports-and-order-receiving.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/issues/07-multi-build-shopping-list-exports-and-order-receiving.md) *(Blocked by: 03)*
  8. [`08-unified-data-directory-zip-backup-and-restore.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/issues/08-unified-data-directory-zip-backup-and-restore.md) *(Blocked by: 05)*

---

## 8. Session 9: Autonomous Vertical-Slice TDD & Two-Axis `/code-review` (`/implement-spec`) (Stops 103–107)

**Session Trace**: [`09-980f0ef9-conversation.md`](../artifacts/traces/audio_inventory_skilled/09-980f0ef9-conversation.md) (plus subagent traces `10`–`20`, 203 orchestrator steps, 20 `invoke_subagent` calls)

### Stop 103 (`Session Cleared`) & Stop 104 (`Submit Command: /implement-spec`)
After `/clear`, the operator issues a single command with **zero further human intervention** for the entire build and code review:

```text
/implement-spec
```

### Stop 105 (`All 8 Tickets Implemented -> Launch Parallel /code-review`)
Here is what the `/implement-spec` orchestrator executes autonomously:

1. **Git Initialization & Integration Branch**:
   - Initializes the git repository, commits the initial spec and tickets on `main`, and checks out the integration branch `feat/audio-inventory`.
2. **Codebase Exploration Subagent** ([Trace `10`](../artifacts/traces/audio_inventory_skilled/10-59aedb73-none.md)):
   - Inspects `prototype-ui.html` and verifies the exact `turso` (`0.9.0-pre.2`), `axum` (`0.8`), and `maud` (`0.27`) APIs before ticket work begins.
3. **Sequential Foundation Waves (Tickets `01` → `02` → `03`)**:
   - For each ticket, the orchestrator claims the ticket on `feat/audio-inventory`, creates an isolated git worktree (`/tmp/audio-inventory-wt/ticket-01`, `ticket-02`, `ticket-03`), spawns an Implementer subagent following strict Red-Green-Refactor `/tdd`, and then spawns a Merger subagent ([Traces `11`–`13`](../artifacts/traces/audio_inventory_skilled/11-91b57720-none.md)) to merge the branch into `feat/audio-inventory` and verify `cargo test`.
4. **Parallel 4-Way Frontier Wave (Tickets `04`, `05`, `06`, `07`)**:
   - Once Ticket `03` merges, Tickets `04`, `05`, `06`, and `07` all become unblocked simultaneously!
   - The orchestrator creates 4 separate git worktrees (`ticket-04`, `ticket-05`, `ticket-06`, `ticket-07`) and launches **4 parallel Implementer subagents** at the same time, merging each one (`ticket-04` → `ticket-07` → `ticket-06` → `ticket-05`, [Traces `14`–`17`](../artifacts/traces/audio_inventory_skilled/14-74030cf8-none.md)) as it finishes and resolving merge conflicts cleanly against ` cargo test`.
5. **Final Wave (Ticket `08`)**:
   - Once Ticket `05` merges, Ticket `08` (`.zip` backup & restore) is unblocked, implemented in `/tmp/audio-inventory-wt/ticket-08`, and merged via [Trace `18`](../artifacts/traces/audio_inventory_skilled/18-efe5e8a6-none.md).

### Stop 106 (`Code Review & Fixes Complete`) & Stop 107 (`Implementation & Code Review Summary (41 Passing Tests)`)
Before declaring `/implement-spec` done, the skill automatically invokes `/code-review` across `main...feat/audio-inventory` using **two parallel review subagents**:

- **Standards Axis Subagent** ([Trace `19`](../artifacts/traces/audio_inventory_skilled/19-8ba90db0-none.md)): Audits the codebase against `GLOSSARY.md`, `ADR 0001`, `.agents/skills/codebase-design/SKILL.md`, and Fowler code smells.
- **Spec Axis Subagent** ([Trace `20`](../artifacts/traces/audio_inventory_skilled/20-ee6bf12d-none.md)): Audits every requirement and formula in `spec.md` and `issues/01..08.md` against the Rust implementation.

**What the Two-Axis Code Review Caught (and Fixed Automatically)**:
1. **Schema & Glossary Alignment (`migrations/001_initial_schema.sql`, `src/builds.rs`)**: Renamed legacy table/column identifiers (`kit_log_entries` → `bench_log_entries`, `checked_designators_json` → `installed_designators_json`) to strictly match `GLOSSARY.md` and `spec.md`.
2. **Installed Stock Permanence Invariant (`src/builds.rs`)**: Fixed `transition_bom_line_status` and `substitute_bom_line_component` so parts already marked `Installed` (soldered onto a PCB) are never refunded to a `Stock Lot` if a line is later edited or re-linked.
3. **Transactional Atomicity in Staged CSV Commits (`src/imports.rs`, `src/catalog.rs`)**: Wrapped `commit_component_import_batch` and `commit_bom_import_batch` in explicit `BEGIN` / `COMMIT` / `ROLLBACK` transactions using `upsert_or_merge_stock_lot_tx`.
4. **HTMX `hx-patch` Alignment (`src/web.rs`)**: Updated inline BOM status buttons to emit `hx-patch="/builds/{kit_id}/bom/{bom_id}/status"` as specified in Decision 05.
5. **Dedicated `attachments` Adapter (`src/attachments.rs`, `src/backup.rs`)**: Extracted shared attachment path sanitization and `resolve_effective_data_dir` into [`src/attachments.rs`](../experiments/audio_inventory_skilled/src/attachments.rs) and deduplicated stock lot metadata merging in [`src/catalog.rs`](../experiments/audio_inventory_skilled/src/catalog.rs).

The orchestrator spawns a single **Code Review Fixer Implementer** in `/tmp/audio-inventory-wt/ticket-review-fixes`, merges the fixes into `feat/audio-inventory`, runs `cargo test` (**41 passed, 0 failed, 0 warnings**), marks `spec.md` and all 8 tickets `Status: resolved`, and cleans up all temporary worktrees (Stop 107).

---

## 9. Session 10: Live Browser Testing via Chrome DevTools MCP (Stops 108–111)

**Session Trace**: [`21-73c7d66f-please-test-this-site-in-a-web.md`](../artifacts/traces/audio_inventory_skilled/21-73c7d66f-please-test-this-site-in-a-web.md) (`23m 12s`, 191 steps, 67 `chrome-devtools-mcp` calls)

### Stop 108 (`Session Cleared`) & Stop 109 (`Submit Prompt: please test this site in a web browser`)
Just as in Run 1, the operator clears the session and prompts:

```text
please test this site in a web browser
```

### Stop 110 (`Browser Testing & Live HTMX Fixes Complete`) & Stop 111 (`Browser Verification Summary Complete`)
`agy` builds and launches `./target/debug/audio_inventory` against an isolated test data directory, opens `http://127.0.0.1:3000/` in Chrome via `chrome-devtools-mcp`, and systematically exercises every tab:
1. **Component Catalog & Storage Locations**: Creates hierarchical storage locations (`Resistor Binder A`), tests parametric shorthand search (`4k7`), creates components (`TL072 Dual Op-Amp DIP-8`), and tests inline and drawer-based stock lot steppers.
2. **Kit Build BOM & Stock Allocation**: Creates `Klon Centaur Overdrive (#1)`, adds BOM lines (`R1, R2, R3`, `IC1, IC2`), creates a catalog component directly from an unlinked BOM line, opens the native `<dialog>` substitution modal, and transitions BOM lines between `Unallocated`, `Reserved`, `Picked`, and `Installed`.
3. **Active Bench Checklist**: Toggles individual per-designator soldering chips (`[○ R1]` → `[✓ R1]`) and posts timestamped **Bench Log Entries**.
4. **Shopping List, Staged CSV Import & `.zip` Backup**: Verifies multi-build **Net Shortfall** calculation, supplier quick-order CSV preview, Mouser CSV import staging & reconciliation, and `.zip` backup/restore settings.

**Four Live Browser / HTMX Integration Bugs Caught & Fixed in [`src/web.rs`](../experiments/audio_inventory_skilled/src/web.rs) and [`assets/styles.css`](../experiments/audio_inventory_skilled/assets/styles.css)**:
1. **Unscoped `hx-trigger` on the Catalog Filter Ribbon Wiping Out Inspector Drawer Inputs**:
   - In [`src/web.rs`](../experiments/audio_inventory_skilled/src/web.rs), the catalog filter form originally used `hx-trigger="input changed delay:150ms from:input, change from:select, search, submit"`. Because `from:input` matched *every* `<input>` on the page, typing inside the right-hand Inspector Drawer triggered a `/catalog` refresh 150ms later and wiped out the user's partially typed form!
   - **Fix**: Scoped the trigger selectors to descendants of the filter form using `from:find input, change from:find select`.
2. **HTML5 Parser Stripping Bare Out-of-Band `<tr hx-swap-oob="true">` Fragments**:
   - When updating a Stock Lot or BOM Line from the Inspector Drawer, `render_component_row_oob` and `render_bom_line_row_oob` returned `<aside class="inspector-card">...</aside><tr id="..." hx-swap-oob="true">...</tr>`. Because the fragment started with `<aside>`, the browser's HTML5 parser ran in *"in body"* mode and stripped bare `<tr>`, `<th>`, and `<td>` tags before HTMX 2.0.4 could process `hx-swap-oob`!
   - **Fix**: Wrapped out-of-band `<tr>` fragments inside `<template><tr id="..." hx-swap-oob="true">...</tr></template>`, which HTMX 2.0.4 unwraps cleanly.
3. **340px Inspector Drawer Side-by-Side Form Input Overflow**:
   - Browser screenshots revealed that `.form-row-2` inputs inside the 340px right-hand Inspector Drawer overflowed horizontally due to default intrinsic `<input>` minimum widths.
   - **Fix**: Added `grid-template-columns: minmax(0, 1fr)` and `min-inline-size: 0; inline-size: 100%` rules in [`assets/styles.css`](../experiments/audio_inventory_skilled/assets/styles.css).
4. **Stale `aria-selected` State on Top Workbench Tabs During HTMX Partial Navigation**:
   - Navigating across top-level tabs via `hx-target="#app-root"` updated the workspace content without updating the active tab pill in `<nav id="workbench-subnav">`.
   - **Fix**: Updated `render_page_or_partial` in [`src/web.rs`](../experiments/audio_inventory_skilled/src/web.rs) to include an out-of-band `<nav id="workbench-subnav" hx-swap-oob="true">` whenever `HX-Target == "app-root"`.

After rebuilding and restarting the server, `agy` re-verifies all flows in Chrome (confirming zero console errors and clean in-place HTMX swaps) and runs `cargo test` (**41 passed, 0 failed**).

---

## 10. Key Codelab Takeaways: Baseline Run (Run 1) vs. Skilled Run (Run 2)

| Dimension | Run 1 — Baseline Run (`audio_inventory_baseline`) | Run 2 — Skilled Run (`audio_inventory_skilled`) |
| :--- | :--- | :--- |
| **Opening Prompt** | `"I'd like to build a web application to track my audio electronics components inventory and DIY kit builds."` | `"/wayfinder I'd like to build a web application to track my audio electronics components inventory and DIY kit builds."` |
| **Installed Repo Skills** | None (`0` skills) | `mattpocock/skills` (`38` skills) |
| **Requirements Discovery** | 4 high-level multiple-choice questions in 1 session | 11 Map questions + 22 deep-dive domain/architecture questions across 6 `/wayfinder` sessions |
| **Domain Model Depth** | Flat location string; single quantity per component; 3-state allocation (`On Hand`, `Allocated`, `Available`) | Self-referential **Storage Location** tree; multi-location **Stock Lots** with lot merging; 9 categories (incl. **Modules & Sub-Assemblies**); verbatim `raw_spec_text` + canonical values; 5-state **Fulfillment Status** (`Unallocated`, `Reserved`, `Picked`, `Installed`, `Included in Kit`); 1-click **Split BOM Line** & **Clone Kit Build** |
| **UI Exploration** | Agent picked a single layout unprompted | Generated interactive [`prototype-ui.html`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/prototype-ui.html) with Variants A, B, and C for user selection before coding |
| **Durable Documentation** | None (`0` files; all rules locked in chat history) | [`GLOSSARY.md`](../experiments/audio_inventory_skilled/GLOSSARY.md), [`ADR 0001`](../experiments/audio_inventory_skilled/docs/adr/0001-rust-htmx-turso-stack.md), [`map.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/map.md), 5 decision records, [`spec.md`](../experiments/audio_inventory_skilled/.scratch/audio-inventory/spec.md), 8 tickets |
| **Implementation Execution** | Single 14-minute monolithic turn writing all files at once | 8 vertical-slice tickets implemented in isolated git worktrees (including 4 parallel subagents for Tickets `04`–`07`) |
| **Automated Tests** | `0` automated test files | **41 Rust unit & integration tests** running against ephemeral `:memory:` Turso databases |
| **Code Review** | Required manual operator `/clear` → `Please review the codebase` → `please fix` | Built-in parallel 2-axis `/code-review` (**Standards** + **Spec** subagents) automatically executed and fixed before completion |
| **Readiness for Cloud Migration (Run 3)** | Raw synchronous SQLite queries coupled directly to Express routes | 5 deep domain modules (`spec_parser`, `catalog`, `builds`, `imports`, `shopping_list`) + complete `GLOSSARY.md`, `ADR`, and `spec.md` ready for **Run 3** (**Production Run**) |

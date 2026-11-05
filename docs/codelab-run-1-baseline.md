# Codelab Script — Run 1: The Baseline Run (`audio_inventory_baseline`)

This document chronicles **Run 1** (**Baseline Run**) of the three-run Antigravity Cloud Website Experiment and serves as the foundational script for the **Baseline Run** **Codelab**.

In this run, we start in an empty directory outside this repository (`~/Demos/audio-inventory-experiment/audio_inventory_baseline`) with **no repository skills installed** (`mattpocock/skills` and `google/skills` are absent), guided only by two ambient customizations in the user profile:

1. **Global Clarifying-Question Rule** (`~/.gemini/config/AGENTS.md`):
   ```markdown
   You are a helpful assistant.

   When something is unclear, please ask clarifying questions, one at a time.
   ```
2. **Global Modern Web Guidance Plugin** (`~/.gemini/config/plugins/modern-web-guidance-plugin/`): An ambient UI/HTML/CSS/accessibility plugin installed in the user's global Antigravity config.

---

## 1. Run Overview & Artifacts

| Artifact / Property | Details |
| :--- | :--- |
| **Experiment Run** | **Baseline Run** (`audio_inventory_baseline`) |
| **Model** | Gemini 4 Argon (Medium) |
| **Terminal Cast** | [`recordings/final/01-baseline.cast`](../recordings/final/01-baseline.cast) (raw: `43m 25s`, 1x idle-clamped: `30m 08s`, adaptive warped playback: `3m 01s` across 17 chapter stop points) |
| **Stop-Point Spec** | [`scripts/markers/01-baseline.json`](../scripts/markers/01-baseline.json) |
| **Interactive Cast Player** | [`prototypes/01-baseline-cast-player.html`](../prototypes/01-baseline-cast-player.html) |
| **Session Traces** | 1. [`01-4c874de2-i-d-like-to-build-a-web.md`](../artifacts/traces/audio_inventory_baseline/01-4c874de2-i-d-like-to-build-a-web.md) (`27m 00s`, 47 steps, 24 tool calls)<br>2. [`02-9a12e573-please-review-the-codebase.md`](../artifacts/traces/audio_inventory_baseline/02-9a12e573-please-review-the-codebase.md) (`15m 06s`, 118 steps, 75 tool calls) |
| **Synced Application Code** | [`experiments/audio_inventory_baseline/`](../experiments/audio_inventory_baseline/) ("PatchBay & Bench" — Node.js `express` + built-in `node:sqlite` (`DatabaseSync`) + vanilla HTML/CSS/JS) |
| **Durable Domain / ADR Docs Produced** | **None** (`0` `GLOSSARY.md` files, `0` `docs/adr/` records, `0` specification or ticket files) |

---

## 2. Stop-Point Timeline at a Glance

The **Terminal Cast** ([`01-baseline.cast`](../recordings/final/01-baseline.cast)) is segmented into **17 chapter stop points** across two Antigravity sessions separated by `/clear`:

| Stop # | Marker Label | Event # | 1x Time | Segment Speed | Warped Time | Phase |
| :---: | :--- | :---: | :---: | :---: | :---: | :--- |
| **1** | `1. Launch agy` | `#0010` | `0:01.9` | `2x` (`0.9s`) | `0:00.9` | Session 1 — Launch & Prompt |
| **2** | `2. Ready for Initial Prompt` | `#0020` | `0:05.5` | `2x` (`1.8s`) | `0:02.7` | Session 1 — Launch & Prompt |
| **3** | `3. Initial Prompt` | `#0022` | `0:06.5` | `2x` (`0.5s`) | `0:03.2` | Session 1 — Launch & Prompt |
| **4** | `4. Q1: Storage & Stack` | `#0242` | `0:27.2` | `2x` (`10.4s`) | `0:13.6` | Session 1 — Interactive Q&A |
| **5** | `5. Q2: Key Features` | `#0359` | `0:40.1` | `2x` (`6.5s`) | `0:20.1` | Session 1 — Interactive Q&A |
| **6** | `6. Q2: Features & Write-In` | `#0414` | `1:11.1` | `2x` (`15.5s`) | `0:35.6` | Session 1 — Interactive Q&A |
| **7** | `7. Q3: Part Allocation Model` | `#0573` | `1:26.3` | `2x` (`7.6s`) | `0:43.1` | Session 1 — Interactive Q&A |
| **8** | `8. Q4: Audio DIY Focus` | `#0705` | `1:40.3` | `2x` (`7.0s`) | `0:50.1` | Session 1 — Interactive Q&A |
| **9** | `9. Q4: Options Selected` | `#0714` | `1:47.0` | `2x` (`3.4s`) | `0:53.5` | Session 1 — Interactive Q&A |
| **10** | `10. Build Execution Complete` | `#9051` | `15:48.1` | `29x` (`29.0s`) | `1:22.5` | Session 1 — One-Shot Build |
| **11** | `11. Build Summary` | `#9792` | `16:05.6` | `2x` (`8.8s`) | `1:31.3` | Session 1 — One-Shot Build |
| **12** | `12. Session Cleared` | `#9815` | `16:09.6` | `2x` (`2.0s`) | `1:33.3` | Context Reset (`/clear`) |
| **13** | `13. Codebase Review Prompt` | `#9817` | `16:10.6` | `2x` (`0.5s`) | `1:33.8` | Session 2 — Code Review |
| **14** | `14. Review Findings` | `#11835` | `17:23.5` | `3x` (`24.3s`) | `1:58.1` | Session 2 — Code Review |
| **15** | `15. Fixes Verified` | `#17118` | `25:01.3` | `16x` (`28.6s`) | `2:26.7` | Session 2 — Fix Implementation |
| **16** | `16. Browser Test Prompt` | `#17158` | `25:10.6` | `2x` (`4.6s`) | `2:31.3` | Session 2 — Live Browser Test |
| **17** | `17. Browser Verification Complete` | `#20210` | `30:07.6` | `10x` (`29.7s`) | `3:01.0` | Session 2 — Live Browser Test |

---

## 3. Session 1: Interactive Q&A & One-Shot Prototype Build

**Session Trace**: [`artifacts/traces/audio_inventory_baseline/01-4c874de2-i-d-like-to-build-a-web.md`](../artifacts/traces/audio_inventory_baseline/01-4c874de2-i-d-like-to-build-a-web.md)

### Phase 1.1 — Launching `agy` and Submitting the Prompt (Stops 1–3)

#### Stop 1 (`1. Launch agy`) & Stop 2 (`2. Ready for Initial Prompt`)
We start in a completely empty directory (`~/Demos/audio-inventory-experiment/audio_inventory_baseline`) and launch the Antigravity CLI:

```bash
agy
```

Antigravity 1.5.1 starts up with **Gemini 4 Argon (Medium)** selected.

#### Stop 3 (`3. Initial Prompt`)
We paste the exact same opening prompt that will later kick off the **Skilled Run** (Run 2), deliberately omitting any mention of tech stack, architecture, or Google Cloud:

```text
I'd like to build a web application to track my audio electronics components inventory and DIY kit builds.
```

**What `agy` does first**:
- Runs `ls -la` (Step 1) and sees the directory is completely empty.
- Because the user's global `~/.gemini/config/AGENTS.md` states *"When something is unclear, please ask clarifying questions, one at a time"*, `agy` invokes the `ask_question` tool rather than immediately guessing requirements.

---

### Phase 1.2 — The Four Clarifying Questions (Stops 4–9)

Notice how `agy` conducts a structured 4-question discovery interview using interactive multiple-choice menus (`ask_question`), adapting Q3 directly in response to a custom write-in answer in Q2.

#### Stop 4 (`4. Q1: Storage & Stack`)
`agy` asks how data should be persisted:

> **Question 1/1**: *How would you like your inventory and kit build data to be stored?*
> 1. `(Recommended) Local backend (Node.js + SQLite) for persistent file-based storage on your machine`
> 2. `Browser-only (LocalStorage / IndexedDB) with JSON/CSV import & export — zero server setup`
> 3. `Cloud-backed (e.g., Firebase / Firestore) so you can access it across multiple devices`
> 4. `Write-in...`

- **Operator Action**: Selects **Option 1** (`(Recommended) Local backend (Node.js + SQLite) for persistent file-based storage on your machine`).
- **Codelab Callout**: Without a stack preference or architecture skill, `agy` recommends **Node.js + SQLite** as its default local full-stack setup.

#### Stop 5 (`5. Q2: Key Features`) & Stop 6 (`6. Q2: Features & Write-In`)
Next, `agy` asks a multi-select question about domain features:

> **Question 1/1**: *Which key features would you like included for managing components and DIY kit builds?*
> - `[x]` 1. `(Recommended) BOM (Bill of Materials) matching — check which parts you have in stock vs. need to order for a kit`
> - `[ ]` 2. `(Recommended) Automatic stock deduction when allocating or completing a kit build`
> - `[x]` 3. `Build status & progress tracking (e.g., Planning, Parts Ordered, Soldering, Testing, Completed) plus build notes/logs`
> - `[x]` 4. `CSV / BOM import & export (e.g., importing a Mouser/Tayda/Eurorack BOM or exporting shopping lists)`
> - `[x]` 5. `Storage location tracking (e.g., binder, drawer bin, box label) and low-stock alerts`
> - `[x]` 6. `Write-in...` → **`Allocate parts to a project`**

- **Operator Action**: Toggles options **1, 3, 4, and 5** (leaving option 2 unchecked because we want explicit part allocation rather than immediate deduction) and types **`Allocate parts to a project`** into the **Write-in** field (Stop 6).

#### Stop 7 (`7. Q3: Part Allocation Model`)
Because we wrote in *"Allocate parts to a project"*, `agy` immediately follows up to disambiguate the inventory state model:

> **Question 1/1**: *How should allocating parts to a project work?*
> 1. `(Recommended) Track 'Total On Hand' vs. 'Allocated' vs. 'Available' — allocating reserves parts for a project (reducing Available stock), and you can un-allocate or finalize/consume them when built`
> 2. `Immediately deduct from stock when allocated to a project (with the option to return them to stock if cancelled)`
> 3. `Allow partial or per-BOM-line allocation (e.g., allocate the parts you already have into a project baggie while waiting on the rest)`
> 4. `Write-in...`

- **Operator Action**: Selects **Option 1** (`Track 'Total On Hand' vs. 'Allocated' vs. 'Available'`).
- **Codelab Callout**: This is a crucial domain invariant (`qty_available = qty_on_hand - qty_allocated` for unconsumed BOM items). In the **Baseline Run**, this invariant is captured *only* in ephemeral chat history and inline SQL queries—no `GLOSSARY.md` or ADR is written to disk.

#### Stop 8 (`8. Q4: Audio DIY Focus`) & Stop 9 (`9. Q4: Options Selected`)
Finally, `agy` asks about the specific audio DIY sub-domains to tailor categories, footprints, and seed data:

> **Question 1/1**: *What types of DIY audio projects do you mainly build? (This helps tailor the default component categories, footprints, and fields.)*
> - `[x]` 1. `(Recommended) Eurorack / Modular Synth DIY (Thonkiconn jacks, pots, op-amps, SMD/THT passives, power headers)`
> - `[x]` 2. `(Recommended) Guitar Pedals / Stompboxes (3PDT switches, enclosures, audio/log pots, germanium/silicon transistors & diodes)`
> - `[ ]` 3. `Tube / Solid-State Amps & Hi-Fi Audio (high-voltage caps, tubes, transformers, terminal strips)`
> - `[ ]` 4. `Pro Audio / 500-Series / Studio Gear (XLR, audio transformers, stepped attenuators, VCA chips)`
> - `[ ]` 5. `Write-in...`

- **Operator Action**: Selects **Options 1 and 2** (Eurorack / Modular Synth DIY + Guitar Pedals / Stompboxes) and presses `Enter` (Stop 9).

---

### Phase 1.3 — The One-Shot Application Build (Stops 10–11)

#### Stop 10 (`10. Build Execution Complete`)
Once Q4 is answered, `agy` builds the entire application in a single uninterrupted turn (~14 minutes of wall-clock execution, compressed to `29s` at `29x` in the cast player):

1. **Consults Global Web Guidance (`modern-web-guidance`)**:
   - Reads `~/.gemini/config/plugins/modern-web-guidance-plugin/skills/modern-web-guidance/SKILL.md`.
   - Runs `npx -y modern-web-guidance@latest` to retrieve best-practice guides for `dialog`, `light-dismiss-a-dialog`, `prevent-page-scroll-under-modal-dialog`, `responsive-table`, `dark-mode`, `dark-mode-toggle`, `validate-input-after-interaction`, `forms`, `html`, `css`, and `accessibility`.
2. **Probes Local Node Runtime**:
   - Checks `node -v && npm -v` (Node.js v26).
   - Verifies that Node's built-in `node:sqlite` (`DatabaseSync`) is available, eliminating native C++ addon compilation (`better-sqlite3`).
3. **Creates the "PatchBay & Bench" Application Files**:
   - [`package.json`](../experiments/audio_inventory_baseline/package.json) — Minimal manifest with a single dependency (`express: ^5.1.0`).
   - [`db.js`](../experiments/audio_inventory_baseline/db.js) — SQLite schema (`components`, `projects`, `bom_items`, `build_logs`), electronic value normalization (`normalizeElectronicValue` converting `4k7` → `4.7k` and `0.1uF` ↔ `100nF`), footprint normalization (`normalizeFootprint`), a fuzzy BOM-to-Component auto-matcher (`findBestComponentMatch`), and rich seed data:
     - **26 realistic audio Components** across 5 categories (`Resistors`, `Capacitors`, `Semiconductors & ICs`, `Potentiometers & Encoders`, `Jacks, Switches & Hardware`) including `TL072CP`, `TL074CN`, `TL072CDR` (SOIC-8), `LM13700N`, `SSI2144`, `1N34A` germanium diodes, `Thonkiconn PJ398SM` jacks, and `3PDT` footswitches.
     - **3 DIY Kit Projects**:
       1. *Mutable Instruments Ripples (2020 Liquid Filter)* (`Eurorack Module`, status `Soldering`, 8 BOM lines)
       2. *Klon Centaur Transparent Overdrive (125B)* (`Guitar Pedal`, status `Planning`, 8 BOM lines including an unlinked `TC1044SCPA` charge pump IC to showcase shortage detection)
       3. *AI Synthesis AI004 OTA Vintage Filter* (`Eurorack Module`, status `Parts Ordered`, 4 BOM lines)
   - [`server.js`](../experiments/audio_inventory_baseline/server.js) — Express 5 REST API serving `/api/overview`, `/api/components` (CRUD, quick `±` stock adjustment, CSV import), `/api/projects` (CRUD, BOM line management, auto-match, bulk allocate, release, finalize/consume build, build logs), and `/api/shopping-list`.
   - [`public/index.html`](../experiments/audio_inventory_baseline/public/index.html), [`public/styles.css`](../experiments/audio_inventory_baseline/public/styles.css), [`public/app.js`](../experiments/audio_inventory_baseline/public/app.js) — Single-page workbench UI with 3 tabs (*Components Inventory*, *DIY Kit Builds & BOMs*, *Shopping List & Shortages*), native `<dialog closedby="any">` modals, CSS `@layer` cascade, `@container` responsive tables, and two-state light/dark theme toggle.
4. **Starts the Server & Runs a Smoke Check**:
   - Launches `node server.js` in the background on `http://localhost:3000`.
   - Runs an inline `node -e` fetch script against `/api/overview`, `/api/projects/1`, and `/api/projects/2` to verify the seeded database and API responses.

#### Stop 11 (`11. Build Summary`)
`agy` presents its build summary, detailing the database schema, REST endpoints, seed data, and the specific `modern-web-guidance` rules it applied.

> **Codelab Takeaway (End of Session 1)**:
> In one shot, `agy` produced a remarkably feature-complete local prototype. **However:**
> - There is **no test suite** (`npm test` does not exist; only a one-off `node -e` smoke check was run).
> - There is **no domain glossary (`GLOSSARY.md`) or ADR (`docs/adr/`)** recording the `On Hand / Allocated / Available` state machine or value-normalization rules.
> - Because `db.js` and `server.js` are tightly coupled with raw synchronous SQLite statements, swapping persistence to a cloud database later would require rewriting the data layer.
> - And as we are about to see in Session 2, writing ~2,500 lines of JS/HTML/CSS in a single unverified pass left subtle domain bugs in the code.

---

## 4. Context Reset: `/clear` (Stop 12)

#### Stop 12 (`12. Session Cleared`)
To test how well the codebase stands on its own without the original session's conversational memory, the operator clears the session:

```text
/clear
```

The terminal resets to a fresh Antigravity CLI banner (`Session 2`), while `node server.js` continues running in the background.

---

## 5. Session 2: Codebase Review, Bug Fixing & Live Browser Verification

**Session Trace**: [`artifacts/traces/audio_inventory_baseline/02-9a12e573-please-review-the-codebase.md`](../artifacts/traces/audio_inventory_baseline/02-9a12e573-please-review-the-codebase.md)

### Phase 2.1 — Reviewing the Freshly Built Codebase (Stops 13–14)

#### Stop 13 (`13. Codebase Review Prompt`)
In the fresh session, the operator prompts:

```text
Please review the codebase
```

#### Stop 14 (`14. Review Findings`)
`agy` lists the workspace files, reads `package.json`, `db.js`, `server.js`, `public/index.html`, `public/styles.css`, and `public/app.js`, and produces a structured review identifying **8 concrete bugs, edge cases, and security/performance issues** that slipped into the one-shot build:

1. **Multi-Line BOM Stock Readiness Over-Counting (`db.js`)**:
   - `getAllProjects()` and `getProjectDetail()` checked `b.qty_available >= needed` independently per BOM line without deducting shared component usage across earlier lines in the same project (or across projects in `getShoppingList()`).
2. **Broken Allocation Invariants When Stock Is Reduced (`server.js`)**:
   - Reducing `qty_on_hand` via `PUT /api/components/:id` or `PATCH /api/components/:id/stock` below a component's currently allocated quantity left `SUM(qty_allocated) > qty_on_hand`. Similarly, `PUT /api/bom/:bomId` did not clamp `qty_allocated` when `qty_required` was lowered.
3. **Electronic Value Normalization Edge Cases (`db.js`)**:
   - `normalizeElectronicValue('4k7Ω')` turned `Ω` into `r` (`4k7r`), which failed the middle-decimal regex `/^(\d+)([rkmnup])(\d+)(f?)$/`.
   - Plain numeric ohms (`470`) normalized to `'470'` while `'470R'` and `'470Ω'` normalized to `'470r'`, preventing exact +60 value matches.
4. **Potential `javascript:` URL Injection & Missing Security Headers (`app.js`, `server.js`)**:
   - `project.doc_url` was interpolated into `<a href="${escapeHtml(project.doc_url)}">` without verifying an `http:` or `https:` protocol, and `server.js` lacked baseline security headers (`Content-Security-Policy`, `X-Content-Type-Options`, `X-Frame-Options`).
5. **Unhandled Promise Rejections in UI Event Listeners (`app.js`)**:
   - Async event handlers called `await api(...)` without `try/catch`, causing failed HTTP requests (`400`/`404`) to fail silently in the UI without updating the `#status-toast` live region.
6. **N+1 SQLite Queries (`db.js`)**:
   - `getAllProjects()` and `getShoppingList()` prepared and executed `SELECT * FROM bom_items WHERE project_id = ?` inside a per-project loop.
7. **CSV Parser Breaking on Quoted Multiline Notes (`app.js`)**:
   - `parseCsvText()` split on `/\r?\n/` before parsing quotes, corrupting CSV rows containing multiline quoted fields.
8. **Misaligned Mobile Table Column Labels on `#bom-table` (`styles.css`)**:
   - Because `#bom-table` has a checkbox `<td class="check-col">` as column 1 and `<th scope="row">` as column 2, the `@container (width < 680px)` CSS rules shifted mobile `::before` column labels off by one column.

---

### Phase 2.2 — Applying & Verifying the Fixes (Stop 15)

#### Stop 15 (`15. Fixes Verified`)
The operator prompts:

```text
please fix
```

`agy` retrieves the `responsive-table` and `security` guides from `modern-web-guidance`, edits `db.js`, `server.js`, `public/styles.css`, and `public/app.js` across 12 `edit_file` calls, and runs a verification command (Step 60):

```bash
node --check db.js && node --check server.js && node --check public/app.js && node -e "
const assert = require('node:assert/strict');
const { normalizeElectronicValue, getAllProjects, getProjectDetail, getShoppingList } = require('./db');

assert.equal(normalizeElectronicValue('4k7Ω'), '4.7k');
assert.equal(normalizeElectronicValue('470'), '470r');
assert.equal(normalizeElectronicValue('470R'), '470r');
assert.equal(normalizeElectronicValue('470.0Ω'), '470r');
assert.equal(normalizeElectronicValue('0.1uF'), 'cap:100000pf');
assert.equal(normalizeElectronicValue('100nF'), 'cap:100000pf');

const projects = getAllProjects();
assert.ok(projects.length >= 3);
const detail = getProjectDetail(projects[0].id);
assert.ok(detail && Array.isArray(detail.bom));
const shopping = getShoppingList();
assert.ok(Array.isArray(shopping.linked_items));
console.log('All assertions passed!');
"
```

---

### Phase 2.3 — Live Browser Testing via Chrome DevTools MCP (Stops 16–17)

#### Stop 16 (`16. Browser Test Prompt`)
Next, the operator asks `agy` to test the running web application in a real browser:

```text
please test this site in a web browser
```

*(Note: In the raw recording, a brief terminal keystroke hiccup `p Gleasetestthissiteinawebbrowser` occurs right as the prompt is entered—preserved as Stop 16 in the cast.)*

#### Stop 17 (`17. Browser Verification Complete`)
`agy` uses the `chrome-devtools-mcp` server to drive a live Chrome session against the app:

1. **Starts a Fresh Server Instance**:
   - Checks port 3000 (`lsof -i :3000`) and launches `PORT=3123 node server.js` so it runs the newly fixed `server.js` and `db.js` code.
2. **Navigates & Inspects Console / Network**:
   - Opens `http://localhost:3123` in Chrome (`navigate_page`), takes an accessibility tree snapshot (`take_snapshot`) and screenshot (`take_screenshot`), and inspects `list_console_messages` and `list_network_requests`.
3. **Catches & Fixes Two Live Browser Issues (`public/index.html`)**:
   - **Missing Favicon (`404 Not Found` on `/favicon.ico`)**: Chrome logged a network/console 404 error for `/favicon.ico`. `agy` adds an inline SVG data-URI `<link rel="icon" ...>` matching the header patchbay icon in [`public/index.html`](../experiments/audio_inventory_baseline/public/index.html).
   - **Chrome Autofill Heuristic Warning**: Chrome DevTools flagged `<input id="comp-name" name="name">` and `<input id="proj-name" name="name">` with an autofill heuristic warning because `autocomplete="off"` was missing on non-personal entity name fields. `agy` adds `autocomplete="off"` to both inputs in [`public/index.html`](../experiments/audio_inventory_baseline/public/index.html).
4. **Verifies Interactive Flows & Responsive Breakpoints**:
   - Reloads with cache disabled, clicks through the theme toggle, *Add Component* modal, *DIY Kit Builds & BOMs* tab, and *Shopping List & Shortages* tab.
   - Resizes the viewport to `480×900` (`resize_page`) to verify the `@container (width < 680px)` stacked mobile BOM cards, then restores `1280×800`.
   - Confirms **0 console errors, 0 DevTools issues, and 0 failed network requests**, and cleanly terminates the background server task.

---

## 6. Key Codelab Takeaways: Why the Baseline Run Matters

When presenting or walking learners through **Run 1**, emphasize these four observations:

1. **One Rule Goes a Long Way—Until It Doesn't**:
   - Simply having *"When something is unclear, please ask clarifying questions, one at a time"* in `~/.gemini/config/AGENTS.md` prevented `agy` from hallucinating requirements upfront and produced a thoughtful 4-question discovery interview.
2. **Ambient Global Plugins Shape Output Silently**:
   - Even with zero repo skills installed, `agy` discovered the user's global `modern-web-guidance` plugin and applied modern HTML `<dialog closedby="any">`, CSS `@layer`, and `@container` responsive table patterns.
3. **One-Shot Builds Hide Bugs & Skip Architecture Seams**:
   - Without `/tdd`, `/code-review`, or `/domain-modeling` skills, `agy` wrote ~2,500 lines of code in a single turn with **no automated test suite**, **no repository trait/interface seam** (SQLite queries are tightly coupled to route logic), and **8 logic/security/CSS bugs** that only surfaced when the operator manually ran `/clear` → `Please review the codebase` → `please fix` → `please test this site in a web browser`.
4. **Zero Durable Institutional Memory**:
   - After `/clear`, the next agent session had **no `GLOSSARY.md`**, **no ADRs (`docs/adr/`)**, and **no ticket history** explaining *why* parts are tracked as `On Hand` vs. `Allocated` vs. `Available`. Every future session has to reverse-engineer the domain model from raw SQL and JavaScript.

This sets the stage for **Run 2 (The Skilled Run)**, where installing [`mattpocock/skills`](https://github.com/mattpocock/skills) transforms the exact same initial prompt into a disciplined `/wayfinder` → `/to-spec` → `/to-tickets` → `/implement` engineering workflow.

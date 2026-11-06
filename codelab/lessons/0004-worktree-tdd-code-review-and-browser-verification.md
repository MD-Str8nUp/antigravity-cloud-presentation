---
title: "Worktree TDD, Two-Axis Code Review & Live Browser Verification"
page_title: "Lesson 0004 — Worktree TDD, Code Review & Browser Verification"
nav_id: "lesson-0004"
badge: "Skilled Run • Sessions 9–10 (02-skilled.cast)"
subtitle: "Inside the autonomous /implement-spec engine: isolated git worktrees, 4-way parallel subagents, Standards + Spec /code-review, and the 4 HTMX integration bugs caught by live Chrome DevTools testing."
card:
  title: "**Worktree TDD, Two-Axis Code Review & Live Browser Verification**"
  summary: "Walk through Sessions 9–10 of Run 2 (02-skilled.cast): how /implement-spec orchestrated 11 subagents across git worktrees, what 2-axis /code-review caught, and why browser testing still mattered."
---

## 1. Session 9: Autonomous Multi-Worktree Orchestration (`/implement-spec`) {#implement-spec}

At **Stop 104** of `02-skilled.cast`, the operator cleared the session (`/clear`) and typed a single command:

```text caption="Stop 104 of 02-skilled.cast — Launching autonomous implementation & review"
/implement-spec
```

For the next **71 minutes of 1x execution** (compressed to `71 seconds` in the warped cast player), the operator did not type a single keystroke.

Why could `/implement-spec` run autonomously for over an hour without drifting off course, whereas the 14-minute one-shot build in the **Baseline Run** drifted into 8 domain bugs?

Because the **orchestrator** (`Session 09`) never wrote application code in its own context window! Instead, it acted as a coordinator—spawning **11 fresh-context subagents** (`Traces 10–20`) in isolated git worktrees (`/tmp/audio-inventory-wt/ticket-01..08`), each armed with:
1. [`GLOSSARY.md`](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/experiments/audio_inventory_skilled/GLOSSARY.md) & [`docs/adr/0001-rust-htmx-turso-stack.md`](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/experiments/audio_inventory_skilled/docs/adr/0001-rust-htmx-turso-stack.md)
2. [`.scratch/audio-inventory/spec.md`](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/experiments/audio_inventory_skilled/.scratch/audio-inventory/spec.md)
3. One specific vertical-slice ticket (`issues/01..08.md`)
4. The `/tdd` Red-Green-Refactor discipline running against ephemeral `:memory:` Turso databases (`cargo test`)

```mermaid caption="Figure 4.1: Multi-worktree subagent waves orchestrated during Session 9 (Stops 104–107)."
flowchart LR
  W1["Wave 1–3 (Sequential):<br/>Tickets 01 → 02 → 03<br/>(Worktree TDD + Merger)"] --> W4["Wave 4 (4x Parallel!):<br/>Tickets 04, 05, 06, 07<br/>(4 Concurrent Worktrees)"]
  W4 --> W5["Wave 5:<br/>Ticket 08<br/>(ZIP Backup)"]
  W5 --> CR["Parallel 2-Axis /code-review:<br/>Trace 19 (Standards) +<br/>Trace 20 (Spec)"]
  CR --> FIX["Review Fixer Worktree<br/>(41 Passing Tests)"]
```

---

## 2. What the Parallel Two-Axis `/code-review` Caught Automatically {#two-axis-review}

Remember how in the **Baseline Run**, the operator had to manually run `/clear` → `Please review the codebase` → `please fix` to discover domain bugs?

In the **Skilled Run**, `/implement-spec` automatically invoked `/code-review` at **Stop 105**, spawning two independent review subagents in parallel across the `main...feat/audio-inventory` diff:

1. **Standards Axis Reviewer ([Trace 19](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/artifacts/traces/audio_inventory_skilled/19-8ba90db0-none.md))**: Audited the code against `GLOSSARY.md`, `ADR 0001`, and deep-module design principles (`/codebase-design`).
2. **Spec Axis Reviewer ([Trace 20](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/artifacts/traces/audio_inventory_skilled/20-ee6bf12d-none.md))**: Audited every user story, SQL column name, and state transition against `spec.md`.

Look at the **5 subtle discrepancies** those two review subagents caught and fixed *before* handing control back to the operator at **Stop 107**:

Table: Findings caught and fixed automatically by the parallel Standards + Spec `/code-review` in Session 9.
| Axis | Finding Caught | Automated Fix Applied in `ticket-review-fixes` |
| :---: | :--- | :--- |
| **Standards + Spec** | **Glossary & Schema Drift**: Subagent for Ticket 03 named a table `kit_log_entries` and column `checked_designators_json` instead of the canonical `GLOSSARY.md` / `spec.md` terms. | Renamed to `bench_log_entries` and `installed_designators_json` across SQL migrations and `src/builds.rs`. |
| **Spec** | **Installed Stock Permanence Invariant (`src/builds.rs`)**: Editing or substituting a BOM line that was already `Installed` (soldered onto a PCB) could refund parts to a `Stock Lot`. | Enforced strict permanence in `transition_bom_line_status` and `substitute_bom_line_component`: once `Installed`, stock is never returned to a lot. |
| **Spec** | **Transactional Atomicity in Staged CSV Commits (`src/imports.rs`)**: Batch CSV imports inserted rows in a loop without wrapping the batch in a single SQL transaction. | Wrapped `commit_component_import_batch` and `commit_bom_import_batch` in explicit `BEGIN` / `COMMIT` / `ROLLBACK` transactions. |
| **Spec** | **HTMX Verb Alignment (`src/web.rs`)**: Inline BOM status buttons used `hx-post` instead of `hx-patch="/builds/{kit_id}/bom/{bom_id}/status"`. | Added `.patch(...)` route support and updated the Maud button templates to `hx-patch`. |
| **Standards** | **Shallow Attachment Helper Duplication**: Path sanitization was duplicated between `web.rs` and `backup.rs`. | Extracted a deep `src/attachments.rs` module (`resolve_effective_data_dir`, `sanitize_attachment_filename`). |

---

## 3. Session 10: Why Live Browser Testing Still Matters (`chrome-devtools-mcp`) {#browser-testing}

At the end of Session 9 (**Stop 107**), all **41 Rust unit and integration tests** passed with zero warnings. Was the app 100% bug-free?

**Almost—but not quite!** Backend unit tests and HTTP status assertions don't run a real browser HTML5 parser, CSS layout engine, or client-side `htmx.min.js` event loop.

In **Session 10 (Stops 108–111)**, the operator ran `/clear` and gave `agy` the exact same final prompt as the Baseline Run:

```text caption="Stop 109 of 02-skilled.cast — Driving Chrome via Chrome DevTools MCP"
please test this site in a web browser
```

Using `chrome-devtools-mcp` ([Trace 21](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/artifacts/traces/audio_inventory_skilled/21-73c7d66f-please-test-this-site-in-a-web.md)), `agy` clicked through every workflow in a live Chrome instance and caught **4 real browser/HTMX integration bugs**:

### Browser Bug 1: Unscoped `hx-trigger="from:input"` Wiping Out the Inspector Drawer

In `src/web.rs`, the top catalog filter bar originally listened for `input changed delay:150ms from:input`. In HTMX, `from:input` listens to **every `<input>` in the entire document**—so typing a quantity inside the right-hand Inspector Drawer triggered a `/catalog` reload 150ms later and wiped out the drawer form!

```rust caption="experiments/audio_inventory_skilled/src/web.rs — Scoping HTMX triggers to the filter form with 'find'"
// ❌ BEFORE: Listened to every <input> and <select> on the entire page!
hx-trigger="input changed delay:150ms from:input, change from:select, search, submit"

// ✅ AFTER (Session 10 fix): Scoped strictly to descendants of the filter <form>:
hx-trigger="input changed delay:150ms from:find input, change from:find select, search, submit"
```

### Browser Bug 2: HTML5 Parser Stripping Bare Out-of-Band `<tr hx-swap-oob="true">` Tags

When updating a Stock Lot in the right-hand Inspector Drawer, the server returned `<aside class="inspector-card">...</aside>` followed by `<tr id="component-row-1" hx-swap-oob="true">...</tr>` so the main table row would update in place.

Wait—when a response fragment starts with `<aside>`, the browser's HTML5 parser parses the fragment in **"in body" mode**, which **silently strips bare `<tr>` and `<td>` tags** before HTMX 2.0.4 ever sees them! Wrapping the out-of-band row in `<template><tr ...>...</tr></template>` fixed the swap immediately:

```rust caption="experiments/audio_inventory_skilled/src/web.rs — Wrapping out-of-band table rows in <template> for HTML5 parsing"
fn render_component_row_oob(c: &ComponentSummary) -> Markup {
    let row_id = format!("component-row-{}", c.id);
    html! {
        // Wrapping in <template> prevents the browser's HTML5 fragment parser
        // from stripping bare <tr> tags when mixed with <aside> fragments:
        template {
            (render_component_row_inner(c, &row_id, Some("true")))
        }
    }
}
```

---

## 4. Interactive Exercise: Two-Axis Review & HTMX Browser Fixes {#exercise-0004}

Complete the HTMX and domain-invariant patterns verified during Sessions 9 and 10:

:::exercise id="ex-0004" title="Interactive Lab: HTMX Fragment Parsing & Scoped Triggers" badge="Skilled Run • Sessions 9–10"
Fill in the missing HTMX selector keyword, HTML5 wrapper tag, and permanent fulfillment state:

```rust caption="Scoping HTMX triggers and preserving out-of-band <tr> swaps"
// 1. Scope hx-trigger to inputs inside the current <form> rather than the whole page:
// hx-trigger="input changed delay:150ms from:{{blank:find|HTMX modifier that scopes selector to descendants of the current element|find|8}} input"

// 2. Wrap out-of-band <tr hx-swap-oob="true"> rows so the HTML5 parser doesn't strip them:
html! {
    {{blank:template|HTML5 element that holds inert table rows without stripping them|template|10}} {
        tr id=(row_id) hx-swap-oob="true" { /* ... */ }
    }
}

// 3. Once a BOM line reaches this Fulfillment Status, its parts are soldered on the PCB
// and are NEVER refunded to a Stock Lot if the line is later edited:
// Permanent status: FulfillmentStatus::{{blank:Installed|The 5-state Fulfillment Status representing soldered parts|Installed|11}}
```

:::prompts
hint: "Use `find` in `from:find input`, wrap out-of-band `<tr>` elements in `template`, and use `Installed` for permanently soldered BOM lines."
success: "Spot on! Those three details illustrate why combining automated Spec/Standards review with live Chrome DevTools testing is so effective."
:::
:::

---

## 5. Try It Yourself: Implement & Browser-Test a Vertical Slice {#verify-0004}

Take one vertical ticket from your own project (from Lesson 0003) and run the implementation + review + browser verification loop:

:::verify id="verify-0004" title="Hands-On Build & Verification Gate: TDD + Two-Axis Review + Browser Test" badge="Bring Your Own Idea • Sessions 9–10"
Run these steps in `agy` on your own project workspace:

```text caption="Building and verifying across fresh /clear sessions"
# 1. Implement a single ticket with TDD (or run /implement-spec for the whole graph):
/implement

# 2. In a fresh session, run the parallel Standards + Spec code review:
/clear
/code-review

# 3. If your project has a web UI, test it live in Chrome via Chrome DevTools MCP:
/clear
please test this site in a web browser
```

- [ ] Verify that your unit/integration test suite runs and passes from the CLI.
- [ ] Run `/code-review` after `/clear` and inspect the **Standards Axis** (glossary/architecture alignment) and **Spec Axis** (requirements compliance) reports side by side.
- [ ] Run `please test this site in a web browser` and check whether live DOM/CSS/network inspection catches any issues that backend tests missed!
:::

---

## 6. Retrieval Practice {#quiz-0004}

:::quiz id="quiz-0004" title="Retrieval Practice: Worktree TDD, Code Review & Browser Verification" badge="3 Questions • Equal-Length Options"
- question: "What are the two axes checked in parallel when `/code-review` runs at the end of `/implement-spec`?"
  wrong_hint: "Check Traces 19 and 20 in Session 9: what two documents/concerns do they audit against?"
  options:
    - letter: "A"
      text: "The Standards axis (GLOSSARY, ADRs) and the Spec axis (spec.md compliance)."
      correct: true
      explain: "Correct! Running Standards and Spec reviews in parallel subagents catches both architectural smells and missed requirements."
    - letter: "B"
      text: "The Frontend axis (CSS colors) and the CloudBilling axis (monthly invoices)."
      correct: false
      explain: "`/code-review` specifically splits review into Standards (coding standards/glossary/ADRs) and Spec (originating spec/ticket requirements)."
    - letter: "C"
      text: "The Python axis (pip packages) and the Node axis (Express middleware)."
      correct: false
      explain: "The two axes are language-agnostic engineering dimensions: Standards and Spec."
    - letter: "D"
      text: "The Marketing axis (landing copy) and the Legal axis (patent filings)."
      correct: false
      explain: "Neither marketing copy nor patent filings are part of `/code-review`."

- question: "Why did bare `<tr hx-swap-oob=\"true\">` fragments fail to update the catalog table when returned alongside `<aside>`?"
  wrong_hint: "Remember how the browser's HTML5 fragment parser behaves when a response starts with `<aside>`."
  options:
    - letter: "A"
      text: "Because SQLite locked the database file whenever an aside element was rendered."
      correct: false
      explain: "The database query succeeded; the issue occurred entirely in client-side HTML5 fragment parsing."
    - letter: "B"
      text: "Because the browser's HTML5 parser strips bare tr tags in body mode."
      correct: true
      explain: "Correct! Wrapping the out-of-band `<tr>` in `<template><tr ...>...</tr></template>` prevented the HTML5 parser from stripping it."
    - letter: "C"
      text: "Because Rust's Maud macro refuses to compile tr tags with hyphenated attributes."
      correct: false
      explain: "Maud compiled the `<tr>` HTML string fine; the browser stripped it when parsing the HTTP response."
    - letter: "D"
      text: "Because Chrome blocks all HTTP responses larger than five hundred bytes locally."
      correct: false
      explain: "Chrome has no such limit; live inspection via `chrome-devtools-mcp` exposed the HTML5 fragment parsing rule."

- question: "Why is live browser verification (`chrome-devtools-mcp`) still valuable after 41 backend tests pass?"
  wrong_hint: "Think about what `cargo test` exercises versus what happens inside a real browser."
  options:
    - letter: "A"
      text: "Because cargo test deletes all CSS files before compiling the server binary."
      correct: false
      explain: "`cargo test` never deletes CSS assets; it simply doesn't execute a browser layout engine or client-side JS."
    - letter: "B"
      text: "Because backend tests do not execute client-side HTMX triggers or HTML5 parsing."
      correct: true
      explain: "Correct! Live browser verification caught unscoped `from:input` triggers, `<template>` OOB wrapping, and 340px drawer CSS overflow."
    - letter: "C"
      text: "Because Rust unit tests cannot run SQL queries against in-memory Turso databases."
      correct: false
      explain: "All 41 Rust tests ran real SQL migrations and queries against ephemeral `:memory:` Turso databases."
    - letter: "D"
      text: "Because Chrome DevTools automatically rewrites local Rust code into TypeScript React components."
      correct: false
      explain: "`chrome-devtools-mcp` inspects the live DOM, console, network, and layout so the agent can fix the existing stack."
:::

:::teacher-prompt
**Next up**: You've seen how **Vibe Coding** (Lesson 0001) and **Skill-Driven Engineering** (Lessons 0002–0004) complement each other. Now let's bring in [`google/skills`](https://github.com/google/skills) for our open-ended capstone in [Lesson 0005: Capstone Exercise — Productionizing on Google Cloud with `google/skills`](0005-capstone-exercise-productionizing-with-google-cloud-skills.md)!
:::

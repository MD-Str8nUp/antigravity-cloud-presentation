---
title: "UI Prototyping, Specs & Vertical-Slice Tickets"
page_title: "Lesson 0003 — UI Prototyping, Specs & Vertical-Slice Tickets"
nav_id: "lesson-0003"
badge: "Skilled Run • Sessions 6–8 (02-skilled.cast)"
subtitle: "See how /prototype lets you test-drive three UI layout variants before committing to backend templates—and how /to-spec and /to-tickets turn Wayfinder decisions into parallelizable vertical slices."
card:
  title: "**UI Prototyping, Specs & Vertical-Slice Tickets**"
  summary: "Walk through Sessions 6–8 of Run 2 (02-skilled.cast): exploring Variants A/B/C with /prototype, synthesizing spec.md (/to-spec), and slicing 8 vertical tickets (/to-tickets)."
---

## 1. Borrowing the Best Part of Vibe Coding: Cheap Visual Prototypes (`/prototype`) {#ui-prototyping}

One fair criticism of traditional "spec-first" engineering is that **humans are terrible at evaluating user interfaces in prose**. If someone asks you in a text chat whether you want a 2-pane table + drawer layout or a 3-pane IDE tree layout, you're just guessing until you see it on screen.

That is why `/wayfinder` includes a dedicated **`prototype`** ticket type (powered by the `/prototype` skill). Instead of choosing a single UI layout unprompted (like the **Baseline Run** did) or debating UI layouts purely in Markdown, **Session 6 (Stops 81–92)** of the **Skilled Run** built a self-contained HTML prototype file—[`.scratch/audio-inventory/prototype-ui.html`](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/experiments/audio_inventory_skilled/.scratch/audio-inventory/prototype-ui.html)—implementing **three complete, interactive layout variants** in a single file:

Table: The three interactive UI variants generated during Session 6 (Ticket 05).
| Variant | Layout Architecture | Best For... | Operator Verdict (Stop 87) |
| :---: | :--- | :--- | :--- |
| **Variant A** (`?variant=a`) | **Full-Width Dense Workbench Table + 340px Right-Hand Inspector Drawer (`#inspector`)** | Scanning wide parametric columns while inspecting/merging multi-location **Stock Lots** in a slide-out drawer. | **Selected (`A`)** |
| **Variant B** (`?variant=b`) | **Three-Pane IDE Layout** (Left location/kit tree + center table + right inspector) | Deep folder navigation, at the cost of horizontal table width on 13" laptop screens. | Rejected (too cramped horizontally) |
| **Variant C** (`?variant=c`) | **Split-Screen Kitting View** (Kit BOM on left + live Catalog search on right + Bench HUD) | Dedicated side-by-side kitting, inspired by the ** Active Bench Checklist** mode. | Merged checklist ideas into Variant A's Builds tab |

At **Stop 85**, the operator prompted:

```text caption="Stop 85 of 02-skilled.cast — Opening the 3-variant prototype in Chrome"
please open the prototype in a browser
```

By clicking through Variants A, B, and C live in Chrome before a single line of Rust or Maud HTML template code was written, the operator got the **visual speed of vibe coding** combined with the **deliberate choice of engineering design**!{{sidenote:Notice how `/prototype` bridges the gap between vibe coding and engineering: you still get a clickable UI in minutes, but it's explicitly framed as a cheap, disposable decision artifact with 3 options to compare rather than the final production codebase.}}

---

## 2. Synthesizing the Canonical Specification (`/to-spec`, Session 7) {#to-spec}

Once Ticket 05 resolved at **Stop 92**, every decision on the Wayfinder map was complete.

In **Session 7 (Stops 93–97)**, the operator cleared the session (`/clear`) and ran:

```text caption="Stop 94 of 02-skilled.cast — Synthesizing the specification"
/to-spec
```

Why not jump straight from 5 separate decision tickets into coding? Because decision tickets record the *journey* (questions asked, options rejected), whereas an implementer needs a **single coherent snapshot of truth**.

`/to-spec` read `GLOSSARY.md`, `ADR 0001`, `map.md`, and all 5 resolved decision records (`decisions/01..05.md`) and synthesized [`.scratch/audio-inventory/spec.md`](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/experiments/audio_inventory_skilled/.scratch/audio-inventory/spec.md) containing:
- **42 numbered user stories** across Component Catalog, Storage Locations, Stock Lots, Kit Builds, Active Bench Mode, CSV Imports, Shopping Lists, and ZIP Backup.
- **Complete SQLite/Turso DDL schema** (`storage_locations`, `components`, `stock_lots`, `kit_builds`, `bom_lines`, `bench_log_entries`, `kit_attachments`).
- **Exact mathematical and state-machine invariants** (including the 5-state **Fulfillment Status** transition table and **Installed Stock Permanence**).

---

## 3. Slicing Vertical Implementation Tickets (`/to-tickets`, Session 8) {#to-tickets}

In **Session 8 (Stops 98–102)**, the operator ran `/clear` and invoked:

```text caption="Stop 99 of 02-skilled.cast — Breaking spec.md into vertical slices"
/to-tickets
```

Instead of slicing horizontally by technical layer (e.g., *"Ticket 1: Write all database tables"*, *"Ticket 2: Write all backend functions"*, *"Ticket 3: Write all HTML pages"*—which leaves nothing testable end-to-end until the very last ticket), `/to-tickets` sliced the specification into **8 vertical slices** with an explicit dependency graph:

```mermaid caption="Figure 3.1: The 8 vertical-slice implementation tickets generated by /to-tickets in Session 8 (Stops 98–102)."
flowchart TD
  T1["Ticket 01: Workspace Foundation,<br/>spec_parser &amp; Storage Location Tree"] --> T2["Ticket 02: Component Catalog,<br/>Parametric Search &amp; Stock Lots"]
  T2 --> T3["Ticket 03: Kit Builds, BOM Matching<br/>&amp; 5-State Stock Fulfillment"]
  T3 --> T4["Ticket 04: BOM Substitutions,<br/>Partial Allocations, Split &amp; Clone"]
  T3 --> T5["Ticket 05: Active Bench Checklist,<br/>Attachments &amp; Bench Logs"]
  T3 --> T6["Ticket 06: Staged CSV &amp;<br/>Clipboard Imports"]
  T3 --> T7["Ticket 07: Multi-Build Shopping List,<br/>Exports &amp; Order Receiving"]
  T5 --> T8["Ticket 08: Unified Data Directory,<br/>ZIP Backup &amp; Restore"]
```

Look closely at what happens right after **Ticket 03** in Figure 3.1:
- **Tickets 01 → 02 → 03** form the sequential foundation spine.
- Once **Ticket 03** finishes, **Tickets 04, 05, 06, and 07 are all unblocked simultaneously!**
- Because `/to-tickets` wired explicit `Blocked by:` dependencies on each ticket, `/implement-spec` in Session 9 was able to launch **4 parallel Implementer subagents in separate git worktrees at the same time**.

---

## 4. Interactive Exercise: Vertical Slices vs. Horizontal Layers {#exercise-0003}

Practice the commands and vertical-slicing principles from Sessions 6–8 of the **Skilled Run**:

:::exercise id="ex-0003" title="Interactive Lab: From Prototype to Vertical Tickets" badge="Skilled Run • /prototype, /to-spec, /to-tickets"
Fill in the missing skill commands and dependency structure used in Sessions 6–8:

```markdown caption="Turning Wayfinder decisions into a dependency-ordered ticket graph"
# 1. When a Wayfinder ticket has Type: prototype, build a multi-variant HTML file via:
# Skill: {{blank:/prototype||prototype|Use the skill that builds concrete UI/logic artifacts to react to|/prototype|12}}

# 2. Once all Wayfinder decision tickets are closed, synthesize them into spec.md:
# Command: {{blank:/to-spec|Use the slash command that compiles decisions into spec.md|/to-spec|10}}

# 3. Slice spec.md into end-to-end testable implementation tickets with Blocked by edges:
# Command: {{blank:/to-tickets|Use the slash command that slices spec.md into vertical tickets|/to-tickets|12}}
```

:::prompts
hint: "Use `/prototype` for UI exploration, `/to-spec` to synthesize `spec.md`, and `/to-tickets` to create vertical implementation tickets."
success: "Nice work! That three-step bridge (`/prototype` → `/to-spec` → `/to-tickets`) turns open-ended design conversations into an unambiguous, parallelizable build graph."
:::
:::

---

## 5. Try It Yourself: Prototype & Slice Your Own Project {#verify-0003}

Continue in the project directory you started in Lesson 0002 (or pick any UI-heavy feature you are designing):

:::verify id="verify-0003" title="Hands-On Planning Gate: Prototype Variants & Vertical Tickets" badge="Bring Your Own Idea • Sessions 6–8"
Try running `/prototype`, `/to-spec`, and `/to-tickets` on your own project idea in `agy`:

```text caption="Commands to run across fresh /clear sessions in your own project"
# 1. Ask agy to prototype 2–3 distinct UI layout variants in a single HTML file:
/prototype Create a self-contained HTML prototype with 3 switchable layout variants (?variant=a|b|c) for my project's main screen.

# 2. After resolving your Wayfinder tickets, clear context and synthesize the spec:
/clear
/to-spec

# 3. In a fresh session, slice the spec into dependency-ordered vertical tickets:
/clear
/to-tickets
```

- [ ] Open the generated HTML prototype in your browser and compare the layout variants side by side before choosing one.
- [ ] Run `/to-spec` and verify that the resulting `spec.md` aligns with your `GLOSSARY.md` terms.
- [ ] Run `/to-tickets` and inspect the `Blocked by:` dependency graph to see which tickets can run in parallel once your foundation slice is complete.
:::

---

## 6. Retrieval Practice {#quiz-0003}

:::quiz id="quiz-0003" title="Retrieval Practice: Prototyping, Specifications & Vertical Slicing" badge="3 Questions • Equal-Length Options"
- question: "Why did Session 6 build an HTML file with three switchable variants (`?variant=a|b|c`) before writing Rust templates?"
  wrong_hint: "Think about how hard it is to judge table density and drawer layouts from text descriptions alone."
  options:
    - letter: "A"
      text: "Because concrete visual options let humans evaluate layout trade-offs cheaply before coding."
      correct: true
      explain: "Correct! A disposable multi-variant HTML prototype gives you the visual speed of vibe coding while letting you compare options deliberately."
    - letter: "B"
      text: "Because the Rust compiler requires three HTML files before compiling Axum routes."
      correct: false
      explain: "`rustc` and `maud` have no requirement for `.scratch/` HTML files; the prototype is purely for human design evaluation."
    - letter: "C"
      text: "Because HTMX cannot swap table rows unless three query parameters are active."
      correct: false
      explain: "The `?variant=a|b|c` switcher was only part of the static prototype harness in `.scratch/audio-inventory/prototype-ui.html`."
    - letter: "D"
      text: "Because Chrome DevTools MCP refuses to open localhost ports without static files."
      correct: false
      explain: "Chrome DevTools MCP works with any URL; the prototype was opened by the human operator during Session 6."

- question: "What is the key difference between a horizontal task and a vertical-slice ticket produced by `/to-tickets`?"
  wrong_hint: "Consider what can be verified end-to-end with a test when a single ticket finishes."
  options:
    - letter: "A"
      text: "A horizontal task deletes old tests whereas a vertical slice skips testing."
      correct: false
      explain: "Every vertical-slice ticket is built test-first using Red-Green-Refactor TDD."
    - letter: "B"
      text: "A vertical slice cuts through schema, domain logic, and UI for verification."
      correct: true
      explain: "Correct! Each vertical slice delivers a complete, testable capability from database to UI so it can be verified immediately."
    - letter: "C"
      text: "A horizontal task runs in Rust whereas a vertical slice uses JavaScript."
      correct: false
      explain: "Slicing strategy is independent of programming language."
    - letter: "D"
      text: "A vertical slice removes all blocking dependencies so every ticket runs first."
      correct: false
      explain: "Vertical slices still respect real foundation dependencies (like Tickets 01 → 02 → 03) before fanning out in parallel."

- question: "How did the dependency graph of the 8 tickets in the Skilled Run enable parallel execution?"
  wrong_hint: "Look at Figure 3.1: what depended on Ticket 03 once the core catalog and BOM tables were in place?"
  options:
    - letter: "A"
      text: "All eight tickets had zero blockers and ran simultaneously from minute one."
      correct: false
      explain: "Tickets 01 → 02 → 03 had to run sequentially first to establish the schema, catalog, and BOM foundation."
    - letter: "B"
      text: "Every single ticket blocked the next ticket in a strict single-file chain."
      correct: false
      explain: "Only 01 → 02 → 03 and 05 → 08 were sequential; four middle tickets fanned out in parallel."
    - letter: "C"
      text: "Once Ticket 03 merged, Tickets 04, 05, 06, and 07 unblocked simultaneously."
      correct: true
      explain: "Correct! Because Tickets 04–07 all depended on Ticket 03 and not on each other, `/implement-spec` executed all 4 in parallel git worktrees."
    - letter: "D"
      text: "Ticket 08 ran first and generated the database migrations for Ticket 01."
      correct: false
      explain: "Ticket 08 (`.zip` backup and restore) was the final wave, blocked by Ticket 05 (attachments)."
:::

:::teacher-prompt
**Next up**: What happens when you type `/implement-spec` and let the orchestrator build all 8 tickets autonomously across isolated git worktrees, run a parallel 2-axis code review, and test the live HTMX app in Chrome? Head to [Lesson 0004: Worktree TDD, Two-Axis Code Review & Live Browser Verification](0004-worktree-tdd-code-review-and-browser-verification.md)!
:::

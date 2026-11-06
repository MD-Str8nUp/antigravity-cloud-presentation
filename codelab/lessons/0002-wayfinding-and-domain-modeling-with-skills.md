---
title: "Wayfinding & Domain Modeling: Durable Intent Across /clear"
page_title: "Lesson 0002 — Wayfinding & Domain Modeling with Agent Skills"
nav_id: "lesson-0002"
badge: "Skilled Run • Sessions 1–5 (02-skilled.cast)"
subtitle: "Learn how installing mattpocock/skills (/wayfinder, /grilling, /domain-modeling) uncovers real-world domain requirements you didn't know to ask for—and writes them to GLOSSARY.md and ADRs so every session survives /clear."
card:
  title: "**Wayfinding & Domain Modeling: Durable Intent Across `/clear`**"
  summary: "Walk through Sessions 1–5 of Run 2 (02-skilled.cast): how /wayfinder and /domain-modeling uncovered hierarchical Storage Locations, multi-location Stock Lots, and 5-state BOM fulfillment."
---

## 1. Crossing the Threshold from Exploration to Engineering {#exploration-to-engineering}

Once a rapid vibe-coding spike convinces you that a project is worth building for real, the rules of the game change. You no longer just want *any* code that renders on `localhost:3000`—you want:

1. **A Domain Model That Matches Reality**: Probing edge cases *before* they are baked into database tables.
2. **Institutional Memory That Survives `/clear`**: Writing domain terms (`GLOSSARY.md`) and architectural trade-offs (`docs/adr/`) into the repository so future sessions (and human teammates) never have to guess.

In **Run 2** of our experiment (the **Skilled Run**, recorded in [`recordings/final/02-skilled.cast`](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/recordings/final/02-skilled.cast)), we started in a fresh empty directory, installed [`mattpocock/skills`](https://github.com/mattpocock/skills), and gave `agy` the **exact same opening prompt**—prefixed with `/wayfinder`:

```bash caption="Stops 1–5 of 02-skilled.cast — Installing mattpocock/skills and invoking /wayfinder"
npx skills@latest add mattpocock/skills
agy

# Inside agy (Stop 5):
/wayfinder I'd like to build a web application to track my audio electronics components inventory and DIY kit builds.
```

:::callout
**Primary Source**: Inspect the [Skilled Run Codelab Script (`docs/codelab-run-2-skilled.md`)](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/docs/codelab-run-2-skilled.md), the generated [`GLOSSARY.md`](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/experiments/audio_inventory_skilled/GLOSSARY.md), [`docs/adr/0001-rust-htmx-turso-stack.md`](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/experiments/audio_inventory_skilled/docs/adr/0001-rust-htmx-turso-stack.md), and [`.scratch/audio-inventory/map.md`](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/experiments/audio_inventory_skilled/.scratch/audio-inventory/map.md).
:::

---

## 2. How `/wayfinder` + `/grilling` + `/domain-modeling` Work Together {#wayfinder-mechanics}

When you invoke `/wayfinder`, the agent explicitly follows *"Plan, don't do"*: it refuses to write application code until the route to the destination is clear.{{sidenote:Notice the division of labor: `/wayfinder` manages the shared map and decision tickets, `/grilling` interviews you to stress-test edge cases, and `/domain-modeling` challenges fuzzy words and updates `GLOSSARY.md` and `docs/adr/` inline.}}

```mermaid caption="Figure 2.1: How Sessions 1–5 of the Skilled Run resolved the Wayfinder map across /clear boundaries."
flowchart TD
  S1["Session 1 (Stops 5–28):<br/>Chart the Map (11 Questions)<br/>Writes GLOSSARY.md, ADR 0001, map.md"] --> C1["/clear"]
  C1 --> S2["Session 2 (Stops 29–45):<br/>Ticket 01: Taxonomy &amp; Storage<br/>Uncovers Location Trees &amp; Stock Lots!"]
  S2 --> C2["/clear"]
  C2 --> S3["Session 3 (Stops 46–58):<br/>Ticket 02: BOM &amp; 5-State Fulfillment<br/>Defines Installed Permanence &amp; Split Line"]
  S3 --> C3["/clear"]
  C3 --> S4["Session 4 (Stops 59–69):<br/>Ticket 03: CSV &amp; Multi-Build Shortfall<br/>Prevents Run 1's Double-Counting Bug!"]
  S4 --> C4["/clear"]
  C4 --> S5["Session 5 (Stops 70–80):<br/>Ticket 04: Rust + HTMX + Turso Modules<br/>Verifies turso crate &amp; 5 Deep Modules"]
```

### Session 1 (Stops 5–28): Charting the Map

In Session 1 (`10m 23s`), `agy` asked **11 breadth-first map-charting questions** (one at a time) to pin down the destination, scope boundaries (single-user hobby builder, manual + CSV import rather than live Mouser API scraping), and standing tech preferences (Rust backend, server-rendered HTML with HTMX, local Turso database).

Before Session 1 ended at **Stop 28**, `agy` wrote **8 markdown files to disk**:
- `GLOSSARY.md` (initial core vocabulary with explicit `_Avoid_` synonyms)
- `docs/adr/0001-rust-htmx-turso-stack.md`
- `.scratch/audio-inventory/map.md`
- 5 open decision tickets (`01` through `05`) in `.scratch/audio-inventory/issues/`

---

## 3. How `/grilling` Uncovered Requirements That Vibe Coding Missed {#grilling-breakthroughs}

The most striking difference between the **Baseline Run** and the **Skilled Run** happened in **Session 2 (Stops 31–42)** while resolving **Ticket 01 (`01-component-category-taxonomy-and-parametric-schema.md`)**.

In the **Baseline Run**, `agy` assumed every component has a single flat `location` text field and a single `qty_on_hand` integer. Look at what happened when `/grilling` probed how the operator *actually* stores electronics parts in real life:

Table: Real-world domain breakthroughs uncovered during Session 2 (Stops 31–42) and Session 3 (Stops 48–57).
| Cast Stops | What `/grilling` Asked | Operator's Real-World Answer | Resulting Domain Model Upgrade |
| :---: | :--- | :--- | :--- |
| **31 → 32** | Are these 8 audio categories complete? | *"I'll need a ninth category for manufactured sub assemblies like MCU boards and the like."* | Added **9. Modules & Sub-Assemblies** (Daisy Seed, Teensy, Belton reverb bricks). |
| **35 → 36** | How should electrical shorthand (`4k7`, `100kA`) be stored? | *"Store the original spec string alongside a canonicalized value. A lot of the values include variance and pot curve that are important."* | Dual `raw_spec_text` (preserved verbatim) + `canonical_value_display` + normalized SI floats (`resistance_ohms`, `capacitance_pf`). |
| **37 → 38** | Are storage locations flat labels or nested containers? | *"I have crates with bags full of bagged components. Given I need to move the bags around it is worth having a hierarchical concept."* | Self-referential **Storage Location** tree (`parent_id`) where moving a bag updates the breadcrumb path for all nested items! |
| **39 → 40** | Does a component live in one location or multiple? | *"I have duplicate bags with the same components in multiple locations, because it was easier to order more than find the ones I have."* | First-class **Stock Lots** (`stock_lots` table) with per-lot locations and a **Merge Lots** workflow when consolidating bags! |
| **50 → 51** | How do parts move from drawers to a finished PCB? | Confirmed 5 distinct physical states during a build. | 5-state **Fulfillment Status** (`Unallocated`, `Reserved`, `Picked`, `Installed`, `Included in Kit`) + **Split BOM Line**. |

:::callout-success
**Why This Matters for Your Own Projects**:
When you vibe-code, the agent optimizes for *getting to code fast*, so it picks the simplest possible schema (flat strings, single quantities). When you run `/wayfinder`, the agent's job is to **stress-test your domain model through concrete scenarios**—helping you discover requirements you wouldn't have thought to put in an initial prompt!
:::

---

## 4. Preventing Bugs Before Writing a Single Line of Code (Session 4) {#preventing-bugs}

Remember **Bug #1** from the **Baseline Run** (Lesson 0001), where `getAllProjects()` and `getShoppingList()` checked `qty_available >= needed` per row and double-counted shared stock across BOM lines?

Look at **Session 4 (Stops 65–66)** of the **Skilled Run**, where `/wayfinder` resolved **Ticket 03 (`03-csv-import-and-shopping-list-export-workflows.md`)** *before any Rust code existed*. `agy` explicitly formulated and confirmed the **Net Shortfall** aggregation invariant:

$$\text{Net Shortfall}(c) = \max\left(0, \sum_{b \in \text{Selected Builds}} \text{unallocated\_needed}(b, c) - \text{Available Stock}(c)\right)$$

And updated `GLOSSARY.md` immediately at **Stop 69**:

```markdown caption="experiments/audio_inventory_skilled/GLOSSARY.md — Capturing Available Stock & Net Shortfall in durable prose"
**Available Stock**:
The quantity of a Component currently free to be allocated to new Kit Builds, calculated as total on-hand stock across all Stock Lots minus the quantity currently Reserved for active builds.
_Avoid_: Free stock, unallocated inventory

**Net Shortfall**:
The quantity of a Component that must be ordered across one or more selected Kit Builds, calculated by aggregating total unallocated BOM demand across all selected builds before subtracting Available Stock.
_Avoid_: Line shortage, raw deficit
```

---

## 5. Interactive Exercise: Sharp Domain Modeling in `GLOSSARY.md` {#exercise-0002}

Practice the `/domain-modeling` and `/wayfinder` patterns used in Sessions 1–4 of the **Skilled Run**:

:::exercise id="ex-0002" title="Interactive Lab: Wayfinder & Glossary Invariants" badge="Skilled Run • Domain Modeling"
Fill in the missing command, glossary section marker, and SQL/domain invariant from the **Skilled Run**:

```markdown caption="Charting a Wayfinder map and locking domain terms in GLOSSARY.md"
# 1. Start a new Wayfinder map in agy after installing mattpocock/skills:
# Prompt prefix: {{blank:/wayfinder|Use the slash command that charts a decision map|/wayfinder|12}} I'd like to build...

# 2. In GLOSSARY.md, list imprecise synonyms to reject under each definition:
**Stock Lot**:
A specific physical stash or bag of a Component stored at a particular Storage Location.
_{{blank:Avoid|The GLOSSARY-FORMAT.md label listing rejected synonyms|Avoid|8}}_: Duplicate component, bin count

# 3. Multi-Build Net Shortfall sums demand across all selected builds BEFORE subtracting:
# Net Shortfall = max(0, sum(unallocated_needed) - {{blank:Available Stock|Subtract the canonical glossary term for free stock|Available Stock|17}})
```

:::prompts
hint: "Use `/wayfinder` to start the map, `_Avoid_:` for rejected synonyms in `GLOSSARY.md`, and `Available Stock` in the shortfall formula."
success: "Exactly right! Pinning down `Stock Lot`, `Available Stock`, and `Net Shortfall` in `GLOSSARY.md` meant every subsequent `/clear` session spoke the exact same language."
:::
:::

---

## 6. Try It Yourself: Run `/wayfinder` on Your Own Project Idea {#verify-0002}

Take the same project idea you explored in Lesson 0001 (or a project you genuinely want to build and maintain) and compare how `/wayfinder` changes the conversation:

:::verify id="verify-0002" title="Hands-On Discovery Gate: Chart a Wayfinder Map for Your Own Idea" badge="Bring Your Own Idea • mattpocock/skills"
Create a fresh directory, install `mattpocock/skills`, and run `/wayfinder` on your own idea:

```bash caption="Installing mattpocock/skills and launching /wayfinder on your own project"
mkdir -p ~/Demos/my-skilled-project && cd ~/Demos/my-skilled-project
npx skills@latest add mattpocock/skills
agy

# Inside agy:
/wayfinder <describe your own project idea in one sentence>
```

- [ ] Notice how `/wayfinder` interviews you about your **Destination**, scope boundaries, and domain terms before writing application code.
- [ ] Check that `GLOSSARY.md` and your Wayfinder map (`.scratch/.../map.md` or GitHub issue) are written to disk at the end of Session 1.
- [ ] Run `/clear`, then type `/wayfinder` with no arguments—and watch the fresh session orient itself from your repo files and claim the first decision ticket!
:::

---

## 7. Retrieval Practice {#quiz-0002}

:::quiz id="quiz-0002" title="Retrieval Practice: Wayfinding & Domain Modeling" badge="3 Questions • Equal-Length Options"
- question: "What is the primary rule of `/wayfinder` during map charting and decision ticket sessions?"
  wrong_hint: "Remember the heading 'Plan, don't do' in `wayfinder/SKILL.md`."
  options:
    - letter: "A"
      text: "Write all database migrations and HTTP routes before asking the user questions."
      correct: false
      explain: "Writing code before resolving decisions is the opposite of wayfinding."
    - letter: "B"
      text: "Resolve one decision ticket per session and record durable decisions before coding."
      correct: true
      explain: "Correct! Wayfinder resolves open architectural and domain decisions one ticket at a time until the route to the destination is clear."
    - letter: "C"
      text: "Keep all domain definitions exclusively inside conversational memory without writing markdown files."
      correct: false
      explain: "Wayfinder and domain-modeling write `GLOSSARY.md`, ADRs, and decision records to disk so they survive `/clear`."
    - letter: "D"
      text: "Never allow the operator to clear the terminal session until deployment finishes."
      correct: false
      explain: "Wayfinder is explicitly designed for `/clear` between every ticket so each session starts with a clean context window."

- question: "How did the Skilled Run discover hierarchical Storage Locations and multi-location Stock Lots?"
  wrong_hint: "Look back at Stops 37–40 in Session 2 of `02-skilled.cast`."
  options:
    - letter: "A"
      text: "The operator pasted a complete SQL database schema inside the opening prompt."
      correct: false
      explain: "The opening prompt in Run 2 was the exact same one-sentence prompt used in Run 1."
    - letter: "B"
      text: "The Rust compiler rejected flat location strings during the initial cargo build."
      correct: false
      explain: "No Rust code existed yet during Session 2; the discovery happened purely through domain grilling."
    - letter: "C"
      text: "Grilling probed how the builder physically stores parts, surfacing nested bags naturally."
      correct: true
      explain: "Correct! By asking concrete scenario questions during Ticket 01, `/grilling` and `/domain-modeling` uncovered nested bags and duplicate stashes."
    - letter: "D"
      text: "Chrome DevTools MCP automatically scraped Mouser and DigiKey warehouse bin hierarchy tables."
      correct: false
      explain: "Live supplier APIs were explicitly ruled out of scope during Session 1 map charting."

- question: "What should `GLOSSARY.md` contain according to the `domain-modeling` skill?"
  wrong_hint: "Check the rules in `domain-modeling/SKILL.md`: is `GLOSSARY.md` for domain concepts or implementation code?"
  options:
    - letter: "A"
      text: "Tight, unambiguous definitions of domain concepts and rejected synonyms, without implementation details."
      correct: true
      explain: "Correct! `GLOSSARY.md` defines ubiquitous domain language and `_Avoid_` aliases, staying free of implementation details."
    - letter: "B"
      text: "Complete SQLite CREATE TABLE statements, Express route handlers, and CSS flexbox rules."
      correct: false
      explain: "`GLOSSARY.md` must be devoid of implementation details; technical trade-offs belong in `docs/adr/` and `spec.md`."
    - letter: "C"
      text: "A chronological transcript of every tool call and compiler error from sessions."
      correct: false
      explain: "Tool logs live in Session Traces (`artifacts/traces/`), never in the domain glossary."
    - letter: "D"
      text: "Temporary todo items and unresolved questions that get deleted after every commit."
      correct: false
      explain: "Open questions live on the Wayfinder map frontier; `GLOSSARY.md` is a durable, evolving record of domain language."
:::

:::teacher-prompt
**Next up**: Once your domain terms and architectural decisions are locked, how do you explore UI layouts visually and slice the build into testable vertical tickets? Continue to [Lesson 0003: UI Prototyping, Specs & Vertical-Slice Tickets](0003-prototyping-specs-and-vertical-slice-tickets.md)!
:::

---
title: "Glossary of Experiment, Workflow & Domain Terms"
page_title: "Glossary — Vibe Coding vs. Agent Skills Codelab"
nav_id: "glossary"
nav_label: "Glossary"
badge: "Mission: Vibe Coding vs. Agent Skills"
subtitle: "Canonical definitions for the three-run experiment, Agent Skills workflows, and the audio electronics case study. Designed for quick lookup and clean printing."
card:
  title: "**Glossary of Experiment, Workflow & Domain Terms**"
  summary: "Canonical definitions for Baseline Run, Skilled Run, Production Run, Wayfinder artifacts, and the audio electronics case study."
---

## 1. Experiment Runs & Core Modes

**Baseline Run**:
Run 1 of the recorded experiment (`audio_inventory_baseline`), where `agy` builds a local web application from a single-sentence prompt guided only by the user's global clarifying-question rule (`~/.gemini/config/AGENTS.md`) and ambient global plugins, with zero repository skills installed.
_Avoid_: Unskilled run, vanilla run

**Skilled Run**:
Run 2 of the recorded experiment (`audio_inventory_skilled`), where `agy` uses `mattpocock/skills` (`/wayfinder`, `/grilling`, `/domain-modeling`, `/prototype`, `/to-spec`, `/to-tickets`, `/implement-spec`, `/code-review`) across 10 `/clear`-separated sessions to model the domain, commit durable documentation, and build a tested application.
_Avoid_: Spec run, planning run

**Production Run**:
Stage 3 of the experiment, where a local application is adapted for low-cost serverless cloud deployment (**Cloud Run** and **Cloud Firestore** behind a repository seam) using [`google/skills`](https://github.com/google/skills) guidance, presented in this Codelab as an architectural walkthrough and open-ended reader capstone exercise.
_Avoid_: Implementation run, comprehension run

**Vibe Coding**:
Working with an AI coding agent in an exploratory, conversational loop where the agent moves directly from brief Q&A to generating working application code without first committing formal specifications, glossaries, or test suites. Ideal for fast discovery of requirements and UI possibilities.
_Avoid_: Sloppy coding, unprompted generation

**Skill-Driven Engineering**:
Shaping an AI coding agent with explicit, on-demand workflow and domain skills (`SKILL.md`) so that discovery, domain modeling (`GLOSSARY.md`, ADRs), UI prototyping, specification, vertical-slice TDD, and code review happen as distinct, verifiable stages recorded in the repository.
_Avoid_: Waterfall prompting, heavy process

## 2. Recording & Inspection Artifacts

**Terminal Cast**:
An `asciinema` v3 recording of a terminal session (`01-baseline.cast`, `02-skilled.cast`), sanitized and annotated with `"m"` chapter stop-point markers for interactive playback.
_Avoid_: Video, screen recording

**Session Trace**:
The exported Markdown conversation log (`artifacts/traces/*`) from Antigravity's session storage capturing every user prompt, internal reasoning step, and tool call for a single session.
_Avoid_: Chat history, transcript dump

**Canonical Q&A**:
The consistent set of operator domain preferences used when answering `agy`'s clarifying questions across runs so that architectural differences stem from the installed skills rather than divergent requirements.
_Avoid_: Cheat sheet, prompt script

**Codelab**:
An interactive lesson in `./codelab/` that walks learners through the recorded experiment runs using the captured **Terminal Casts**, **Session Traces**, and resulting codebases while inviting learners to test the workflows on their own project ideas.
_Avoid_: Tutorial page, handout, lab doc

## 3. Agent Customization & Wayfinder Vocabulary

**Global Rule**:
An always-on instruction in `$HOME/.gemini/config/AGENTS.md` (or repo-root `AGENTS.md`) loaded into every session automatically—such as *"When something is unclear, please ask clarifying questions, one at a time."*
_Avoid_: System prompt hack

**Agent Skill**:
A modular folder (`.agents/skills/<name>/SKILL.md`) containing YAML frontmatter (`name`, `description`) and specialized markdown instructions, reference docs, or scripts that the agent loads on demand when a task or `/slash-command` matches.
_Avoid_: Plugin macro, custom GPT

**Wayfinder Map**:
A persistent planning artifact (`map.md` or a `wayfinder:map` issue) that states the **Destination**, **Notes**, **Decisions so far**, **Not yet specified** (fog of war), and **Out of scope** boundaries across multiple agent sessions.
_Avoid_: Backlog dump, todo list

**Decision Ticket**:
A child issue of a **Wayfinder Map** (`grilling`, `prototype`, `research`, or `task`) that resolves a single architectural or domain question before implementation begins.
_Avoid_: Sprint task, build ticket

**Vertical-Slice Ticket**:
An implementation ticket produced by `/to-tickets` that cuts through schema, domain logic, and UI for one user-visible capability so it can be built and tested end-to-end via Red-Green-Refactor TDD.
_Avoid_: Horizontal layer task

## 4. Case Study Domain (Audio Electronics Workbench)

**Component**:
An individual electronic part in the catalog (such as a `TL072` op-amp, `4k7` resistor, or `100nF` capacitor), characterized by category, package mounting style (`SMT` or `THT`), raw and normalized electrical values, and stock across one or more **Stock Lots**.
_Avoid_: Item, product

**Storage Location**:
A physical place where parts live on the bench. Modeled as a flat text label in the **Baseline Run**, and as a self-referential parent/child hierarchy (`Binder` → `Bag` → `Sub-Bag`) in the **Skilled Run**.
_Avoid_: Warehouse bin

**Stock Lot**:
A specific stash or bag of a **Component** stored at a specific **Storage Location** with its own quantity on hand, supplier metadata, and merge history. Uncovered during `/wayfinder` Ticket 01 in the **Skilled Run**.
_Avoid_: Duplicate component row

**Kit Build**:
A DIY audio electronics PCB project (such as a Eurorack filter module or guitar overdrive pedal) tracked across lifecycle stages alongside its **BOM Lines** and timestamped **Bench Log Entries**.
_Avoid_: Bundle, assembly

**Fulfillment Status**:
The 5-state lifecycle of a **BOM Line** in the **Skilled Run** (`Unallocated`, `Reserved`, `Picked`, `Installed`, `Included in Kit`), where `Installed` parts are permanently soldered and never refunded to stock if a line is later edited (**Installed Stock Permanence**).
_Avoid_: Boolean checked flag

**Net Shortfall**:
The quantity of a **Component** that must be ordered across multiple selected **Kit Builds**, calculated by summing unallocated demand across all selected builds *before* subtracting **Available Stock**: $\max(0, \sum \text{unallocated\_needed} - \text{Available Stock})$.
_Avoid_: Per-line shortage

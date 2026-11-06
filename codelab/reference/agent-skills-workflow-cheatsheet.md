---
title: "Vibe Coding vs. Agent Skills Decision & Workflow Cheat Sheet"
page_title: "Cheat Sheet — Vibe Coding vs. Agent Skills Workflows"
nav_id: "workflow-cheatsheet"
nav_label: "Workflow Cheat Sheet"
badge: "Reference • Decision Matrix & Commands"
subtitle: "When to vibe-code for fast discovery, when to switch to /wayfinder and mattpocock/skills for maintainable engineering, and the exact session loop for each."
card:
  title: "**Vibe Coding vs. Agent Skills Decision & Workflow Cheat Sheet**"
  summary: "Side-by-side decision matrix, session loops, and slash-command reference for Baseline exploration vs. Skilled engineering."
---

## 1. Decision Matrix: Which Mode Should You Use Right Now?

Neither mode is "always right" or "always wrong." Match your workflow to the **uncertainty and lifespan** of what you are building:

Table: Choosing between Baseline Vibe-Coding and Skill-Driven Engineering.
| Question | Choose Vibe Coding (Baseline Mode) | Choose Skill-Driven Engineering (`/wayfinder`) |
| :--- | :--- | :--- |
| **What is your goal today?** | Explore a fuzzy idea, see a live UI in 15 minutes, or test whether a concept is worth pursuing. | Build a well-engineered system with documentation and tests that you (or a team) will maintain over time. |
| **How well do you know your requirements?** | Low — "I'll know what I want once I see something working on screen." | Medium to High — or you have domain nuances (hierarchies, state machines, accounting rules) that must not break. |
| **What happens after `/clear`?** | Acceptable if the prototype is disposable or you are ready to manually prompt `/clear` → `Please review the codebase`. | Critical — every new session must inherit `GLOSSARY.md`, `docs/adr/`, `spec.md`, and automated tests without re-explaining the domain. |
| **Expected Lifespan** | Hours to a few days (throwaway spike, hackathon demo, personal one-off utility). | Weeks to years (ongoing maintenance, team handoff, or cloud production deployment). |

:::callout-success
**The Hybrid Sweet Spot ("Vibe First, Wayfind Second")**:
Many experienced builders use **both** on the same project! Spend 15 minutes vibe-coding a quick throwaway spike to discover what domain edge cases and UI interactions actually matter to you—then start a clean repository with `mattpocock/skills` and run `/wayfinder` armed with those insights.
:::

---

## 2. Mode A: The Rapid Exploration Loop (Baseline Vibe-Coding)

Even when vibe-coding without repository skills, two small practices dramatically improve your results:

1. **Keep the One-Question-at-a-Time Global Rule in `~/.gemini/config/AGENTS.md`**:
   ```markdown caption="~/.gemini/config/AGENTS.md"
   You are a helpful assistant.

   When something is unclear, please ask clarifying questions, one at a time.
   ```
   Without this two-line file, an agent will guess every requirement silently. With it, `agy` conducts a focused 3–5 question interview before writing code.
2. **Always Run a Fresh-Context Review Pass After `/clear`**:
   Never trust a 2,000-line one-shot build unreviewed. Once the prototype runs:
   ```text caption="Fresh-context review loop in agy"
   /clear
   Please review the codebase
   please fix
   please test this site in a web browser
   ```

---

## 3. Mode B: The Skill-Driven Engineering Loop (`mattpocock/skills`)

When you are ready to build for ongoing maintenance, install `mattpocock/skills` and use `/clear` between every stage so each session stays well within its context budget and relies on **durable repo files**:

```mermaid caption="Figure R1.1: The multi-session Skill-Driven Engineering lifecycle with /clear between stages."
flowchart LR
  S0["npx skills@latest add<br/>mattpocock/skills"] --> S1["Session 1:<br/>/wayfinder &lt;idea&gt;<br/>(GLOSSARY + ADR + map)"]
  S1 --> S2["Sessions 2..N:<br/>/wayfinder<br/>(Grilling &amp; /prototype)"]
  S2 --> S3["Session N+1:<br/>/to-spec<br/>(Synthesize spec.md)"]
  S3 --> S4["Session N+2:<br/>/to-tickets<br/>(Vertical slices)"]
  S4 --> S5["Session N+3:<br/>/implement-spec<br/>(Worktree TDD + Review)"]
  S5 --> S6["Session N+4:<br/>Browser Verification<br/>(Chrome DevTools MCP)"]
```

Table: Reference of core `mattpocock/skills` commands and the durable artifacts they produce.
| Command / Skill | Role in the Lifecycle | Durable Repo Artifact Written |
| :--- | :--- | :--- |
| **`/wayfinder <idea>`** | Interviews you to lock the **Destination**, standing constraints, and initial decision frontier. | `GLOSSARY.md`, `docs/adr/0001-*.md`, `.scratch/<app>/map.md`, `.scratch/<app>/issues/01..N.md` |
| **`/wayfinder`** *(subsequent sessions)* | Claims the next unblocked decision ticket (`grilling`, `prototype`, `research`, or `task`) and resolves it. | `.scratch/<app>/decisions/01..N.md`, updated `GLOSSARY.md` & `map.md` |
| **`/prototype`** *(invoked on prototype tickets)* | Builds a cheap, self-contained HTML prototype with multiple layout variants (`?variant=a\|b\|c`) to react to in a browser. | `.scratch/<app>/prototype-ui.html` |
| **`/to-spec`** | Synthesizes `GLOSSARY.md`, ADRs, and all resolved Wayfinder decisions into a single canonical specification. | `.scratch/<app>/spec.md` |
| **`/to-tickets`** | Breaks `spec.md` into dependency-ordered vertical-slice implementation tickets with explicit acceptance criteria. | `.scratch/<app>/issues/01..M-*.md` |
| **`/implement-spec`** | Executes unblocked tickets in isolated git worktrees using Red-Green-Refactor `/tdd` (running independent waves in parallel), then runs 2-axis `/code-review`. | Tested application source code, migrations, and resolved ticket records |
| **`/code-review`** | Spawns two parallel review subagents—**Standards Axis** (`GLOSSARY.md`, ADRs, deep modules) and **Spec Axis** (`spec.md` compliance)—and fixes discrepancies. | Verified, spec-compliant codebase |

---

## 4. Empirical Comparison from the Recorded Audio Inventory Experiment

Table: Head-to-head metrics from `01-baseline.cast` vs. `02-skilled.cast`.
| Metric | Run 1 — Baseline Run (`01-baseline.cast`) | Run 2 — Skilled Run (`02-skilled.cast`) |
| :--- | :--- | :--- |
| **Opening Prompt** | `"I'd like to build a web application to track my audio electronics components inventory and DIY kit builds."` | `"/wayfinder I'd like to build a web application to track my audio electronics components inventory and DIY kit builds."` |
| **Time to First Working Build** | ~`16m` (1 session: 4 questions + 14m one-shot build) | ~`117m` across 9 `/clear` sessions (6 wayfinding + spec + tickets + parallel TDD build) |
| **Clarifying Questions Asked** | `4` multiple-choice questions | `33` questions (`11` map-charting + `22` across 5 decision tickets) |
| **Durable Docs Written** | `0` (`GLOSSARY.md`, ADRs, and specs absent) | `GLOSSARY.md` (`11` terms), `ADR 0001`, `map.md`, `5` decision records, `prototype-ui.html`, `spec.md`, `8` tickets |
| **Automated Test Count** | `0` test files | `41` unit & integration tests against ephemeral `:memory:` Turso |
| **Domain Bugs Found After Build** | `8` bugs in `db.js`/`server.js`/`app.js` (multi-line BOM stock over-counting, broken allocation invariants, `4k7Ω` regex bug, N+1 queries) | `5` spec/standards nuances caught & fixed automatically by `/implement-spec`'s built-in 2-axis `/code-review` + `4` live HTMX browser fixes |

---
title: "Baseline Vibe-Coding: Rapid Exploration & Its Trade-Offs"
page_title: "Lesson 0001 — Baseline Vibe-Coding for Rapid Exploration"
nav_id: "lesson-0001"
badge: "Baseline Run • 01-baseline.cast"
subtitle: "See how a two-line global clarifying-question rule turns a one-sentence prompt into a working prototype in 16 minutes—and why clearing the session exposes the limits of vibe coding for ongoing maintenance."
card:
  title: "**Baseline Vibe-Coding: Rapid Exploration & Its Trade-Offs**"
  summary: "Walk through Run 1 (01-baseline.cast): how one global rule enabled a 16-minute prototype, what went right for fast exploration, and the 8 bugs exposed after /clear."
---

## 1. Why Vibe Coding Exists (and When It Shines) {#why-vibe-coding}

When you have a brand-new software idea, your biggest risk usually isn't code quality—it's **building the wrong thing**, or spending three days setting up architecture for an idea you abandon on day four.{{sidenote:In early-stage discovery, seeing a concrete working UI in 15 minutes often teaches you more about your own requirements than hours of abstract whiteboard speculation.}}

That is where **Vibe Coding** shines:
- You start with a loose idea in an empty folder.
- You let the agent propose a sensible local tech stack and generate a working prototype immediately.
- Within 15–20 minutes, you can click through real tables, forms, and seed data in your browser and react to what feels right or missing.

In **Run 1** of our experiment (the **Baseline Run**, recorded in [`recordings/final/01-baseline.cast`](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/recordings/final/01-baseline.cast)), we started in an empty directory (`~/Demos/audio-inventory-experiment/audio_inventory_baseline`) with **zero repository skills installed**.

Yet `agy` didn't blindly hallucinate a random app on turn one. Why? Because of a **two-line Global Rule** in `$HOME/.gemini/config/AGENTS.md`:

```markdown caption="~/.gemini/config/AGENTS.md — The two-line Global Rule active during the Baseline Run"
You are a helpful assistant.

When something is unclear, please ask clarifying questions, one at a time.
```

:::callout
**Primary Source**: Inspect the complete [Baseline Run Codelab Script (`docs/codelab-run-1-baseline.md`)](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/docs/codelab-run-1-baseline.md) and the exported [Session 1 Trace (`01-4c874de2`)](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/artifacts/traces/audio_inventory_baseline/01-4c874de2-i-d-like-to-build-a-web.md) and [Session 2 Trace (`02-9a12e573`)](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/artifacts/traces/audio_inventory_baseline/02-9a12e573-please-review-the-codebase.md).
:::

---

## 2. Inside the Baseline Run Terminal Cast (`01-baseline.cast`) {#baseline-cast}

Let's walk through what happened across the **17 chapter stop points** of `01-baseline.cast` (`30m 08s` at 1x, warped to `3m 01s`):

Table: Stop-point timeline of the Baseline Run (`01-baseline.cast`).
| Stops | 1x Time | Phase | What Happened in the Terminal |
| :---: | :---: | :--- | :--- |
| **1–3** | `0:01`–`0:06` | **Launch & Prompt** | Operator runs `agy` in an empty folder and submits: *"I'd like to build a web application to track my audio electronics components inventory and DIY kit builds."* |
| **4–9** | `0:27`–`1:47` | **4 Clarifying Questions** | Guided by the global rule, `agy` calls `ask_question` 4 times: **Q1** Stack (`Node.js + SQLite`), **Q2** Features + operator write-in (`Allocate parts to a project`), **Q3** Part allocation (`On Hand` vs. `Allocated` vs. `Available`), and **Q4** Audio focus (`Eurorack` + `Guitar Pedals`). |
| **10–11** | `1:47`–`16:05` | **One-Shot Build** | In a single 14-minute turn, `agy` checks `modern-web-guidance`, writes `package.json`, `db.js`, `server.js`, and `public/{index.html,styles.css,app.js}` (~2,500 lines), seeds 26 audio **Components** and 3 **Kits**, and runs a quick `node -e` smoke check. |
| **12** | `16:09` | **Context Reset (`/clear`)** | Operator runs `/clear` to test what survives when conversational memory is wiped. |
| **13–15** | `16:10`–`25:01` | **Review & Fix Pass** | Operator prompts `Please review the codebase` → `agy` uncovers **8 bugs** in its own one-shot build → operator prompts `please fix`. |
| **16–17** | `25:10`–`30:07` | **Browser Verification** | Operator prompts `please test this site in a web browser` → `agy` drives Chrome via `chrome-devtools-mcp`, catching and fixing a missing favicon `404` and Chrome autofill warnings. |

### What Went Right in Session 1?

Look at **Stops 5–7** in the cast:
1. In **Q2** (Stop 6), the operator unchecked *"Automatic stock deduction"* and typed a custom write-in answer: **`Allocate parts to a project`**.
2. In **Q3** (Stop 7), `agy` immediately adapted its next question to disambiguate what "allocate" meant, proposing:
   > `Track 'Total On Hand' vs. 'Allocated' vs. 'Available' — allocating reserves parts for a project (reducing Available stock), and you can un-allocate or finalize/consume them when built`
3. By **Stop 11** (`16m 05s` elapsed), the operator had a working local web app ("PatchBay & Bench") running on `http://localhost:3000` using Node's built-in `node:sqlite` (`DatabaseSync`), complete with `<dialog closedby="any">` modals and responsive `@container` tables picked up from the user's ambient `modern-web-guidance` plugin.

If your goal was **rapid exploration**—to see what a workbench inventory UI feels like in under 20 minutes—Session 1 was a massive success!

---

## 3. What Happened After `/clear`? The Hidden Cost of One-Shot Builds {#after-clear}

At **Stop 12**, the operator typed `/clear` and asked the fresh session:

```text caption="Stop 13 — Prompting a fresh session after /clear"
Please review the codebase
```

Because Session 1 generated ~2,500 lines of SQL, Express routes, and DOM manipulation in a single unverified pass—and wrote **zero durable domain documents** (`0` `GLOSSARY.md` files, `0` ADRs, `0` automated test files)—Session 2 found **8 concrete bugs** hiding beneath the surface:

### Bug Pattern 1: Multi-Line BOM Stock Over-Counting (`db.js`)

In the one-shot build, `getAllProjects()` checked whether each BOM line had enough `qty_available` **independently**, forgetting that two lines in the same Kit (or across Kits in the shopping list) might share the same `TL072` op-amp or `100nF` decoupling capacitor:

```javascript caption="experiments/audio_inventory_baseline/db.js — The one-shot bug vs. the Session 2 cumulative fix"
// ❌ BEFORE (Session 1 one-shot build):
// Checks each BOM line in isolation! If Line 1 and Line 2 both need 2x TL072
// and only 2 are available, BOTH lines report "Ready"!
const isReady = b.qty_available >= needed;

// ✅ AFTER (Session 2 fix after "Please review the codebase"):
// Tracks cumulative component consumption across earlier lines in the project:
const compId = b.component_id;
const prevUsed = usedByComp.get(compId) || 0;
const availAfterPrev = Math.max(0, (b.qty_available || 0) - prevUsed);
const coveredFromStock = Math.min(stillNeeded, availAfterPrev);
usedByComp.set(compId, prevUsed + coveredFromStock);
```

### Bug Pattern 2: Broken Allocation Invariants When Editing Stock (`server.js`)

Because the domain rule $\text{Available} = \text{On Hand} - \text{Allocated}$ lived only in Session 1's chat transcript, `PUT /api/components/:id` and `PATCH /api/components/:id/stock` allowed a user to decrement `qty_on_hand` below the sum of parts already allocated to active Kits—producing **negative available stock**!

### Bug Pattern 3: Electronics Shorthand Regex Collision (`4k7Ω` in `db.js`)

In Session 1, `normalizeElectronicValue('4k7Ω')` replaced `Ω` with `r` first (`4k7r`), which then failed to match the EE-infix regex `/^(\d+)([rkmnup])(\d+)(f?)$/`:

```javascript caption="experiments/audio_inventory_baseline/db.js — FixingΩ stripping before EE-infix parsing"
// Strip trailing ohm symbols (Ω, ohm, ohms) BEFORE checking 4k7 infix notation,
// while preserving trailing 'R' on plain values like 470R:
s = s.replace(/(?:ω|ohm|ohms)$/i, '');
const infixMatch = s.match(/^(\d+)([rkmnup])(\d+)(r|f|h)?$/);
```

---

## 4. Interactive Explorer: Baseline Vibe-Coding vs. Skilled Engineering {#interactive-explorer}

Click through the stages below to compare how the **Baseline Run** (`01-baseline.cast`) and the **Skilled Run** (`02-skilled.cast`) handled each phase of building the application:

{{widget:run-comparison-explorer}}

---

## 5. Interactive Exercise: Auditing a One-Shot Prototype {#exercise-0001}

Complete the fresh-context review sequence and the cumulative BOM allocation guard that Session 2 added to `experiments/audio_inventory_baseline/db.js`:

:::exercise id="ex-0001" title="Interactive Lab: Fresh-Context Review & Cumulative Stock Guard" badge="Baseline Run • Verification Loop"
Fill in the missing command and JavaScript expressions used in Session 2 of the **Baseline Run**:

```javascript caption="Auditing multi-line BOM stock allocation after a session reset"
// 1. In agy, reset conversational memory before asking for an objective review:
// Command: {{blank:/clear|Use the slash command that clears session context|/clear|10}}

function computeCoveredFromStock(bomLine, usedByComp) {
  const stillNeeded = Math.max(0, bomLine.qty_required - bomLine.qty_allocated);
  const prevUsed = usedByComp.get(bomLine.component_id) || 0;
  // 2. Subtract prevUsed from bomLine.qty_available so shared parts aren't double-counted:
  const availAfterPrev = Math.max(0, bomLine.qty_available - {{blank:prevUsed|Subtract the quantity already used by earlier BOM lines|prevUsed|12}});
  const covered = Math.min(stillNeeded, {{blank:availAfterPrev|Cap covered stock by what remains available after earlier lines|availAfterPrev|16}});
  usedByComp.set(bomLine.component_id, prevUsed + covered);
  return covered;
}
```

:::prompts
hint: "Use `/clear` to reset context, subtract `prevUsed` from `bomLine.qty_available`, and pass `availAfterPrev` to `Math.min`."
success: "Spot on! Resetting context with `/clear` and reviewing with fresh eyes caught the exact multi-line BOM double-counting bug in `db.js`."
:::
:::

---

## 6. Try It Yourself: Run a 15-Minute Baseline Exploration on Your Own Idea {#verify-0001}

You don't need to build an audio inventory app to experience this. Pick **any small project idea of your own** (a book/vinyl tracker, a home lab port registry, a workout log, or a recipe scaling calculator) and try the Baseline exploration loop:

:::verify id="verify-0001" title="Hands-On Exploration Gate: Your Own Baseline Run" badge="Bring Your Own Idea • 15 Mins"
Run these steps in your terminal to experience both the speed and the `/clear` boundary of Baseline Vibe-Coding:

```bash caption="1. Confirm your global clarifying-question rule & start a clean exploration directory"
cat "$HOME/.gemini/config/AGENTS.md"
mkdir -p ~/Demos/my-baseline-spike && cd ~/Demos/my-baseline-spike
agy
```

- [ ] Confirm `$HOME/.gemini/config/AGENTS.md` contains `"When something is unclear, please ask clarifying questions, one at a time."`
- [ ] Give `agy` a **single-sentence prompt** describing your own project idea without specifying a tech stack or database.
- [ ] Answer `agy`'s 3–5 clarifying questions and let it build a working prototype in one shot.
- [ ] Once it runs, type `/clear` and prompt: `Please review the codebase`. Observe what edge cases, missing invariants, or security issues the fresh session discovers!
:::

---

## 7. Retrieval Practice {#quiz-0001}

:::quiz id="quiz-0001" title="Retrieval Practice: When Vibe Coding Works & Where It Breaks" badge="3 Questions • Equal-Length Options"
- question: "When is Baseline Vibe-Coding (using only the global clarifying-question rule) the strongest engineering choice?"
  wrong_hint: "Think about uncertainty and how quickly you want to see a concrete working prototype."
  options:
    - letter: "A"
      text: "When you are rapidly exploring an unfamiliar problem space to discover requirements."
      correct: true
      explain: "Correct! Vibe coding gets you from a fuzzy one-sentence idea to a clickable prototype in ~15 minutes so you can see what you actually want."
    - letter: "B"
      text: "When you are migrating a multi-service banking ledger to production cloud infrastructure."
      correct: false
      explain: "Production financial or cloud migrations require durable specifications, repository seams, and automated regression tests."
    - letter: "C"
      text: "When ten autonomous subagents must collaborate across isolated git worktrees after clear."
      correct: false
      explain: "Multi-session and multi-agent collaboration breaks down without durable repo artifacts like GLOSSARY.md and spec.md."
    - letter: "D"
      text: "When you need zero human review and guaranteed transactional database invariants immediately."
      correct: false
      explain: "As Session 2 of the Baseline Run showed, one-shot builds frequently miss multi-line stock and allocation invariants until reviewed."

- question: "Why did `agy` ask four clarifying questions instead of guessing immediately in the Baseline Run?"
  wrong_hint: "Check the two-line configuration file in the user's home directory (`~/.gemini/config/AGENTS.md`)."
  options:
    - letter: "A"
      text: "Because the repository already contained thirty-eight workflow skills inside the agents folder."
      correct: false
      explain: "The Baseline Run started in a completely empty directory with zero repository skills installed."
    - letter: "B"
      text: "Because the user's global AGENTS.md rule instructed it to ask clarifying questions."
      correct: true
      explain: "Correct! That two-line global rule in `~/.gemini/config/AGENTS.md` triggered the `ask_question` discovery interview."
    - letter: "C"
      text: "Because Node SQLite refuses to initialize a database schema without four prompts."
      correct: false
      explain: "`node:sqlite` is just a library; the questioning behavior came entirely from the global `AGENTS.md` instruction."
    - letter: "D"
      text: "Because Chrome DevTools MCP blocked the terminal until interactive options were chosen."
      correct: false
      explain: "Chrome DevTools MCP was only invoked at the end of Session 2 during live browser testing."

- question: "What happened right after the operator ran `/clear` at Stop 12 of the Baseline Run?"
  wrong_hint: "Consider where the domain rules (`On Hand` vs. `Allocated` vs. `Available`) were stored during Session 1."
  options:
    - letter: "A"
      text: "The agent automatically read GLOSSARY.md and ran forty-one unit tests in memory."
      correct: false
      explain: "Those artifacts belong to the Skilled Run (Run 2); the Baseline Run produced zero markdown docs or test files."
    - letter: "B"
      text: "The background Express server deleted all seeded audio components from local disk."
      correct: false
      explain: "`/clear` only resets the agent's conversational context; the SQLite database file on disk remained intact."
    - letter: "C"
      text: "Conversational memory vanished, and a fresh review uncovered eight domain code bugs."
      correct: true
      explain: "Correct! With chat memory cleared, the fresh session inspected the code objectively and found 8 concrete bugs in the one-shot build."
    - letter: "D"
      text: "The agent refused to read JavaScript files until Wayfinder tickets were created."
      correct: false
      explain: "Session 2 happily read `package.json`, `db.js`, `server.js`, and `app.js` and performed a thorough code review."
:::

:::teacher-prompt
**Next up**: Now that you've seen how fast Baseline Vibe-Coding is for initial exploration—and where it hits a wall after `/clear`—head to [Lesson 0002: Wayfinding & Domain Modeling — Durable Intent Across `/clear`](0002-wayfinding-and-domain-modeling-with-skills.md) to see what changes when we install `mattpocock/skills`!
:::

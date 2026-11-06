---
title: "Vibe Coding vs. Skill-Driven Engineering in Antigravity (agy)"
page_title: "Codelab Index — Shaping AI Coding Agents with Skills"
nav_id: "index"
nav_label: "Index"
badge: "Codelab • Vibe Coding, Wayfinding & Google Cloud Skills"
subtitle: "Explore when to use fast exploratory vibe-coding vs. structured Agent Skills (mattpocock/skills & google/skills)—grounded in a recorded three-run experiment and designed for you to test on your own project ideas."
---

## Codelab Mission: Neither Approach Is Always Right or Always Wrong

When you sit down with an AI coding agent like Google Antigravity (`agy`), you have two fundamentally different ways of working:

1. **Vibe Coding (Rapid Exploration)**: Give the agent a high-level prompt, answer a few quick questions, and let it build a working prototype in 15 minutes.
2. **Skill-Driven Engineering (`/wayfinder` & Composable Skills)**: Equip the workspace with Agent Skills (`mattpocock/skills` and `google/skills`) to interview you deeply, commit a domain glossary (`GLOSSARY.md`), record Architecture Decision Records (`docs/adr/`), prototype UI variants, slice vertical tickets, and build with TDD and automated code review across `/clear` session boundaries.

**The goal of this Codelab is not to make you copy-paste our audio electronics inventory app.** Instead, we use the recorded **Baseline Run** (`01-baseline.cast`) and **Skilled Run** (`02-skilled.cast`) as a transparent field study—complete with real **Terminal Cast** stop points and **Session Traces**—so you can see the pros and cons of each mode firsthand, and apply them to **your own requirements and project ideas**.

```mermaid caption="Figure 0.1: Choosing your workflow across the 5 Codelab lessons—from rapid vibe-coding exploration to skill-driven engineering and cloud production."
flowchart LR
  L1["0001: Baseline Vibe-Coding<br/>(Fast Exploration &amp; Limits)"] --> L2["0002: /wayfinder &amp; Domain<br/>Modeling (GLOSSARY &amp; ADRs)"]
  L2 --> L3["0003: /prototype, /to-spec<br/>&amp; /to-tickets"]
  L3 --> L4["0004: /implement-spec,<br/>Worktree TDD &amp; /code-review"]
  L4 --> L5["0005: Capstone Exercise<br/>(google/skills on Cloud Run)"]
```

## Published Codelab Lessons {#lessons}

The codelab unfolds across five sequential lessons that follow our three-run experiment from an initial rapid prototype to a cloud-ready architecture:

We begin in **[Lesson 0001: Baseline Vibe-Coding — Rapid Exploration & Its Trade-Offs](lessons/0001-baseline-vibe-coding-for-rapid-exploration.md)** (`01-baseline.cast`), where a single global clarifying-question rule turns a one-sentence prompt into a working prototype in 16 minutes—highlighting both what went right for fast discovery and the eight bugs exposed once the session was cleared with `/clear`.

From there, the next three lessons trace the **Skilled Run** (`02-skilled.cast`). **[Lesson 0002: Wayfinding & Domain Modeling — Durable Intent Across `/clear`](lessons/0002-wayfinding-and-domain-modeling-with-skills.md)** walks through Sessions 1–5, showing how `/wayfinder` and `/domain-modeling` uncovered hierarchical Storage Locations, multi-location Stock Lots, and five-state BOM fulfillment while persisting those decisions in `GLOSSARY.md` and ADRs. **[Lesson 0003: UI Prototyping, Specs & Vertical-Slice Tickets](lessons/0003-prototyping-specs-and-vertical-slice-tickets.md)** continues through Sessions 6–8 as we explore interactive layout Variants A, B, and C with `/prototype`, synthesize `spec.md` via `/to-spec`, and slice the work into eight parallelizable vertical tickets with `/to-tickets`. **[Lesson 0004: Worktree TDD, Two-Axis Code Review & Live Browser Verification](lessons/0004-worktree-tdd-code-review-and-browser-verification.md)** completes the Skilled Run in Sessions 9–10, examining how `/implement-spec` orchestrated 11 subagents across isolated git worktrees, what two-axis `/code-review` caught, and why live browser verification still mattered.

Finally, **[Lesson 0005: Capstone Exercise — Productionizing on Google Cloud with `google/skills`](lessons/0005-capstone-exercise-productionizing-with-google-cloud-skills.md)** brings the workflow into the **Production Run**, demonstrating how `google/skills` (`cloud-databases-onboarding`, `google-cloud-waf-cost-optimization`, `cloud-run-basics`, and `gcloud`) compose with `/wayfinder` so you can apply the full toolkit to your own project on scale-to-zero Cloud Run and Cloud Firestore.

## Reference Sheets & Printable Guides {#reference-sheets}

Keep these reference sheets open alongside the lessons—or print them directly (`Cmd+P`), which automatically switches to a high-contrast light print stylesheet:

{{widget:reference-cards}}

:::teacher-prompt
**Ready to explore?** Start with [Lesson 0001: Baseline Vibe-Coding — Rapid Exploration & Its Trade-Offs](lessons/0001-baseline-vibe-coding-for-rapid-exploration.md), and bring a project idea of your own that you'd like to test!
:::

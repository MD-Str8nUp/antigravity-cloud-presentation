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

{{widget:lesson-cards}}

## Reference Sheets & Printable Guides {#reference-sheets}

Keep these reference sheets open alongside the lessons—or print them directly (`Cmd+P`), which automatically switches to a high-contrast light print stylesheet:

{{widget:reference-cards}}

:::teacher-prompt
**Ready to explore?** Start with [Lesson 0001: Baseline Vibe-Coding — Rapid Exploration & Its Trade-Offs](lessons/0001-baseline-vibe-coding-for-rapid-exploration.md), and bring a project idea of your own that you'd like to test!
:::

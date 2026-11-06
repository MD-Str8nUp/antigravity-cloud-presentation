# Teaching Notes & Curriculum Roadmap

## Core Pedagogical Stance
- **Do not ask the reader to clone or re-type the audio inventory app step by step.** The recorded audio electronics **Component** and **Kit** web system is our **case study**, not a typing test.
- **Emphasize pragmatic trade-offs over dogma**:
  - **Vibe Coding (Baseline Run)** is genuinely great when you want to explore an unfamiliar problem space quickly, see a working prototype in 15 minutes, and figure out what you actually want.
  - **Skill-Driven Engineering (Skilled Run with `mattpocock/skills`)** is great once you know the project matters and you need durable domain documentation (`GLOSSARY.md`, `docs/adr/`), deep module boundaries, and automated verification (`TDD` + 2-axis `/code-review`) that survive `/clear` for ongoing maintenance.
- **Open-ended Capstone (`google/skills`)**: Leave the Google Cloud production migration (**Cloud Run** + **Cloud Firestore**) as a guided capstone exercise at the end (`Lesson 0005`) where the reader applies `google/skills` to their own project or an existing local prototype.

## 5-Lesson Codelab Arc (`./codelab/lessons/`)
1. **`0001-baseline-vibe-coding-for-rapid-exploration.md`**:
   - Walks through the recorded **Baseline Run** (`01-baseline.cast`, `audio_inventory_baseline`).
   - Shows how a single global rule (`When something is unclear, please ask clarifying questions, one at a time.`) + ambient `modern-web-guidance` built a rich Node.js + SQLite prototype in 16 minutes—and why `/clear` exposed 8 bugs and zero durable domain docs.
   - Reader Lab: Run a 15-minute Baseline Vibe-Coding session on your own project idea and test what happens after `/clear`.
2. **`0002-wayfinding-and-domain-modeling-with-skills.md`**:
   - Walks through Sessions 1–5 of the **Skilled Run** (`02-skilled.cast`, `audio_inventory_skilled`) after installing `mattpocock/skills`.
   - Shows how `/wayfinder`, `/grilling`, and `/domain-modeling` uncovered real-world domain requirements (hierarchical **Storage Locations**, multi-location **Stock Lots**, 5-state **Fulfillment Status**, and multi-build **Net Shortfall**) and wrote `GLOSSARY.md` and `ADR 0001` to disk before writing application code.
   - Reader Lab: Chart a `/wayfinder` map and `GLOSSARY.md` for your own project idea.
3. **`0003-prototyping-specs-and-vertical-slice-tickets.md`**:
   - Walks through Sessions 6–8 of the **Skilled Run**: generating a 3-variant interactive HTML UI prototype (`/prototype`), synthesizing `spec.md` (`/to-spec`), and slicing work into 8 dependency-ordered vertical tickets (`/to-tickets`).
   - Reader Lab: Use `/prototype`, `/to-spec`, and `/to-tickets` to turn your own project's Wayfinder decisions into an actionable build plan.
4. **`0004-worktree-tdd-code-review-and-browser-verification.md`**:
   - Walks through Sessions 9–10 of the **Skilled Run**: autonomous multi-worktree TDD orchestration (`/implement-spec`), parallel 2-axis `/code-review` (**Standards** + **Spec**), and live browser verification via Chrome DevTools MCP (catching subtle HTMX out-of-band swap and trigger-scoping bugs).
   - Reader Lab: Execute and verify a vertical slice of your own project across a `/clear` boundary.
5. **`0005-capstone-exercise-productionizing-with-google-cloud-skills.md`**:
   - Walks through the architectural **Production Run** (`docs/codelab-run-3-production.md`), composing `mattpocock/skills` with `google/skills` (`cloud-databases-onboarding`, `google-cloud-waf-cost-optimization`, `cloud-run-basics`, `gcloud`).
   - Open-Ended Reader Exercise: Install `google/skills` and guide `agy` through designing and validating a scale-to-zero Google Cloud deployment for your own application.

# Shaping AI Coding Agents with Skills — Resources

## Knowledge

- [Experiment Repository: `domesticmouse/antigravity-cloud-presentation`](https://github.com/domesticmouse/antigravity-cloud-presentation)
  Contains the sanitized `asciinema` **Terminal Casts** (`recordings/final/01-baseline.cast`, `02-skilled.cast`), standalone 16:9 HTML cast players (`prototypes/`), exported Antigravity **Session Traces** (`artifacts/traces/`), and synced application workspaces (`experiments/audio_inventory_baseline/`, `experiments/audio_inventory_skilled/`). Use for: inspecting the exact prompts, tool calls, and code produced in each run.
- [`mattpocock/skills` — Engineering Workflow & Domain Modeling Skills](https://github.com/mattpocock/skills)
  Open-source collection of composable agent skills (`/wayfinder`, `/grilling`, `/domain-modeling`, `/codebase-design`, `/prototype`, `/to-spec`, `/to-tickets`, `/implement-spec`, `/tdd`, `/code-review`). Use for: turning an AI coding agent into a structured design partner that commits durable intent artifacts (`GLOSSARY.md`, `docs/adr/`, `spec.md`) and builds vertical slices with TDD.
- [`google/skills` — Official Google Cloud Agent Skills](https://github.com/google/skills)
  Official Google Cloud skills (`cloud-databases-onboarding`, `google-cloud-waf-cost-optimization`, `cloud-run-basics`, `gcloud`, `firestore-rules-creation`). Use for: evaluating cloud database options, optimizing serverless FinOps (`--min-instances=0`), and enforcing `gcloud help <leaf_command>` CLI validation guardrails.
- [`domesticmouse/skills` — Codelab, Recording & Presentation Skills](https://github.com/domesticmouse/skills)
  Agent skills for interactive code-teaching courses (`/teach-code`), `asciinema` terminal cast processing (`/asciinema`, `/cast-player-html`), session trace extraction (`/agy-brain-explorer`), and Slidev decks (`/slidev-deck`). Use for: turning recorded agent experiments into reproducible Codelabs and presentations.
- [Google Cloud Run Documentation](https://cloud.google.com/run/docs)
  Official documentation for container contracts (`0.0.0.0:$PORT`), scale-to-zero request billing, and source/container deployments. Use for: Lesson 0005 capstone deployment.
- [Google Cloud Firestore Documentation](https://cloud.google.com/firestore/docs)
  Official documentation for Firestore Native mode document modeling, atomic transactions, and free-tier quotas. Use for: Lesson 0005 persistence seam design.

## Wisdom (Communities)

- [Agent Skills Ecosystem (`npx skills`)](https://github.com/vercel-labs/skills)
  Open CLI ecosystem for discovering, sharing, and installing portable `.agents/skills/` across Antigravity (`agy`), Claude Code, Cursor, and other coding agents. Use for: exploring community skills and publishing your own workflow skills.
- [Google Cloud Community & Serverless Forums](https://www.googlecloudcommunity.com/)
  Practitioner discussions on Cloud Run, Firestore, and Well-Architected FinOps patterns. Use for: real-world serverless architecture trade-offs.

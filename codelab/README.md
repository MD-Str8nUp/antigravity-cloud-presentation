# Interactive Codelab: Vibe Coding vs. Skill-Driven Engineering in Antigravity (`agy`)

This directory (`./codelab/`) contains the interactive `/teach-code` **Codelabs** for the **Antigravity Cloud Website Experiment**.

Rather than asking you to copy-paste our audio electronics inventory app line by line, these **Codelabs** use the recorded three-run experiment (**Baseline Run**, **Skilled Run**, and architectural **Production Run**) as a concrete case study to help you explore **your own requirements**—and develop engineering judgment around when to **vibe-code** for rapid discovery versus when to use **Agent Skills** (`mattpocock/skills` and `google/skills`) for long-term maintainability.

## Published Lessons

- [Course Index (`index.md`)](./index.md)
- [Lesson 0001: Baseline Vibe-Coding — Rapid Exploration & Its Trade-Offs (`lessons/0001-baseline-vibe-coding-for-rapid-exploration.md`)](./lessons/0001-baseline-vibe-coding-for-rapid-exploration.md)
- [Lesson 0002: Wayfinding & Domain Modeling — Durable Intent Across `/clear` (`lessons/0002-wayfinding-and-domain-modeling-with-skills.md`)](./lessons/0002-wayfinding-and-domain-modeling-with-skills.md)
- [Lesson 0003: UI Prototyping, Specs & Vertical-Slice Tickets (`lessons/0003-prototyping-specs-and-vertical-slice-tickets.md`)](./lessons/0003-prototyping-specs-and-vertical-slice-tickets.md)
- [Lesson 0004: Worktree TDD, Two-Axis Code Review & Live Browser Verification (`lessons/0004-worktree-tdd-code-review-and-browser-verification.md`)](./lessons/0004-worktree-tdd-code-review-and-browser-verification.md)
- [Lesson 0005: Capstone Exercise — Productionizing on Google Cloud with `google/skills` (`lessons/0005-capstone-exercise-productionizing-with-google-cloud-skills.md`)](./lessons/0005-capstone-exercise-productionizing-with-google-cloud-skills.md)

## Reference Sheets & Printable Guides

- [Glossary of Experiment, Workflow & Domain Terms (`reference/GLOSSARY.md`)](./reference/GLOSSARY.md)
- [Vibe Coding vs. Agent Skills Decision & Workflow Cheat Sheet (`reference/agent-skills-workflow-cheatsheet.md`)](./reference/agent-skills-workflow-cheatsheet.md)
- [Google Cloud Skills & Scale-to-Zero Serverless Reference (`reference/google-cloud-skills-and-serverless-reference.md`)](./reference/google-cloud-skills-and-serverless-reference.md)

## Building & Validating the Static Codelab Site (`build.py`)

```bash
cd codelab
uv venv
uv pip install markdown-it-py pyyaml
.venv/bin/python build.py --check
```

## License

Licensed under the [Apache License, Version 2.0](../LICENSE).

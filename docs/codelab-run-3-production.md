# Codelab Script — Stage 3: The Production Run (`google/skills` Capstone Exercise)

This document chronicles Stage 3 (**Production Run**) of the Antigravity Cloud Website Experiment and serves as the script for the **Production Run** **Codelab** and Slidev act.

Unlike the recorded **Baseline Run** ([`01-baseline.cast`](../recordings/final/01-baseline.cast)) and **Skilled Run** ([`02-skilled.cast`](../recordings/final/02-skilled.cast)), the **Production Run** is designed as an **architectural walkthrough and open-ended reader capstone exercise**. Rather than asking learners to mechanically replicate a single pre-scripted deployment, it shows how the durable artifacts created during the **Skilled Run** prepare a codebase for architectural change—and invites the reader to install [`google/skills`](https://github.com/google/skills) and take **their own application** (or the completed **Skilled Run** workspace) from a local prototype to a scale-to-zero Google Cloud deployment.

---

## 1. The Core Thesis: Choosing the Right Mode for the Stage of Your Idea

Before adding cloud skills, pause and compare what the first two runs actually proved:

| Mode | Best Used When... | What You Gain | What You Trade Off |
| :--- | :--- | :--- | :--- |
| **Vibe Coding** (**Baseline Run** — Global Clarifying-Question Rule only) | You are **exploring an unfamiliar problem space**, testing whether an idea is worth building, or trying to see a working UI in 15 minutes. | Maximum speed to a tangible prototype (`16m` to build "PatchBay & Bench"); low ceremony; helps you discover requirements you didn't know you had. | Zero durable institutional memory (`0` `GLOSSARY.md` or ADR files); tight coupling between routes and raw SQLite; subtle multi-entity domain bugs that require manual review after `/clear`. |
| **Skill-Driven Engineering** (**Skilled Run** — `mattpocock/skills`) | You have validated that the problem is worth solving and want a **well-engineered, maintainable system** that survives `/clear` and future architectural changes. | Ubiquitous domain language (`GLOSSARY.md`), ADRs (`docs/adr/`), UI options (`/prototype`), vertical-slice TDD (`41` passing tests), parallel 2-axis `/code-review`, and clean module boundaries. | Higher upfront investment in structured discovery (`/wayfinder` → `/to-spec` → `/to-tickets` → `/implement-spec`) before the first line of production code is written. |

**Neither approach is always right or always wrong:**
- If you run `/wayfinder` on a 20-minute throwaway script, the ceremony outweighs the value.
- If you vibe-code a system you plan to maintain, extend, and migrate to production infrastructure, every future `/clear` session starts with amnesia—forcing the agent to reverse-engineer your domain rules from implementation accidents.

---

## 2. Why the Skilled Run Is Ready for Cloud Migration (and the Baseline Run Isn't)

Suppose you wake up a month later, open a fresh terminal (`Session N` after `/clear`), and decide to move your local workbench app to Google Cloud so you can access it from your phone at the electronics bench or local maker space—without paying for an always-on database VM.

### What Happens in the Baseline Run Workspace (`experiments/audio_inventory_baseline/`)
1. **Context Amnesia**: There is no `GLOSSARY.md`, no `docs/adr/`, and no `spec.md`. A fresh agent session must read 2,500 lines of `db.js`, `server.js`, and `public/app.js` just to infer what `Allocated` vs. `Available` means.
2. **Tightly Coupled Synchronous SQL**: Express route handlers in [`server.js`](../experiments/audio_inventory_baseline/server.js) reach directly into synchronous `node:sqlite` (`DatabaseSync`) prepared statements in [`db.js`](../experiments/audio_inventory_baseline/db.js).
3. **Zero Regression Safety Net**: Because there is no automated test suite (`npm test` does not exist), swapping SQLite for an async cloud database like **Cloud Firestore** risks silently re-introducing the exact multi-line BOM over-counting and allocation clamping bugs fixed in Session 2.

### What Happens in the Skilled Run Workspace (`experiments/audio_inventory_skilled/`)
1. **Durable Institutional Memory Survives `/clear`**:
   - [`GLOSSARY.md`](../experiments/audio_inventory_skilled/GLOSSARY.md) defines every core entity (`Component`, `Storage Location`, `Stock Lot`, `Kit Build`, `BOM Line`, `Fulfillment Status`, `Available Stock`, `Bench Log Entry`, `Shopping List`, `Net Shortfall`).
   - [`docs/adr/0001-rust-htmx-turso-stack.md`](../experiments/audio_inventory_skilled/docs/adr/0001-rust-htmx-turso-stack.md) and [`.scratch/audio-inventory/spec.md`](../experiments/audio-inventory/spec.md) document the exact invariants (such as **Installed Stock Permanence** and the 5-state **Fulfillment Status** machine).
2. **Deep Domain Modules & Seam Readiness**:
   - Domain logic is cleanly partitioned into 5 deep modules (`spec_parser`, `catalog`, `builds`, `imports`, `shopping_list`) separated from HTTP/HTMX rendering (`web.rs`).
3. **41 Automated Tests as a Migration Safety Net**:
   - Extracting a repository trait seam (`InventoryRepository`) and adding a cloud persistence adapter can be verified against the existing 41 domain and integration tests at every step.

---

## 3. Composing `mattpocock/skills` + `google/skills` for the Production Run

When we augment a workspace with Google Cloud's official agent skills (`npx skills@latest add google/skills`), the two skill sets compose across the engineering lifecycle:

| Engineering Phase | `mattpocock/skills` (Workflow & Architecture) | `google/skills` (Cloud Domain & Operational Guardrails) |
| :--- | :--- | :--- |
| **1. Discovery & Database Selection** | `/wayfinder`, `/grilling`, `/domain-modeling` | **`cloud-databases-onboarding`** (3-phase Discovery → Matrix Consultation → Plan-Validate-Execute workflow) |
| **2. FinOps & Cost Architecture** | `/wayfinder` standing constraints & ADR authoring | **`google-cloud-waf-cost-optimization`** (Google Cloud Well-Architected Framework Cost Optimization pillar: scale-to-zero serverless, single-region alignment, resource labels, budget alerts) |
| **3. Persistence Seam & Security** | `/codebase-design` (deep modules & repository seams), `/tdd` | **`firestore-rules-creation`** / `firebase-firestore` (document/collection modeling, atomic transactions, least-privilege security rules) |
| **4. Containerization & Deployment** | `/to-spec`, `/to-tickets`, `/implement-spec`, `/code-review` | **`cloud-run-basics`** (`0.0.0.0:$PORT` contract, `--min-instances=0`, `us-central1`) & **`gcloud`** (mandatory `gcloud help <leaf_command>` validation, `--dry-run`, explicit `--project` & `--region`, data reduction flags, and usage attribution) |

---

## 4. Architectural Walkthrough: Adapting the Workbench for Cloud Run + Cloud Firestore

When you prompt `agy` in a workspace equipped with both `mattpocock/skills` and `google/skills`:

```text
/wayfinder I'd like to adapt and productionize this web application for a low-volume, low-cost deployment on Google Cloud using Cloud Run and Cloud Firestore.
```

Here are the four key architectural decisions that `google/skills` and `/wayfinder` surface:

### Decision 3.1 — Database Selection & FinOps (`cloud-databases-onboarding` + `google-cloud-waf-cost-optimization`)
- **Why Not Cloud SQL or AlloyDB for a Personal Workbench?**
  - Provisioning an always-on relational VM (Cloud SQL for PostgreSQL or AlloyDB) incurs continuous hourly instance and storage charges even when you only open your electronics inventory twice a week on the weekend.
- **Why Cloud Firestore (Native Mode) + Cloud Run (`--min-instances=0`)?**
  - **Cloud Firestore** bills strictly per document read/write/delete and GiB stored, with a generous daily free tier (50,000 reads, 20,000 writes, 20,000 deletes per day, and 1 GiB storage).
  - **Cloud Run** with `--min-instances=0` in `us-central1` scales down to zero containers when idle (`$0/month` idle compute cost) while spinning up in milliseconds when HTMX requests arrive (especially fast with a small compiled Rust binary).
  - **Single-Region Co-Location**: Co-locating both the Cloud Run service and the `(default)` Firestore database in `us-central1` avoids cross-region network egress charges and minimizes round-trip latency for HTMX partial swaps.

### Decision 3.2 — Extracting the Repository Seam (`/codebase-design`)
- Rather than ripping out local Turso (`libsql`) and losing fast offline `cargo test` execution, `/codebase-design` guides extracting a deep **Repository Trait Seam**:
  - **Local / Test Adapter (`TursoRepository`)**: Keeps `:memory:` and local file-backed Turso so all 41 unit/integration tests run offline in milliseconds during development and CI.
  - **Production Adapter (`FirestoreRepository`)**: Maps **Storage Locations**, **Components** (with embedded or subcollection **Stock Lots**), and **Kit Builds** (with **BOM Lines** and **Bench Log Entries**) to Firestore documents, using Firestore atomic transactions (`commit` / read-before-write transactions) to enforce the 5-state **Fulfillment Status** and **Installed Stock Permanence** invariants.

### Decision 3.3 — Cloud Run Container Contract (`cloud-run-basics`)
- The `cloud-run-basics` skill enforces a critical runtime guardrail:
  > **CRITICAL RULE:** Any deployed code MUST listen on `0.0.0.0` (not `127.0.0.1`) and use the injected `$PORT` environment variable (defaults to `8080`), or it will crash on boot.
- Notice how in the **Skilled Run** (`experiments/audio_inventory_skilled/src/main.rs`), the local server bound to `127.0.0.1:3000` by default. Guided by `cloud-run-basics`, the production entrypoint reads `HOST` (defaulting to `0.0.0.0` in container builds) and `PORT` (defaulting to `8080`), and packages the Rust binary + vendored `assets/` into a minimal multi-stage `Dockerfile` (`rust:1-bookworm` builder → `debian:bookworm-slim` runtime with `ca-certificates`).

### Decision 3.4 — Safe, Validated `gcloud` Operations (`gcloud` Skill Guardrails)
- Left to parametric memory, LLMs routinely hallucinate stale `gcloud` flags or run unconstrained `gcloud ... list` commands that flood the context window.
- The `gcloud` skill enforces four mandatory guardrails:
  1. **Leaf-Level Syntax Validation First**: Always run `gcloud help <leaf_command>` (e.g., `gcloud help firestore databases create` and `gcloud help run deploy`) before proposing or executing any command.
  2. **Explicit Project & Location Scoping**: Always pass `--project=<PROJECT_ID>` and `--region=us-central1` (or `--location=us-central1`) so commands never hang on interactive region prompts.
  3. **Data Reduction on Queries**: Never run a blind `list`; always include `--filter`, `--format`, or `--limit` (watching out for the `gcloud services list` `--limit` gotcha documented in `gcloud/SKILL.md`).
  4. **Human-in-the-Loop Authorization for IAM & API Enablement**: Never silently mutate IAM bindings or enable billing-impacting APIs without explicit operator confirmation.

---

## 5. Reader Capstone Exercise: Try It on Your Own Application

In the **Codelab** (`./codelab/lessons/0005-capstone-exercise-productionizing-with-google-cloud-skills.md`), readers are encouraged to apply this workflow to **their own project** (or clone `experiments/audio_inventory_skilled/`):

1. **Explore First, Engineer Second**:
   - Pick a domain you care about (a home lab tracker, a synth patch library, a book club organizer, or a team utility).
   - Try a 15-minute **Baseline Vibe-Coding** run first to see what UI and features feel right.
   - Then start a fresh directory, install `npx skills@latest add mattpocock/skills`, and run `/wayfinder` to experience how structured grilling and `GLOSSARY.md` sharpen your requirements.
2. **Add `google/skills` and Plan the Cloud Architecture**:
   - Install Google Cloud skills into your project:
     ```bash
     npx skills@latest add google/skills
     ```
   - Launch `agy` and run `/wayfinder` to evaluate how to host your application on Google Cloud within your budget and latency constraints:
     ```text
     /wayfinder I'd like to adapt and productionize this web application for a low-volume, low-cost deployment on Google Cloud.
     ```
3. **Observe How Skills Shape the Conversation**:
   - Watch how `cloud-databases-onboarding` interviews you about your workload before recommending a database.
   - Watch how `google-cloud-waf-cost-optimization` shapes region selection, scale-to-zero compute (`--min-instances=0`), and cost attribution labels.
   - Watch how `cloud-run-basics` and `gcloud` validate every CLI flag via `gcloud help <leaf_command>` before deployment.

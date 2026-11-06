---
title: "Google Cloud Skills & Scale-to-Zero Serverless Reference"
page_title: "Cloud Reference — Google Cloud Skills (google/skills)"
nav_id: "cloud-reference"
nav_label: "Cloud Skills Reference"
badge: "Reference • google/skills & Serverless FinOps"
subtitle: "Quick reference for composing google/skills with /wayfinder when taking your own application to Cloud Run and Cloud Firestore."
card:
  title: "**Google Cloud Skills & Scale-to-Zero Serverless Reference**"
  summary: "Guardrails, CLI validation rules, container contracts, and FinOps patterns from google/skills for the Lesson 0005 capstone."
---

## 1. Installing & Composing `google/skills`

Install the official Google Cloud agent skills into any project workspace alongside `mattpocock/skills`:

```bash caption="Installing Google Cloud skills into .agents/skills/"
npx skills@latest add google/skills
```

Table: Key Google Cloud skills for serverless web applications.
| Skill Name | When `agy` Activates It | Key Guardrails Enforced |
| :--- | :--- | :--- |
| **`cloud-databases-onboarding`** | Choosing a Google Cloud database ("What database should I use?") or drafting database provisioning plans. | 3-phase **Discovery → Matrix Consultation → Plan-Validate-Execute** workflow; never executes live mutating database writes without human review. |
| **`google-cloud-waf-cost-optimization`** | Evaluating cloud spend, FinOps alignment, scale-to-zero compute, or regional placement. | Aligns architecture with the **Inform → Optimize → Operate** FinOps lifecycle: serverless scale-to-zero, single-region placement, resource labels, and budget alerts. |
| **`cloud-run-basics`** | Containerizing and deploying HTTP services, jobs, or worker pools on Cloud Run. | Enforces the **`0.0.0.0:$PORT`** listening contract (`PORT` defaults to `8080`), `--min-instances=0` autoscaling, and Cloud Logging crash diagnostics. |
| **`gcloud`** | Planning, proposing, or executing any `gcloud` CLI command. | Mandatory **`gcloud help <leaf_command>`** validation before proposing flags; explicit `--project` and `--region`; mandatory `--filter`/`--format`/`--limit` on `list` commands; denylist for autonomous IAM/API/delete operations. |
| **`firestore-rules-creation`** | Authoring or reviewing `firestore.rules` for Cloud Firestore. | Enforces `rules_version = '2';`, explicit deny-by-default fallback, authentication/ownership checks, and schema validation. |

---

## 2. The `gcloud` Skill's 4-Step Mandatory Validation Pattern

Left to parametric memory, LLMs frequently hallucinate outdated `gcloud` flags. Whenever `gcloud/SKILL.md` is active, `agy` follows a strict 4-step validation discipline:

1. **Step 1 — Leaf-Level Syntax Validation**:
   Always run `gcloud help <leaf_command>` on the exact leaf subcommand (never just a parent group like `gcloud help run`):
   ```bash
   gcloud help firestore databases create
   gcloud help run deploy
   ```
2. **Step 2 — Parameter & Dry-Run Verification**:
   Confirm required flags, explicit `--project=<PROJECT_ID>`, explicit location (`--region=us-central1` or `--location=us-central1`), and whether `--dry-run` or `--validate-only` is supported.
3. **Step 3 — Data Reduction on Discovery Queries**:
   Never run an unconstrained `list` command. Always project and filter:
   ```bash
   gcloud services list --enabled \
     --filter="config.name:(run.googleapis.com OR firestore.googleapis.com OR cloudbuild.googleapis.com OR artifactregistry.googleapis.com)" \
     --format="table(config.name,state)" \
     --project="<PROJECT_ID>" \
     --quiet
   ```
   :::callout-warn
   **The `gcloud services list --limit` Gotcha**:
   `gcloud services list` applies `--limit=N` *before* client-side `--filter` evaluation! Combining `--limit=10` with `--filter="config.name:(run.googleapis.com)"` only checks the first 10 enabled APIs alphabetically and silently drops `run.googleapis.com`. Omit `--limit` when filtering `gcloud services list` by `config.name`.
   :::
4. **Step 4 — Safe Execution & Prohibited Operations Denylist**:
   Commands that modify IAM bindings (`add-iam-policy-binding`), enable billing-impacting APIs (`services enable`), or delete resources (`delete`) require explicit human confirmation.

---

## 3. Scale-to-Zero Serverless Checklist (`Cloud Run` + `Cloud Firestore`)

When adapting a local web application (like the **Skilled Run** workbench or your own project) for `$0/month` idle cost within the Google Cloud free tier:

Table: Production readiness checklist for scale-to-zero Cloud Run + Cloud Firestore.
| Requirement | Local Prototype Default | Production Cloud Run + Firestore Target |
| :--- | :--- | :--- |
| **HTTP Bind Address** | `127.0.0.1:3000` | `0.0.0.0:$PORT` (reading `std::env::var("PORT")` or `process.env.PORT` with fallback `8080`) |
| **Persistence Seam** | Direct SQLite / Turso file in local directory | Repository trait/interface seam supporting both local/in-memory tests and **Cloud Firestore (Native Mode)** |
| **Container Filesystem** | Local disk (`data/inventory.db`) persists across restarts | Stateless container memory; all durable state lives in **Cloud Firestore** (or Cloud Storage for large attachments) |
| **Region Alignment** | N/A | Co-locate Cloud Run (`--region=us-central1`) and Firestore (`--location=us-central1`) to eliminate cross-region egress fees and latency |
| **Instance Scaling** | Single local process | `--min-instances=0 --max-instances=3` (scales to zero when idle; caps unexpected traffic spikes) |
| **Cost Attribution** | N/A | Apply `--labels=env=prod,app=<name>` and configure a Cloud Billing budget alert |

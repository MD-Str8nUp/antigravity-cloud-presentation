---
title: "Capstone Exercise: Productionizing on Google Cloud with google/skills"
page_title: "Lesson 0005 — Capstone Exercise with google/skills"
nav_id: "lesson-0005"
badge: "Production Run • Open-Ended Reader Capstone"
subtitle: "Compose mattpocock/skills with Google Cloud's official agent skills (google/skills) to take your own application—or the Skilled Run workbench—to scale-to-zero Cloud Run and Cloud Firestore."
card:
  title: "**Capstone Exercise: Productionizing on Google Cloud with `google/skills`**"
  summary: "Learn how google/skills (cloud-databases-onboarding, google-cloud-waf-cost-optimization, cloud-run-basics, gcloud) compose with /wayfinder, then apply them to your own project."
---

## 1. Why the Skilled Run Makes Architectural Change Easy {#why-seams-matter}

Here is the ultimate test of whether a codebase was engineered for ongoing maintenance or just vibe-coded for a demo: **what happens when you change an architectural constraint?**

Suppose you have been using your local application for a few weeks and now want to host it on Google Cloud so you can access it from any device—under a strict hobbyist constraint:
- **Target `$0/month` idle cost** (within the free tier; no always-on database VMs burning `$30–$100/month` while you sleep).
- **Keep fast, offline unit tests** so you can still develop and run tests on an airplane without live cloud credentials.

Look at how differently that conversation goes depending on how the app was built:

1. **Starting from a Vibe-Coded Prototype (`audio_inventory_baseline`)**:
   - No `GLOSSARY.md`, no ADRs, and no `spec.md`.
   - Synchronous `node:sqlite` queries are interleaved directly inside Express routes.
   - Zero automated tests exist to tell you if migrating to a document database broke multi-line BOM allocation.
2. **Starting from a Skill-Engineered Codebase (`audio_inventory_skilled`)**:
   - Every domain concept (`Stock Lot`, `Fulfillment Status`, `Net Shortfall`, **Installed Stock Permanence**) is documented in [`GLOSSARY.md`](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/experiments/audio_inventory_skilled/GLOSSARY.md) and [`spec.md`](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/experiments/audio_inventory_skilled/.scratch/audio-inventory/spec.md).
   - Domain logic lives in 5 deep modules (`spec_parser`, `catalog`, `builds`, `imports`, `shopping_list`) backed by **41 automated tests**.

:::callout
**Primary Source**: Read the [Production Run Codelab Script (`docs/codelab-run-3-production.md`)](https://github.com/domesticmouse/antigravity-cloud-presentation/blob/main/docs/codelab-run-3-production.md) and our [Google Cloud Skills & Scale-to-Zero Serverless Reference](../reference/google-cloud-skills-and-serverless-reference.md).
:::

---

## 2. Composing `mattpocock/skills` + `google/skills` {#composing-skills}

Agent skills are **composable**. While `mattpocock/skills` teaches `agy` *how to run a disciplined engineering process* (`/wayfinder`, `/grilling`, `/codebase-design`, `/to-spec`, `/to-tickets`, `/tdd`, `/code-review`), installing [`google/skills`](https://github.com/google/skills) gives `agy` **domain-specific Google Cloud expertise and operational guardrails**:

```bash caption="Adding Google Cloud official skills alongside mattpocock/skills"
npx skills@latest add google/skills
```

```mermaid caption="Figure 5.1: How mattpocock/skills (workflow) and google/skills (cloud architecture & guardrails) compose during the Production Run."
flowchart TD
  W["/wayfinder Cloud Migration Prompt"] --> DB["cloud-databases-onboarding<br/>+ google-cloud-waf-cost-optimization<br/>(Selects Native Firestore + Scale-to-Zero)"]
  DB --> SEAM["/codebase-design + /tdd<br/>(Extracts Repository Trait Seam:<br/>TursoRepository + FirestoreRepository)"]
  SEAM --> CR["cloud-run-basics<br/>(Enforces 0.0.0.0:$PORT Contract<br/>&amp; --min-instances=0 in us-central1)"]
  CR --> GC["gcloud Skill Guardrails<br/>(Mandatory gcloud help &lt;leaf_command&gt;,<br/>Explicit --project/--region &amp; Data Reduction)"]
```

### Four Guardrails That `google/skills` Adds Immediately

1. **Workload-Driven Database Selection (`cloud-databases-onboarding` + `google-cloud-waf-cost-optimization`)**:
   - Instead of defaulting to an always-on Cloud SQL or AlloyDB instance, the database onboarding and FinOps skills evaluate the workload (low-volume personal/hobbyist traffic, spiky access, strict idle cost ceiling) and recommend **Cloud Firestore (Native Mode)** paired with **Cloud Run (`--min-instances=0`)** co-located in a single region (`us-central1`).
2. **The Cloud Run Container Contract (`cloud-run-basics`)**:
   - Local prototypes almost always bind to `127.0.0.1:3000`. On Cloud Run, binding to `127.0.0.1` or ignoring the injected `$PORT` environment variable causes an immediate container startup crash!
   - `cloud-run-basics` enforces binding to `0.0.0.0` and reading `PORT` (defaulting to `8080`):
   ```rust caption="Adapting the server entrypoint for Cloud Run's 0.0.0.0:$PORT contract"
   let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
   let port: u16 = std::env::var("PORT")
       .ok()
       .and_then(|p| p.parse().ok())
       .unwrap_or(8080);
   let addr = format!("{host}:{port}");
   let listener = tokio::net::TcpListener::bind(&addr).await?;
   ```
3. **Deep Repository Seam (`/codebase-design` + `firestore-rules-creation`)**:
   - Rather than deleting local Turso/SQLite and losing fast offline tests, `/codebase-design` guides extracting a repository seam so `cargo test` continues running against in-memory Turso while production runs against Cloud Firestore atomic transactions.
4. **Leaf-Level `gcloud` CLI Validation (`gcloud`)**:
   - Before proposing or running *any* `gcloud` command, the `gcloud` skill requires running `gcloud help <leaf_command>`, scoping `--project=<PROJECT_ID>` and `--region=us-central1`, and applying `--filter` / `--format` data reduction flags.

---

## 3. Interactive Simulator: Repository Seam & Scale-to-Zero Cloud Architecture {#cloud-simulator}

Explore how each layer of the application adapts when moving from local single-node persistence to **Cloud Run + Cloud Firestore** guided by `google/skills`:

{{widget:cloud-seam-simulator}}

---

## 4. Interactive Exercise: Applying `google/skills` Guardrails {#exercise-0005}

Test your recall of the `cloud-run-basics` and `gcloud` skill guardrails before starting your open-ended capstone exercise:

:::exercise id="ex-0005" title="Interactive Lab: Cloud Run & gcloud Skill Guardrails" badge="google/skills • Operational Safety"
Fill in the required bind address, environment variable, scaling flag, and `gcloud` validation step:

```bash caption="Validating Cloud Run container contracts and gcloud CLI syntax"
# 1. CRITICAL RULE from cloud-run-basics: A Cloud Run container MUST listen on:
# Host: {{blank:0.0.0.0|Bind to all interfaces, never 127.0.0.1|0.0.0.0|10}}
# Environment variable for port: ${{blank:PORT|Environment variable injected by Cloud Run (defaults to 8080)|PORT|8}}

# 2. Mandatory Step 1 from the gcloud skill before proposing flags for `gcloud run deploy`:
gcloud {{blank:help|Run leaf-level syntax help before proposing or executing flags|help|8}} run deploy

# 3. Cloud Run flag to ensure scale-to-zero ($0/month idle compute cost):
gcloud run deploy audio-inventory \
  --source . \
  --region=us-central1 \
  --{{blank:min-instances=0||min-instances 0|Flag that allows Cloud Run to scale down to zero instances when idle|min-instances=0|17}} \
  --project="$PROJECT_ID"
```

:::prompts
hint: "Bind to `0.0.0.0` and `$PORT`, validate syntax with `gcloud help run deploy`, and pass `--min-instances=0` for scale-to-zero."
success: "Excellent! Those exact rules from `cloud-run-basics` and `gcloud` prevent the most common Cloud Run boot crashes and CLI hallucinations."
:::
:::

---

## 5. Open-Ended Reader Capstone: Take Your Own Project to the Cloud {#verify-0005}

Now the baton is yours! We deliberately leave the **Production Run** as an open-ended exercise so you can apply it to **your own application** (or to a clone of `experiments/audio_inventory_skilled/`):

:::verify id="verify-0005" title="Capstone Exercise: Plan & Validate a Cloud Migration with google/skills" badge="Your Own Project • mattpocock/skills + google/skills"
In your own project workspace (or a copy of `experiments/audio_inventory_skilled/`), install `google/skills` and use `/wayfinder` to chart a productionization map:

```bash caption="Composing mattpocock/skills and google/skills on your own project"
# 1. Inside your project directory, add the official Google Cloud skills:
npx skills@latest add google/skills

# 2. Launch agy and start a Wayfinder map for your cloud deployment:
agy

# Inside agy:
/wayfinder I'd like to adapt and productionize this application for a low-volume, low-cost deployment on Google Cloud.
```

- [ ] Watch how `agy` combines `/wayfinder` and `/grilling` with `cloud-databases-onboarding` and `google-cloud-waf-cost-optimization` to interview you about your traffic, data model, and budget constraints.
- [ ] Record an Architecture Decision Record (`docs/adr/0002-...md`) capturing your chosen Google Cloud compute and persistence architecture (e.g., **Cloud Run** + **Cloud Firestore** behind a repository seam so local tests remain fast and offline).
- [ ] Ask `agy` to draft a deployment validation plan and verify that it invokes `gcloud help <leaf_command>` (per the `gcloud` skill) and enforces `0.0.0.0:$PORT` (per `cloud-run-basics`).
- [ ] Reflect on the journey from Lesson 0001 to Lesson 0005: notice where fast vibe-coding helped you discover what you wanted, and where durable Wayfinder artifacts (`GLOSSARY.md`, ADRs, `spec.md`, vertical TDD slices) made your codebase ready for change!
:::

---

## 6. Retrieval Practice {#quiz-0005}

:::quiz id="quiz-0005" title="Retrieval Practice: Composing Workflow & Cloud Skills" badge="3 Questions • Equal-Length Options"
- question: "Why does `cloud-run-basics` emphasize listening on `0.0.0.0` and reading `$PORT` instead of `127.0.0.1:3000`?"
  wrong_hint: "Consider how Cloud Run's ingress proxy routes HTTP requests into your container instance."
  options:
    - letter: "A"
      text: "Because containers bound to loopback ignore injected ports and crash on boot."
      correct: true
      explain: "Correct! Cloud Run injects `$PORT` (default 8080) and routes traffic from outside the container's loopback interface, requiring `0.0.0.0:$PORT`."
    - letter: "B"
      text: "Because Rust binaries cannot compile TCP sockets on port three thousand locally."
      correct: false
      explain: "Port 3000 works fine on `127.0.0.1` locally, which is why local prototypes often fail when containerized without `cloud-run-basics`."
    - letter: "C"
      text: "Because Cloud Firestore requires every client to open port eighty for TLS."
      correct: false
      explain: "Firestore clients make outbound HTTPS/gRPC connections; `$PORT` is for incoming HTTP requests to the Cloud Run service."
    - letter: "D"
      text: "Because HTMX scripts refuse to send AJAX requests to local localhost addresses."
      correct: false
      explain: "HTMX uses relative URLs (`/catalog`, `/builds`); the `0.0.0.0:$PORT` requirement is strictly a Cloud Run container contract."

- question: "What is Mandatory Step 1 in the `gcloud` skill before proposing or executing any `gcloud` CLI command?"
  wrong_hint: "Remember how the `gcloud` skill prevents hallucinated flags and stale parametric knowledge."
  options:
    - letter: "A"
      text: "Run an unconstrained gcloud projects list command without any filter flags."
      correct: false
      explain: "Unconstrained `list` commands violate the `gcloud` skill's mandatory Data Reduction rule."
    - letter: "B"
      text: "Search external blog posts using search_web for example gcloud deployment scripts."
      correct: false
      explain: "`gcloud/SKILL.md` explicitly forbids web search fallbacks for CLI syntax."
    - letter: "C"
      text: "Validate leaf-level command syntax by running gcloud help on the subcommand."
      correct: true
      explain: "Correct! Running `gcloud help <leaf_command>` against the installed SDK guarantees accurate flags and checks for `--dry-run` support."
    - letter: "D"
      text: "Enable all Google Cloud billing and IAM APIs with quiet flags."
      correct: false
      explain: "Proactive API enabling and IAM changes are on the Prohibited Operations denylist and require explicit user authorization."

- question: "Which overarching lesson does the three-run Antigravity experiment demonstrate about Vibe Coding vs. Agent Skills?"
  wrong_hint: "Think back to our core mission: is one mode always right and the other always wrong?"
  options:
    - letter: "A"
      text: "Vibe coding is always wrong and should never be used anywhere."
      correct: false
      explain: "Vibe coding is a great tool for fast, low-ceremony exploration when you are still discovering what you want to build."
    - letter: "B"
      text: "Wayfinder should be used for every five-line shell script without exception."
      correct: false
      explain: "Heavy planning ceremony is overkill for tiny throwaway scripts."
    - letter: "C"
      text: "Agent skills eliminate the need for automated tests or browser verification."
      correct: false
      explain: "Skills *enable* disciplined TDD and browser verification—they don't replace verification."
    - letter: "D"
      text: "Vibe coding accelerates early exploration, while skills create maintainable engineered systems."
      correct: true
      explain: "Correct! Use vibe coding when you want to explore potential approaches quickly, and use `/wayfinder` + skills when building for ongoing maintenance."
:::

:::teacher-prompt
**Congratulations on completing the Codelab!** Keep the [Workflow Cheat Sheet](../reference/agent-skills-workflow-cheatsheet.md) and [Google Cloud Skills Reference](../reference/google-cloud-skills-and-serverless-reference.md) handy as you experiment with `agy` on your own projects—and ask me any questions whenever you want to dig deeper into a trace, skill, or architectural pattern!
:::

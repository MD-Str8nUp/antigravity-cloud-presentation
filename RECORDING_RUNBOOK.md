# Operator Recording Runbook: Three-Run Antigravity Cloud Website Experiment

This document lists the step-by-step commands, prompts, and **Canonical Q&A** answers to execute the three-part audio electronics **Component** and **Kit** web system experiment in a clean directory (`~/Demos/audio-inventory-experiment`).

Each run (**Baseline Run**, **Skilled Run**, and **Production Run**) is executed as a **self-contained four-stage cycle** before moving on to the next recording:

1. **Record the run**: Capture the raw `asciinema` recording (`recordings/raw/0{1,2,3}-*.cast`) in `$EXP_DIR`.
2. **Verify the recording**: Sanitize and trim the **Terminal Cast** (`recordings/final/0{1,2,3}-*.cast`), export and inspect the Antigravity **Session Traces** (`artifacts/traces/*`), verify the generated app/tests, and sync the completed run workspace into `experiments/`.
3. **Build the demo page for the recording**: Build/update the standalone 16:9 HTML cast-player demo page (`prototypes/0{1,2,3}-*-cast-player.html`) with the embedded recording (`EMBEDDED_CAST_GZ_B64`) and verify playback in the browser.
4. **Tune the stop points for the recording**: Inspect and refine the `"m"` chapter markers, two-stop Q&A/prompt pairs, and paste/submit delays (`dt`) using `mark-cast.py` and `scripts/markers/0{1,2,3}-*.json`, re-embedding into `prototypes/0{1,2,3}-*-cast-player.html` until stop points and adaptive segment speeds are dialed in, before moving on to the next recording.

---

## Overview of Directory Locations

The three web applications are created in `~/Demos/audio-inventory-experiment` (`$EXP_DIR`) outside this repo so `audio_inventory_baseline` does not inherit this repo's `.agents/skills`, while recordings, standalone HTML cast-player demo pages, synced `experiments/` workspaces, and extracted **Session Traces** are written directly into `$REPO_ROOT`:

```text
~/Demos/audio-inventory-experiment/                        # $EXP_DIR (clean workspace root outside this repo)
├── audio_inventory_baseline/                              # Run 1 (Baseline Run): Agent-chosen local stack (e.g., Python + SQLite), no skills
├── audio_inventory_skilled/                               # Run 2 (Skilled Run): npx skills@latest add mattpocock/skills (Rust/Axum + HTMX + local Turso/libSQL)
└── audio_inventory_production/                            # Run 3 (Production Run): Cloned from Skilled Run + npx skills@latest add google/skills (Cloud Run + Cloud Firestore)

/Users/brett/Documents/GitHub/antigravity-cloud-presentation/   # $REPO_ROOT
├── recordings/
│   ├── raw/                                               # Stage 1: Raw .cast files captured during each run
│   │   ├── 01-baseline.cast
│   │   ├── 02-skilled.cast
│   │   └── 03-production.cast
│   └── final/                                             # Stage 2 & 4: Sanitized, chapter-marked & exit-trimmed .cast files
│       ├── 01-baseline.cast
│       ├── 02-skilled.cast
│       └── 03-production.cast
├── prototypes/                                            # Stage 3 & 4: Standalone 16:9 Slidev cast-full HTML demo pages
│   ├── 01-baseline-cast-player.html
│   ├── 02-skilled-cast-player.html
│   └── 03-production-cast-player.html
├── artifacts/
│   └── traces/                                            # Stage 2: Exported per-session Agy Session Traces
│       ├── audio_inventory_baseline/
│       ├── audio_inventory_skilled/
│       └── audio_inventory_production/
├── scripts/
│   ├── setup-workspaces.sh                                # Cast post-processor, trace extractor, and workspace sync helper
│   └── markers/                                           # Stage 4: Canonical chapter stop-point specs for mark-cast.py
│       ├── 01-baseline.json
│       ├── 02-skilled.json
│       └── 03-production.json
└── experiments/                                           # Stage 2: Completed app workspaces synced per run (excluding target/ and .venv/)
    ├── audio_inventory_baseline/
    ├── audio_inventory_skilled/
    └── audio_inventory_production/
```

---

## Step 0: Pre-Flight Tool & Environment Check

Run these quick checks before starting the recordings to confirm every required CLI tool, language toolchain, and cloud configuration is ready:

### 0.1 Verify Python (`python3` / `uv`) & Rust Toolchain (`rustc` / `cargo`)
Run 1 (**Baseline Run**) typically uses Python + SQLite, and Runs 2 & 3 (**Skilled Run** & **Production Run**) use Rust (`axum`, `tokio`, `libsql` / `turso`). Confirm both toolchains are ready:
```bash
python3 --version
uv --version
rustc --version
cargo --version
cargo clippy --version
```

### 0.2 Verify Google Cloud CLI (`gcloud`), Active Project & Cloud Run / Firestore Readiness
Run 3 (**Production Run**) migrates persistence to **Cloud Firestore** and deploys the containerized Rust + HTMX web server to **Cloud Run** (`--min-instances=0`, `us-central1`). Confirm `gcloud` authentication, Application Default Credentials (ADC), and target project APIs before recording so `gcloud` does not pause for interactive browser login mid-cast:
```bash
gcloud --version
gcloud auth list
gcloud config get-value project
gcloud services list --enabled --filter="name:(run.googleapis.com OR firestore.googleapis.com OR cloudbuild.googleapis.com OR artifactregistry.googleapis.com)"
```
- If any of `run.googleapis.com`, `firestore.googleapis.com`, `cloudbuild.googleapis.com`, or `artifactregistry.googleapis.com` are not yet enabled on your active project, enable them and ensure a `(default)` Firestore Native database exists in `us-central1`:
  ```bash
  gcloud services enable run.googleapis.com firestore.googleapis.com cloudbuild.googleapis.com artifactregistry.googleapis.com
  gcloud firestore databases list
  ```

### 0.3 Verify `asciinema` Version & `--window-size` Flag (`>= 3.0`)
The runbook uses `asciinema 3.x` flags (`--window-size 120x32`). Chapter stop-point markers (`"m"` events) are injected and tuned post-recording by `mark-cast.py`. Verify the installed version and run a 1-second non-interactive smoke test:
```bash
asciinema --version
asciinema rec --window-size 120x32 --idle-time-limit 2.0 --overwrite -c "echo 'asciinema pre-flight OK'" /tmp/preflight.cast && rm -f /tmp/preflight.cast
```

### 0.4 Pre-Warm `npx skills@latest`
Run 2 (`mattpocock/skills`) and Run 3 (`google/skills`) invoke `npx skills@latest add ...` live on camera. Pre-warm the `skills` package cache so `npx` doesn't prompt to install `skills@latest` during the recording:
```bash
npx --yes skills@latest --version
```

### 0.5 Verify Antigravity CLI (`agy`) & Your Global Rule
Confirm `agy` is on `PATH` and your **Global Rule** (`When something is unclear, please ask clarifying questions, one at a time.`) is present in `$HOME/.gemini/config/AGENTS.md` so Run 1 (**Baseline Run**) exhibits the expected one-at-a-time questioning behavior:
```bash
which agy
agy --version
cat "$HOME/.gemini/config/AGENTS.md"
```

---

## Step 1: Set Environment Variables & Create Directories

Run the following in your terminal before starting the recordings:

```bash
export REPO_ROOT="/Users/brett/Documents/GitHub/antigravity-cloud-presentation"
export EXP_DIR="$HOME/Demos/audio-inventory-experiment"
mkdir -p "$EXP_DIR" "$REPO_ROOT/recordings/raw" "$REPO_ROOT/recordings/final" "$REPO_ROOT/artifacts/traces" "$REPO_ROOT/experiments"
cd "$EXP_DIR"
```

---

## Canonical Q&A Answers (Used Across Runs 1, 2 & 3)

Keep these answers handy so the **Baseline Run**, **Skilled Run**, and **Production Run** receive consistent domain requirements when `agy` asks clarifying questions:

| Topic | Canonical Answer to Copy/Paste |
| :--- | :--- |
| **Components & Categories** *(Runs 1 & 2)* | `Audio electronics parts across five categories: ICs/Op-Amps (e.g., TL074, NE5532, LM13700), Resistors, Capacitors, Potentiometers/Jacks, and Diodes/LEDs.` |
| **Attributes per Component** *(Runs 1 & 2)* | `Part name/value (e.g., "TL074", "10kΩ Metal Film"), category, package mounting style (SMT or THT with footprint like SOIC-14, DIP-14, 0805), quantity on hand, storage location (drawer/bin label), and freeform electrical specs/notes.` |
| **Kits & BOM Tracking** *(Runs 1 & 2)* | `DIY audio synth/pedal PCB builds (e.g., "VCA Module", "Overdrive Pedal") with a build stage (Planned, In Progress, Completed) and a Bill of Materials (BOM) listing required Components and quantities. Show whether enough stock is on hand for each Kit (highlighting shortfalls) and allow allocating/deducting BOM stock when building a Kit.` |
| **Core Workflows** *(Runs 1 & 2)* | `Browse, search, and filter Components by category or mounting style (SMT/THT), quickly increment/decrement stock, create/edit Components and Kits, view Kit build readiness/missing parts, and deduct stock when a Kit is built. Single-user app: no user auth, barcode scanning, or supplier API integration.` |
| **Tech Stack & Persistence — Run 1 (Baseline Run)** | `Use whatever simple local web stack and local database you prefer (e.g., Python with SQLite), pre-populated with sample audio components (including SMT and THT parts like TL074) and two sample kits on first launch.` |
| **Tech Stack & Persistence — Run 2 (Skilled Run)** | `Rust backend (Axum), server-rendered HTML with HTMX frontend, and a local Turso (libSQL) database behind a clean repository trait seam, pre-populated with sample audio components (including SMT and THT parts like TL074) and two sample kits on first launch.` |
| **UI Style** *(Runs 1 & 2)* | `Clean, high-contrast workbench web UI with fast table filtering, inline HTMX/stock updates, and clear SMT/THT and Kit readiness badges.` |
| **Issue Tracker** *(Runs 2 & 3)* | `Local Markdown (.scratch/)` |
| **Domain Modeling** *(Runs 2 & 3)* | `Single context (GLOSSARY.md + docs/adr/)` |
| **Cloud Target & Cost Constraint — Run 3 (Production Run)** | `Migrate persistence from local Turso/libSQL to Cloud Firestore using the existing repository trait seam (keeping local/in-memory support for tests), containerize the Rust + HTMX server, and deploy to Cloud Run (--min-instances=0, --allow-unauthenticated) in us-central1 so idle cost is $0/month within the free tier.` |

---

## Step 2: Run 1 — Baseline Run (`01-baseline.cast`, `audio_inventory_baseline`)

Complete all four stages for the **Baseline Run** before starting the **Skilled Run**.

### 2.1 Stage 1 — Record the Baseline Run (`recordings/raw/01-baseline.cast`)

1. **Start `asciinema` in `$EXP_DIR`**:
   ```bash
   cd "$EXP_DIR"
   asciinema rec --window-size 120x32 --idle-time-limit 2.0 --overwrite "$REPO_ROOT/recordings/raw/01-baseline.cast"
   ```
2. **Create and enter the project directory** inside the recorded shell:
   ```bash
   mkdir audio_inventory_baseline && cd audio_inventory_baseline
   ```
3. **Start Antigravity in the empty directory** *(no repository skills installed)*:
   ```bash
   agy
   ```
4. **Paste the initial prompt** *(no mention of Google Cloud)*:
   ```text
   I'd like to build a web application to track my audio electronics components inventory and DIY kit builds.
   ```
5. Answer `agy`'s one-at-a-time clarifying questions using the **Canonical Q&A Answers** table above (letting `agy` pick its default simple local stack such as Python + SQLite).
6. Let `agy` build the local web application, seed the sample `TL074` / audio **Components** and **Kits**, and verify tests / local server startup.
7. **Ask `agy` to run or verify the app locally** (if not already verified):
   ```text
   Please run the app and verify it works.
   ```
8. **Start a new `agy` session**:
   ```text
   /clear
   ```
9. **Review the codebase**:
   ```text
   Please review the codebase
   ```
10. **Ask `agy` to fix the issues surfaced by the review**:
    ```text
    please fix
    ```
11. Exit `agy` and press `Ctrl+D` (or type `exit`) to finish the `asciinema` recording.

### 2.2 Stage 2 — Verify the Baseline Run Recording, Traces & App

Immediately after exiting the recording shell, run `process-run` to sanitize/trim `01-baseline.cast` into `recordings/final/01-baseline.cast`, inject initial stop points, update `prototypes/01-baseline-cast-player.html`, export the Antigravity **Session Traces** into `artifacts/traces/audio_inventory_baseline/`, and sync `audio_inventory_baseline` into `experiments/audio_inventory_baseline/`:

```bash
"$REPO_ROOT/scripts/setup-workspaces.sh" process-run 01-baseline.cast audio_inventory_baseline
```

1. **Verify the sanitized Terminal Cast & exit trim**:
   ```bash
   python3 "$REPO_ROOT/.agents/skills/asciinema/scripts/trim-cast.py" "$REPO_ROOT/recordings/final/01-baseline.cast" --info
   asciinema play -m -s 2.0 -i 1.0 "$REPO_ROOT/recordings/final/01-baseline.cast"
   ```
2. **Verify the exported Antigravity Session Traces**:
   ```bash
   cat "$REPO_ROOT/artifacts/traces/audio_inventory_baseline/README.md"
   ```
   - Confirm `agy` asked its clarifying questions one at a time (per your Global Rule), built and verified the local web app in Session 1, reviewed and fixed the codebase in Session 2 after `/clear`, and did **not** produce architecture artifacts (`GLOSSARY.md` or `docs/adr/`).
3. **Verify the synced workspace in `experiments/audio_inventory_baseline/`**:
   ```bash
   ls -la "$REPO_ROOT/experiments/audio_inventory_baseline"
   ```

> **If you need to REDO Run 1**: Clean up the project directories **and** the aborted `audio_inventory_baseline` Agy brain sessions, then repeat from **Step 2.1**:
> ```bash
> uv run "$REPO_ROOT/.agents/skills/agy-brain-explorer/scripts/agy-brain-explorer.py" delete --project "audio_inventory_baseline" --force
> rm -rf "$EXP_DIR/audio_inventory_baseline" "$REPO_ROOT/experiments/audio_inventory_baseline" "$REPO_ROOT/artifacts/traces/audio_inventory_baseline" "$REPO_ROOT/recordings/raw/01-baseline.cast" "$REPO_ROOT/recordings/final/01-baseline.cast"
> ```

### 2.3 Stage 3 — Build & Verify the Demo Page (`prototypes/01-baseline-cast-player.html`)

Embed `recordings/final/01-baseline.cast` (`EMBEDDED_CAST_GZ_B64`) into the standalone 16:9 Slidev `cast-full` HTML player and open it in Chrome to verify playback:

```bash
python3 "$REPO_ROOT/.agents/skills/asciinema/scripts/mark-cast.py" \
  "$REPO_ROOT/recordings/final/01-baseline.cast" \
  --list \
  --embed-html "$REPO_ROOT/prototypes/01-baseline-cast-player.html"

open "$REPO_ROOT/prototypes/01-baseline-cast-player.html"
```

- Confirm the 16:9 stage loads over `file://` with no CORS errors, chapter stepper pills and adaptive segment speed badges (`>= 2x`, `<= 30s` per segment) render cleanly, and `[` / `]` / `p` keyboard controls step across every stop point through `→ End`.

### 2.4 Stage 4 — Tune the Stop Points & Paste Delays for the Baseline Run

Audit every stop point in `prototypes/01-baseline-cast-player.html` and refine `recordings/final/01-baseline.cast` and `scripts/markers/01-baseline.json` until every pause lands on the exact target frame:

1. **Inspect current markers and search stdout events**:
   ```bash
   MARK_SCRIPT="$REPO_ROOT/.agents/skills/asciinema/scripts/mark-cast.py"

   # List existing stop points, event indices, 1x times, and adaptive speeds
   python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" --list

   # Search for shell commands, clarifying questions, typed answers, and /clear
   python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" --search "audio_inventory_baseline" --context 4
   python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" --inspect 0:30
   ```
2. **Apply the stop-point placement rules (`cast-player-html`)**:
   - **Shell command / `agy` entry**: Pause immediately after `[UNREV]` (reverse-video paste cleared, cursor still on the command line before output/TUI startup).
   - **Two-stop Q&A and prompt pairs**: Every clarifying question or typed prompt gets **two paired stop points**:
     1. *Before typing (`Q: ...` / `Ready for ...`)*: Right after the question or prompt footer finishes rendering (`? for shortcuts` or `enter Submit`), before the first keystroke/paste of the answer.
     2. *After typing, before `Enter` (`Answer: ...` / `/command`)*: Right after the complete typed/pasted answer or slash command is echoed in the input box, before the `Enter` clear/spinner event.
   - **`/clear` boundary**: One stop point with `/clear` visible in the input box (before `Enter`) and a second stop point (`Session Cleared`) right after the fresh `Antigravity CLI` banner redraws.
3. **Update `scripts/markers/01-baseline.json`, re-apply markers, tighten paste delays (`dt`), and re-embed into `prototypes/01-baseline-cast-player.html`**:
   ```bash
   python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" \
     --clear --auto-spec "$REPO_ROOT/scripts/markers/01-baseline.json" \
     --tighten-pastes --in-place \
     --embed-html "$REPO_ROOT/prototypes/01-baseline-cast-player.html"
   ```
   *(Use `--set <N> "<pattern_or_@event:IDX>" "<label>"`, `--relabel ...`, or `--set-dt <idx> <sec>` for fine-grained event adjustments, then refresh `prototypes/01-baseline-cast-player.html` in the browser to confirm.)*

---

## Step 3: Run 2 — Skilled Run (`02-skilled.cast`, `audio_inventory_skilled`)

Complete all four stages for the **Skilled Run** before starting the **Production Run**.

### 3.1 Stage 1 — Record the Skilled Run (`recordings/raw/02-skilled.cast`)

1. **Start `asciinema` in `$EXP_DIR`**:
   ```bash
   cd "$EXP_DIR"
   asciinema rec --window-size 120x32 --idle-time-limit 2.0 --overwrite "$REPO_ROOT/recordings/raw/02-skilled.cast"
   ```
2. **Create and enter the project directory** inside the recorded shell:
   ```bash
   mkdir audio_inventory_skilled && cd audio_inventory_skilled
   ```
3. **Install Matt Pocock's engineering workflow skills**:
   ```bash
   npx skills@latest add mattpocock/skills
   ```
4. **Start Antigravity in `audio_inventory_skilled`**:
   ```bash
   agy
   ```
5. **Prompt Antigravity with `/wayfinder`** *(same prompt as Run 1, no mention of Google Cloud)*:
   ```text
   /wayfinder I'd like to build a web application to track my audio electronics components inventory and DIY kit builds.
   ```
6. **Answer questions** using the **Canonical Q&A Answers** table above (specifying **Rust backend (Axum), HTMX frontend, and local Turso (`libSQL`) database behind a clean repository trait seam**, `Local Markdown (.scratch/)`, and `Single context (GLOSSARY.md + docs/adr/)`) until the map and initial decision tickets are created.
7. **Work through the `/wayfinder` decision tickets** (`/clear` → `/wayfinder` → answer questions → `/clear`) until all `/wayfinder` decision tickets are resolved.
8. **Complete the specification and build of the app** (`/to-spec` → `/to-tickets` → `/implement`, with `/clear` between stages as appropriate) until the Rust/Axum + HTMX + local Turso (`libSQL`) web application is built and `cargo test` passes.
9. **Ask `agy` to run and verify the app**:
   ```text
   Please run the app and verify it works.
   ```
10. **Start a new `agy` session**:
    ```text
    /clear
    ```
11. **Review the codebase**:
    ```text
    Please review the codebase
    ```
12. **Ask `agy` to fix the issues surfaced by the review**:
    ```text
    please fix
    ```
13. Exit `agy` and press `Ctrl+D` (or `exit`) to save `02-skilled.cast`.

### 3.2 Stage 2 — Verify the Skilled Run Recording, Traces & App

Immediately after exiting the recording shell, run `process-run` to sanitize/trim `02-skilled.cast` into `recordings/final/02-skilled.cast`, inject initial stop points, update `prototypes/02-skilled-cast-player.html`, export all `/clear`-separated Antigravity **Session Traces** into `artifacts/traces/audio_inventory_skilled/`, and sync `audio_inventory_skilled` into `experiments/audio_inventory_skilled/`:

```bash
"$REPO_ROOT/scripts/setup-workspaces.sh" process-run 02-skilled.cast audio_inventory_skilled
```

1. **Verify the sanitized Terminal Cast & exit trim**:
   ```bash
   python3 "$REPO_ROOT/.agents/skills/asciinema/scripts/trim-cast.py" "$REPO_ROOT/recordings/final/02-skilled.cast" --info
   asciinema play -m -s 2.0 -i 1.0 "$REPO_ROOT/recordings/final/02-skilled.cast"
   ```
2. **Verify the exported Antigravity Session Traces**:
   ```bash
   cat "$REPO_ROOT/artifacts/traces/audio_inventory_skilled/README.md"
   ```
   - Confirm each `/clear`-separated session (`/wayfinder`, `/to-spec`, `/to-tickets`, `/implement`, and codebase review) is exported cleanly in chronological order.
3. **Verify the Skilled Run domain artifacts, Rust tests & synced `experiments/` workspace**:
   ```bash
   cd "$EXP_DIR/audio_inventory_skilled"
   ls -la GLOSSARY.md docs/adr/ .scratch/
   cargo test
   ls -la "$REPO_ROOT/experiments/audio_inventory_skilled"
   ```

> **If you need to REDO Run 2**: Clean up the project directories **and** the aborted `audio_inventory_skilled` Agy brain sessions, then repeat from **Step 3.1**:
> ```bash
> uv run "$REPO_ROOT/.agents/skills/agy-brain-explorer/scripts/agy-brain-explorer.py" delete --project "audio_inventory_skilled" --force
> rm -rf "$EXP_DIR/audio_inventory_skilled" "$REPO_ROOT/experiments/audio_inventory_skilled" "$REPO_ROOT/artifacts/traces/audio_inventory_skilled" "$REPO_ROOT/recordings/raw/02-skilled.cast" "$REPO_ROOT/recordings/final/02-skilled.cast"
> ```

### 3.3 Stage 3 — Build & Verify the Demo Page (`prototypes/02-skilled-cast-player.html`)

Embed `recordings/final/02-skilled.cast` (`EMBEDDED_CAST_GZ_B64`) into the standalone 16:9 Slidev `cast-full` HTML player and open it in Chrome to verify playback:

```bash
python3 "$REPO_ROOT/.agents/skills/asciinema/scripts/mark-cast.py" \
  "$REPO_ROOT/recordings/final/02-skilled.cast" \
  --list \
  --embed-html "$REPO_ROOT/prototypes/02-skilled-cast-player.html"

open "$REPO_ROOT/prototypes/02-skilled-cast-player.html"
```

- Confirm `prototypes/02-skilled-cast-player.html` loads cleanly over `file://`, steps through all `/wayfinder`, `/to-spec`, `/to-tickets`, `/implement`, and review chapters with `[` / `]` / `p`, and displays adaptive segment speeds (`>= 2x`, `<= 30s`).

### 3.4 Stage 4 — Tune the Stop Points & Paste Delays for the Skilled Run

Audit every stop point in `prototypes/02-skilled-cast-player.html` and refine `recordings/final/02-skilled.cast` and `scripts/markers/02-skilled.json` until every pause lands cleanly:

1. **Inspect current markers and search stdout events**:
   ```bash
   MARK_SCRIPT="$REPO_ROOT/.agents/skills/asciinema/scripts/mark-cast.py"

   python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/02-skilled.cast" --list
   python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/02-skilled.cast" --search "/wayfinder|/to-spec|/to-tickets|/implement|/clear" --context 4
   ```
2. **Audit two-stop prompt/question pairs and `/clear` boundaries**:
   - Confirm `mkdir`, `npx skills@latest add mattpocock/skills`, and `agy` pause on `[UNREV]` command entry.
   - Confirm each `/wayfinder` grilling question, slash command (`/wayfinder`, `/to-spec`, `/to-tickets`, `/implement`), and review prompt uses the two-stop pattern (Stop 1: before typing; Stop 2: after typing, before `Enter`), and each `/clear` has its paired `Session Cleared` stop point.
3. **Update `scripts/markers/02-skilled.json`, re-apply markers, tighten paste delays (`dt`), and re-embed into `prototypes/02-skilled-cast-player.html`**:
   ```bash
   python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/02-skilled.cast" \
     --clear --auto-spec "$REPO_ROOT/scripts/markers/02-skilled.json" \
     --tighten-pastes --in-place \
     --embed-html "$REPO_ROOT/prototypes/02-skilled-cast-player.html"
   ```

---

## Step 4: Run 3 — Production Run (`03-production.cast`, `audio_inventory_production`)

Complete all four stages for the **Production Run**.

### 4.1 Stage 1 — Clone Skilled Run & Record the Production Run (`recordings/raw/03-production.cast`)

1. **Clone the finished Skilled Run workspace to `audio_inventory_production`**:
   ```bash
   cd "$EXP_DIR"
   rm -rf "$EXP_DIR/audio_inventory_production"
   cp -R "$EXP_DIR/audio_inventory_skilled" "$EXP_DIR/audio_inventory_production"
   ```
2. **Start `asciinema` inside `audio_inventory_production`**:
   ```bash
   cd "$EXP_DIR/audio_inventory_production"
   asciinema rec --window-size 120x32 --idle-time-limit 2.0 --overwrite "$REPO_ROOT/recordings/raw/03-production.cast"
   ```
3. **Install Google Cloud skills live in the recording**:
   ```bash
   npx skills@latest add google/skills
   ```
4. **Launch `agy`**:
   ```bash
   agy
   ```
5. **Prompt Antigravity with `/wayfinder` to plan the Google Cloud migration**:
   ```text
   /wayfinder I'd like to adapt and productionize this web application for a low-volume, low-cost deployment on Google Cloud using Cloud Run and Cloud Firestore.
   ```
6. **Answer clarifying questions** using the **Cloud Target & Cost Constraint — Run 3 (Production Run)** entry in the **Canonical Q&A Answers** table above:
   - Migrate persistence from local Turso/libSQL to **Cloud Firestore** behind the existing repository trait seam (keeping local/in-memory support for tests).
   - Containerize the Rust + HTMX server (`Dockerfile`).
   - Deploy to **Cloud Run** (`--min-instances=0`, `--allow-unauthenticated`, `us-central1`) so idle cost is `$0/month` within the free tier (avoiding always-on Cloud SQL VM costs).
7. **Work through the `/wayfinder` migration tickets** (`/clear` → `/wayfinder` → answer questions → `/clear`) until the cloud migration map is resolved.
8. **Generate the migration spec, slice into tickets, and implement & deploy** (`/to-spec` → `/to-tickets` → `/implement`, with `/clear` between stages as appropriate) until:
   - `cargo test` passes against the repository seam.
   - The container builds and deploys live to **Cloud Run**.
   - The live `.run.app` URL responds with the seeded audio electronics **Component** and **Kit** workbench UI backed by **Cloud Firestore**.
9. Exit `agy` and press `Ctrl+D` (or `exit`) to save `03-production.cast`.

### 4.2 Stage 2 — Verify the Production Run Recording, Traces & Live Cloud Run Service

Immediately after exiting the recording shell, run `process-run` to sanitize/trim `03-production.cast` into `recordings/final/03-production.cast`, inject initial stop points, update `prototypes/03-production-cast-player.html`, export the Antigravity **Session Traces** into `artifacts/traces/audio_inventory_production/`, and sync `audio_inventory_production` into `experiments/audio_inventory_production/`:

```bash
"$REPO_ROOT/scripts/setup-workspaces.sh" process-run 03-production.cast audio_inventory_production
```

1. **Verify the sanitized Terminal Cast & exit trim**:
   ```bash
   python3 "$REPO_ROOT/.agents/skills/asciinema/scripts/trim-cast.py" "$REPO_ROOT/recordings/final/03-production.cast" --info
   asciinema play -m -s 2.0 -i 1.0 "$REPO_ROOT/recordings/final/03-production.cast"
   ```
2. **Verify the exported Antigravity Session Traces**:
   ```bash
   cat "$REPO_ROOT/artifacts/traces/audio_inventory_production/README.md"
   ```
3. **Verify the Production Run tests, live Cloud Run service & synced `experiments/` workspace**:
   ```bash
   cd "$EXP_DIR/audio_inventory_production"
   cargo test
   gcloud run services list --region=us-central1
   ls -la "$REPO_ROOT/experiments/audio_inventory_production"
   ```

> **If you need to REDO Run 3**: Clean up the `audio_inventory_production` directories and its Agy brain sessions (leaving `audio_inventory_skilled` untouched), then repeat from **Step 4.1**:
> ```bash
> uv run "$REPO_ROOT/.agents/skills/agy-brain-explorer/scripts/agy-brain-explorer.py" delete --project "audio_inventory_production" --force
> rm -rf "$EXP_DIR/audio_inventory_production" "$REPO_ROOT/experiments/audio_inventory_production" "$REPO_ROOT/artifacts/traces/audio_inventory_production" "$REPO_ROOT/recordings/raw/03-production.cast" "$REPO_ROOT/recordings/final/03-production.cast"
> ```

### 4.3 Stage 3 — Build & Verify the Demo Page (`prototypes/03-production-cast-player.html`)

Embed `recordings/final/03-production.cast` (`EMBEDDED_CAST_GZ_B64`) into the standalone 16:9 Slidev `cast-full` HTML player and open it in Chrome to verify playback:

```bash
python3 "$REPO_ROOT/.agents/skills/asciinema/scripts/mark-cast.py" \
  "$REPO_ROOT/recordings/final/03-production.cast" \
  --list \
  --embed-html "$REPO_ROOT/prototypes/03-production-cast-player.html"

open "$REPO_ROOT/prototypes/03-production-cast-player.html"
```

- Confirm `prototypes/03-production-cast-player.html` loads cleanly over `file://`, steps through the Google Cloud skills installation, `/wayfinder` cloud migration planning, Firestore/Cloud Run implementation, and live deployment verification, and displays adaptive segment speeds (`>= 2x`, `<= 30s`).

### 4.4 Stage 4 — Tune the Stop Points & Paste Delays for the Production Run

Audit every stop point in `prototypes/03-production-cast-player.html` and refine `recordings/final/03-production.cast` and `scripts/markers/03-production.json` until every pause lands cleanly:

1. **Inspect current markers and search stdout events**:
   ```bash
   MARK_SCRIPT="$REPO_ROOT/.agents/skills/asciinema/scripts/mark-cast.py"

   python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/03-production.cast" --list
   python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/03-production.cast" --search "google/skills|/wayfinder|/to-spec|/to-tickets|/implement|run.app" --context 4
   ```
2. **Audit two-stop prompt/question pairs, `/clear` boundaries, and deployment verification frames**:
   - Confirm `npx skills@latest add google/skills` and `agy` pause on `[UNREV]` command entry.
   - Confirm each `/wayfinder` cloud migration question, slash command, and `/clear` transition uses the two-stop pattern (Stop 1: before typing; Stop 2: after typing, before `Enter`).
   - Confirm the final deployment stop point lands right after the live `.run.app` verification output finishes rendering.
3. **Update `scripts/markers/03-production.json`, re-apply markers, tighten paste delays (`dt`), and re-embed into `prototypes/03-production-cast-player.html`**:
   ```bash
   python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/03-production.cast" \
     --clear --auto-spec "$REPO_ROOT/scripts/markers/03-production.json" \
     --tighten-pastes --in-place \
     --embed-html "$REPO_ROOT/prototypes/03-production-cast-player.html"
   ```

---

## Step 5: Final Status Check Across All Three Runs

Once all three runs have completed their four-stage cycles (record, verify, demo page, and stop-point tuning), run `status` to confirm all workspaces, `experiments/` snapshots, `recordings/final/*.cast` files, `artifacts/traces/*` **Session Traces**, and `prototypes/*-cast-player.html` demo pages are ready for **Codelab** and Slidev deck authoring:

```bash
"$REPO_ROOT/scripts/setup-workspaces.sh" status
```

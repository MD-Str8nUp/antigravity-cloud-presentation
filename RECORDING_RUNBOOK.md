# Operator Recording Runbook: Three-Run Antigravity Cloud Website Experiment

This document lists the step-by-step commands, prompts, and **Canonical Q&A** answers to run the three-part audio electronics **Component** and **Kit** web system experiment in a clean directory (`~/Demos/audio-inventory-experiment`), record and verify each `asciinema` **Terminal Cast** and its Antigravity **Session Traces** after each run, and sync the finished app workspaces into this repository.

---

## Overview of Directory Locations

The three web applications are created in `~/Demos/audio-inventory-experiment` (`$EXP_DIR`) outside this repo so `audio_inventory_baseline` does not inherit this repo's `.agents/skills`, while recordings, standalone HTML cast players, and extracted **Session Traces** are written directly into `$REPO_ROOT`:

```text
~/Demos/audio-inventory-experiment/                        # $EXP_DIR (clean workspace root outside this repo)
├── audio_inventory_baseline/                              # Run 1 (Baseline Run): Agent-chosen local stack (e.g., Python + SQLite), no skills
├── audio_inventory_skilled/                               # Run 2 (Skilled Run): npx skills@latest add mattpocock/skills (Rust/Axum + HTMX + local Turso/libSQL)
└── audio_inventory_production/                            # Run 3 (Production Run): Cloned from Skilled Run + npx skills@latest add google/skills (Cloud Run + Cloud Firestore)

/Users/brett/Documents/GitHub/antigravity-cloud-presentation/   # $REPO_ROOT
├── recordings/
│   ├── raw/                                               # Raw .cast files captured during each run
│   │   ├── 01-baseline.cast
│   │   ├── 02-skilled.cast
│   │   └── 03-production.cast
│   └── final/                                             # Sanitized, chapter-marked & exit-trimmed .cast files verified after each run
│       ├── 01-baseline.cast
│       ├── 02-skilled.cast
│       └── 03-production.cast
├── prototypes/                                            # Standalone 16:9 Slidev cast-full HTML players synced by process-run
│   ├── 01-baseline-cast-player.html
│   ├── 02-skilled-cast-player.html
│   └── 03-production-cast-player.html
├── artifacts/
│   └── traces/                                            # Exported per-session Agy Session Traces verified after each run
│       ├── audio_inventory_baseline/
│       ├── audio_inventory_skilled/
│       └── audio_inventory_production/
├── scripts/
│   ├── setup-workspaces.sh                                # Cast post-processor, trace extractor, and workspace sync helper
│   └── markers/                                           # Canonical chapter stop-point specs for mark-cast.py
│       ├── 01-baseline.json
│       ├── 02-skilled.json
│       └── 03-production.json
└── experiments/                                           # Completed app workspaces synced in Step 5 (excluding target/ and .venv/)
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
The runbook uses `asciinema 3.x` flags (`--window-size 120x32`). Chapter stop-point markers (`"m"` events) are injected automatically post-recording by `mark-cast.py`. Verify the installed version and run a 1-second non-interactive smoke test:
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
mkdir -p "$EXP_DIR" "$REPO_ROOT/recordings/raw" "$REPO_ROOT/recordings/final" "$REPO_ROOT/artifacts/traces"
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

## Step 2: Record Run 1 — Baseline Run (`01-baseline.cast`)

### 2.1 Start Recording in `$EXP_DIR`
```bash
cd "$EXP_DIR"
asciinema rec --window-size 120x32 --idle-time-limit 2.0 --overwrite "$REPO_ROOT/recordings/raw/01-baseline.cast"
```

### 2.2 Create `audio_inventory_baseline` & Launch `agy`
Inside the recorded shell:

1. **Create and enter the project directory**:
   ```bash
   mkdir audio_inventory_baseline && cd audio_inventory_baseline
   ```
2. **Start Antigravity in the empty directory**:
   ```bash
   agy
   ```
3. **Paste the initial prompt** *(no mention of Google Cloud)*:
   ```text
   I'd like to build a web application to track my audio electronics components inventory and DIY kit builds.
   ```
4. Answer `agy`'s one-at-a-time clarifying questions using the **Canonical Q&A Answers** table above (letting `agy` pick its default simple local stack such as Python + SQLite).
5. Let `agy` build the local web application, seed the sample `TL074` / audio **Components** and **Kits**, and verify tests / local server startup.
6. **Ask `agy` to run or verify the app locally** (if not already verified):
   ```text
   Please run the app and verify it works.
   ```
7. **Start a new `agy` session**:
   ```text
   /clear
   ```
8. **Review the codebase**:
   ```text
   Please review the codebase
   ```
9. **Ask `agy` to fix the issues surfaced by the review**:
   ```text
   please fix
   ```
10. Exit `agy` and press `Ctrl+D` (or type `exit`) to finish the `asciinema` recording.

### 2.3 Process, Inspect & Decide: Good Capture or Redo Run 1?
Immediately after exiting the recording shell, run `process-run` to sanitize/trim `01-baseline.cast`, auto-inject chapter stop-point markers via `mark-cast.py`, sync `prototypes/01-baseline-cast-player.html`, and export the Antigravity **Session Traces** for `audio_inventory_baseline`:

```bash
"$REPO_ROOT/scripts/setup-workspaces.sh" process-run 01-baseline.cast audio_inventory_baseline
```

**Verify the capture:**
1. **Check the Terminal Cast & auto-injected stop points**:
   - Confirm `recordings/final/01-baseline.cast` has non-zero stdout events, chapter markers, and expected playback duration.
   - Quick visual replay in marker-pause mode at 2x speed (`Space` to resume after each stop point, `]` to jump to next marker), or open `prototypes/01-baseline-cast-player.html` in Chrome:
     ```bash
     asciinema play -m -s 2.0 -i 1.0 "$REPO_ROOT/recordings/final/01-baseline.cast"
     ```
2. **Check the exported Agy Session Traces**:
   - Inspect the multi-session summary (Session 1: build & run; Session 2: codebase review) and exported Markdown traces:
     ```bash
     cat "$REPO_ROOT/artifacts/traces/audio_inventory_baseline/README.md"
     ```
   - Confirm `agy` asked its clarifying questions one at a time (per your Global Rule), built and verified the local web app, reviewed the codebase after `/clear`, and did **not** produce architecture artifacts (`GLOSSARY.md` or `docs/adr/`).

> **If you need to REDO Run 1**: Clean up both the project directory **and** the aborted `audio_inventory_baseline` Agy brain sessions (so failed sessions don't pollute your exported logs), then repeat from **Step 2.1**:
> ```bash
> uv run "$REPO_ROOT/.agents/skills/agy-brain-explorer/scripts/agy-brain-explorer.py" delete --project "audio_inventory_baseline" --force
> rm -rf "$EXP_DIR/audio_inventory_baseline" "$REPO_ROOT/artifacts/traces/audio_inventory_baseline" "$REPO_ROOT/recordings/raw/01-baseline.cast" "$REPO_ROOT/recordings/final/01-baseline.cast"
> ```

---

## Step 3: Record Run 2 — Skilled Run (`02-skilled.cast`)

### 3.1 Start Recording in `$EXP_DIR`
```bash
cd "$EXP_DIR"
asciinema rec --window-size 120x32 --idle-time-limit 2.0 --overwrite "$REPO_ROOT/recordings/raw/02-skilled.cast"
```

### 3.2 Execute the Skilled Run Sequence Inside the Recording

1. **Create and enter the project directory**:
   ```bash
   mkdir audio_inventory_skilled && cd audio_inventory_skilled
   ```
2. **Install Matt Pocock's engineering workflow skills**:
   ```bash
   npx skills@latest add mattpocock/skills
   ```
3. **Start Antigravity in `audio_inventory_skilled`**:
   ```bash
   agy
   ```
4. **Prompt Antigravity with `/wayfinder`** *(same prompt as Run 1, no mention of Google Cloud)*:
   ```text
   /wayfinder I'd like to build a web application to track my audio electronics components inventory and DIY kit builds.
   ```
5. **Answer questions** using the **Canonical Q&A Answers** table above (specifying **Rust backend (Axum), HTMX frontend, and local Turso (`libSQL`) database behind a clean repository trait seam**, `Local Markdown (.scratch/)`, and `Single context (GLOSSARY.md + docs/adr/)`) until the map and initial decision tickets are created.
6. **Issue `/clear` in `agy`** to start a new conversation:
   ```text
   /clear
   ```
7. **Issue `/wayfinder`** to start the next round of questions on the next open decision ticket:
   ```text
   /wayfinder
   ```
8. **Answer questions** to resolve the decision ticket.
9. **Issue `/clear`** to start a new conversation:
   ```text
   /clear
   ```
10. **Repeat steps 7 through 9** (`/wayfinder` → answer questions → `/clear`) until all `/wayfinder` decision tickets are complete.
11. **Complete the specification and build of the app** (`/to-spec` → `/to-tickets` → `/implement`, with `/clear` between stages as appropriate) until the Rust/Axum + HTMX + local Turso (`libSQL`) web application is built and `cargo test` passes.
12. **Ask `agy` to run and verify the app**:
    ```text
    Please run the app and verify it works.
    ```
13. **Start a new `agy` session**:
    ```text
    /clear
    ```
14. **Review the codebase**:
    ```text
    Please review the codebase
    ```
15. **Ask `agy` to fix the issues surfaced by the review**:
    ```text
    please fix
    ```
16. Exit `agy` and press `Ctrl+D` (or `exit`) to save `02-skilled.cast`.

### 3.3 Process, Inspect & Decide: Good Capture or Redo Run 2?
Immediately after exiting the recording shell, run `process-run` to sanitize/trim `02-skilled.cast`, auto-inject chapter markers via `mark-cast.py`, sync `prototypes/02-skilled-cast-player.html`, and export all `/clear`-separated Antigravity **Session Traces** for `audio_inventory_skilled`:

```bash
"$REPO_ROOT/scripts/setup-workspaces.sh" process-run 02-skilled.cast audio_inventory_skilled
```

**Verify the capture:**
1. **Check the Terminal Cast & auto-injected stop points**:
   - Confirm the output lists your `"m"` markers at the expected timestamps.
   - Replay in marker-pause mode (`-m`, press `Space` or `]` to jump between markers), or open `prototypes/02-skilled-cast-player.html`:
     ```bash
     asciinema play -m -s 2.0 -i 1.0 "$REPO_ROOT/recordings/final/02-skilled.cast"
     ```
2. **Check the exported Agy Session Traces**:
   - Review the multi-session index (each `/clear` creates a distinct conversation trace in `artifacts/traces/audio_inventory_skilled/`):
     ```bash
     cat "$REPO_ROOT/artifacts/traces/audio_inventory_skilled/README.md"
     ```
3. **Check the Skilled Run domain artifacts & Rust tests**:
   ```bash
   cd "$EXP_DIR/audio_inventory_skilled"
   ls -la GLOSSARY.md docs/adr/ .scratch/
   cargo test
   ```

> **If you need to REDO Run 2**: Clean up both the project directory **and** the aborted `audio_inventory_skilled` Agy brain sessions, then repeat from **Step 3.1**:
> ```bash
> uv run "$REPO_ROOT/.agents/skills/agy-brain-explorer/scripts/agy-brain-explorer.py" delete --project "audio_inventory_skilled" --force
> rm -rf "$EXP_DIR/audio_inventory_skilled" "$REPO_ROOT/artifacts/traces/audio_inventory_skilled" "$REPO_ROOT/recordings/raw/02-skilled.cast" "$REPO_ROOT/recordings/final/02-skilled.cast"
> ```

---

## Step 4: Clone Skilled Run & Record Run 3 — Production Run (`03-production.cast`)

### 4.1 Clone the Finished Skilled Run Workspace
Before starting the recording, make a clean copy of the finished **Skilled Run** directory (`audio_inventory_skilled`) as the starting point for `audio_inventory_production`:
```bash
cd "$EXP_DIR"
rm -rf "$EXP_DIR/audio_inventory_production"
cp -R "$EXP_DIR/audio_inventory_skilled" "$EXP_DIR/audio_inventory_production"
```

### 4.2 Start Recording Inside `audio_inventory_production`
```bash
cd "$EXP_DIR/audio_inventory_production"
asciinema rec --window-size 120x32 --idle-time-limit 2.0 --overwrite "$REPO_ROOT/recordings/raw/03-production.cast"
```

### 4.3 Install Google Cloud Skills & Productionize Inside the Recording

1. **Install Google Cloud skills live in the recording**:
   ```bash
   npx skills@latest add google/skills
   ```
2. **Launch `agy`**:
   ```bash
   agy
   ```
3. **Prompt Antigravity with `/wayfinder` to plan the Google Cloud migration**:
   ```text
   /wayfinder I'd like to adapt and productionize this web application for a low-volume, low-cost deployment on Google Cloud using Cloud Run and Cloud Firestore.
   ```
4. **Answer clarifying questions** using the **Cloud Target & Cost Constraint — Run 3 (Production Run)** entry in the **Canonical Q&A Answers** table above:
   - Migrate persistence from local Turso/libSQL to **Cloud Firestore** behind the existing repository trait seam (keeping local/in-memory support for tests).
   - Containerize the Rust + HTMX server (`Dockerfile`).
   - Deploy to **Cloud Run** (`--min-instances=0`, `--allow-unauthenticated`, `us-central1`) so idle cost is `$0/month` within the free tier (avoiding always-on Cloud SQL VM costs).
5. **Work through the `/wayfinder` migration tickets** (`/clear` → `/wayfinder` → answer questions → `/clear`) until the cloud migration map is resolved.
6. **Generate the migration spec, slice into tickets, and implement & deploy** (`/to-spec` → `/to-tickets` → `/implement`, with `/clear` between stages as appropriate) until:
   - `cargo test` passes against the repository seam.
   - The container builds and deploys live to **Cloud Run**.
   - The live `.run.app` URL responds with the seeded audio electronics **Component** and **Kit** workbench UI backed by **Cloud Firestore**.
7. Exit `agy` and press `Ctrl+D` (or `exit`) to save `03-production.cast`.

### 4.4 Process, Inspect & Decide: Good Capture or Redo Run 3?
Immediately after exiting the recording shell, run `process-run` to sanitize/trim `03-production.cast`, auto-inject chapter markers via `mark-cast.py`, sync `prototypes/03-production-cast-player.html`, and export the Antigravity **Session Traces** for `audio_inventory_production`:

```bash
"$REPO_ROOT/scripts/setup-workspaces.sh" process-run 03-production.cast audio_inventory_production
```

**Verify the capture:**
1. **Check the Terminal Cast & auto-injected stop points**:
   ```bash
   asciinema play -m -s 2.0 -i 1.0 "$REPO_ROOT/recordings/final/03-production.cast"
   ```
2. **Check the exported Agy Session Traces**:
   ```bash
   cat "$REPO_ROOT/artifacts/traces/audio_inventory_production/README.md"
   ```
3. **Check the Production Run artifacts, tests & live Cloud Run service**:
   ```bash
   cd "$EXP_DIR/audio_inventory_production"
   cargo test
   gcloud run services list --region=us-central1
   ```

> **If you need to REDO Run 3**: Clean up the `audio_inventory_production` directory and its Agy brain sessions (leaving `audio_inventory_skilled` untouched), then repeat from **Step 4.1**:
> ```bash
> uv run "$REPO_ROOT/.agents/skills/agy-brain-explorer/scripts/agy-brain-explorer.py" delete --project "audio_inventory_production" --force
> rm -rf "$EXP_DIR/audio_inventory_production" "$REPO_ROOT/artifacts/traces/audio_inventory_production" "$REPO_ROOT/recordings/raw/03-production.cast" "$REPO_ROOT/recordings/final/03-production.cast"
> ```

---

## Step 5: Sync Finished App Workspaces into `$REPO_ROOT/experiments/`

Once all three runs are verified, sync the completed project directories into `$REPO_ROOT/experiments/` (automatically excluding `target/`, `.venv/`, `__pycache__/`, and nested `.git/` directories) and verify the final status:

```bash
"$REPO_ROOT/scripts/setup-workspaces.sh" sync-experiments
"$REPO_ROOT/scripts/setup-workspaces.sh" status
```

---

## Step 6: Adding or Adjusting Stop Points Post-Recording (`mark-cast.py`)

`process-run` automatically injects canonical chapter markers for each run (`--auto-baseline`, `--auto-skilled`, `--auto-production`, backed by `scripts/markers/*.json`). You can also inspect, search, or adjust stop points on any `.cast` file at any time using `mark-cast.py`:

```bash
MARK_SCRIPT="$REPO_ROOT/.agents/skills/asciinema/scripts/mark-cast.py"

# 1. List existing stop points and timestamps (raw, 1x, and time-warped)
python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" --list

# 2. Search terminal output with surrounding context events (to distinguish command-entry vs execution)
python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" --search "audio_inventory_baseline" --context 2

# 3. Inspect a range of events (#idx, dt, raw, 1x, code, stripped text)
python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" --inspect 0:25

# 4. Move/replace an existing stop point by number or label (and sync the standalone HTML cast player)
python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" \
  --set 1 "mkdir audio_inventory_baseline && cd audio_inventory_baseline" "1. Create Project Directory" \
  --in-place --embed-html "$REPO_ROOT/prototypes/01-baseline-cast-player.html"

# 5. Add a stop point right after a terminal output pattern (use pattern#2 for 2nd occurrence)
python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" \
  --add-pattern "Please review the codebase" "Codebase Review" --in-place

# 6. Clear all markers, re-apply canonical markers, tighten paste/submit delays, and sync HTML player
python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" \
  --clear --auto-baseline --tighten-pastes --in-place \
  --embed-html "$REPO_ROOT/prototypes/01-baseline-cast-player.html"
```

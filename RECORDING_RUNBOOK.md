# Operator Recording Runbook: Three-Run Flutter Experiment

This document lists the step-by-step commands, prompts, and canonical Q&A answers to run the three-part experiment in a clean directory (`~/Demos/flutter-parts-experiment`), record and verify each `asciinema` **Terminal Cast** and its **Antigravity Session Traces** after each run, and copy the finished app workspaces into this repository.

---

## Overview of Directory Locations

The three Flutter apps are created in `~/Demos/flutter-parts-experiment` (`$EXP_DIR`) outside this repo so `parts_tracker_baseline` does not inherit this repo's `.agents/skills`, while recordings and extracted session traces are written directly into `$REPO_ROOT`:

```text
~/Demos/flutter-parts-experiment/          # $EXP_DIR (clean workspace root outside this repo)
├── parts_tracker_baseline/                # Run 1: Created via flutter create --platforms=ios (no skills)
├── parts_tracker_skilled/                 # Run 2: Created via flutter create --platforms=ios + npx skills@latest add mattpocock/skills
└── parts_tracker_comprehension/           # Run 3: Cloned from Skilled Run + npx skills@latest add domesticmouse/skills

/Users/brett/Documents/GitHub/flutter-skills-presentation/   # $REPO_ROOT
├── recordings/
│   ├── raw/                               # Raw .cast files captured during each run
│   │   ├── 01-baseline.cast
│   │   ├── 02-skilled.cast
│   │   └── 03-comprehension.cast
│   └── final/                             # Sanitized & exit-trimmed .cast files verified after each run
│       ├── 01-baseline.cast
│       ├── 02-skilled.cast
│       └── 03-comprehension.cast
├── artifacts/
│   └── traces/                            # Exported per-session Agy Markdown logs verified after each run
│       ├── parts_tracker_baseline/
│       ├── parts_tracker_skilled/
│       └── parts_tracker_comprehension/
└── experiments/                           # Completed app workspaces copied in Step 5
```

---

## Step 0: Pre-Flight Tool & Environment Check

Run these quick checks before starting the recordings to confirm every required CLI tool, SDK, and configuration is ready:

### 0.1 Verify Flutter, iOS Toolchain (`flutter doctor`) & Boot iOS Simulator
```bash
flutter --version
flutter doctor
open -a DeviceHub
xcrun simctl boot 110F4146-D907-4E14-8AE8-453F310E8A84 2>/dev/null || true
xcrun simctl list devices booted
```
- Confirm **Flutter**, **Xcode** (for iOS development), and **CocoaPods** show `[✓]`, and that an iOS Simulator is booted (in Xcode 27+, `Simulator.app` was replaced by `DeviceHub.app` at `/Applications/Xcode.app/Contents/Applications/DeviceHub.app`) so `Please run the app.` launches without device-selection prompts.
- Optional quick smoke test that `flutter create --platforms=ios` succeeds cleanly without first-run prompts:
  ```bash
  flutter create --platforms=ios /tmp/flutter_preflight_test && rm -rf /tmp/flutter_preflight_test
  ```

### 0.2 Verify `asciinema` Version & `--window-size` Flag (`>= 3.0`)
The runbook uses `asciinema 3.x` flags (`--window-size 120x32`). Chapter stop-point markers (`"m"` events) are injected automatically post-recording by `mark-cast.py`. Verify the installed version and run a 1-second non-interactive smoke test:
```bash
asciinema --version
asciinema rec --window-size 120x32 --idle-time-limit 2.0 --overwrite -c "echo 'asciinema pre-flight OK'" /tmp/preflight.cast && rm -f /tmp/preflight.cast
```

### 0.3 Pre-Warm `npx skills@latest`
Run 2 and Run 3 invoke `npx skills@latest add ...` live on camera. Pre-warm the `skills` package cache so `npx` doesn't prompt to install `skills@latest` during the recording:
```bash
npx --yes skills@latest --version
```

### 0.4 Verify Antigravity CLI (`agy`) & Your Global Rule
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
export REPO_ROOT="/Users/brett/Documents/GitHub/flutter-skills-presentation"
export EXP_DIR="$HOME/Demos/flutter-parts-experiment"
mkdir -p "$EXP_DIR" "$REPO_ROOT/recordings/raw" "$REPO_ROOT/recordings/final" "$REPO_ROOT/artifacts/traces"
cd "$EXP_DIR"
```

---

## Canonical Q&A Answers (Used Across Runs 1 & 2)

Keep these answers handy so both the **Baseline Run** and **Skilled Run** receive identical product requirements when `agy` asks clarifying questions:

| Topic | Canonical Answer to Copy/Paste |
| :--- | :--- |
| **Parts / Categories** | `Hobbyist/maker electronics parts: resistors, capacitors, ICs/microcontrollers, sensors, LEDs, and breakout boards.` |
| **Attributes per Part** | `Part name/number, category, quantity on hand, storage location (e.g., drawer/bin label), and a freeform specs/notes field (e.g., "10kΩ 1%", "3.3V I2C").` |
| **Core Workflows** | `Browse and search parts, filter by category, increment/decrement quantity quickly, and add/edit parts. No barcode scanning, BOM/project tracking, or supplier API integration.` |
| **Persistence** | `Local on-device storage only (no cloud sync or user accounts), pre-populated with a small set of sample parts on first launch.` |
| **UI Style** | `Standard clean iOS look (Cupertino or adaptive Material 3).` |
| **Issue Tracker** *(Run 2 only)* | `Local Markdown (.scratch/)` |
| **Domain Modeling** *(Run 2 only)* | `Single context (GLOSSARY.md + docs/adr/)` |

---

## Step 2: Record Run 1 — Baseline Run (`01-baseline.cast`)

### 2.1 Start Recording in `$EXP_DIR`
```bash
cd "$EXP_DIR"
asciinema rec --window-size 120x32 --idle-time-limit 2.0 --overwrite "$REPO_ROOT/recordings/raw/01-baseline.cast"
```

### 2.2 Create the Flutter Project & Launch `agy`
Inside the recorded shell:

1. **Create the Flutter project**:
   ```bash
   flutter create --platforms=ios parts_tracker_baseline
   ```
2. **Move into the project**:
   ```bash
   cd parts_tracker_baseline
   ```
3. **Start Antigravity in the Flutter project**:
   ```bash
   agy
   ```
4. **Paste the initial prompt**:
   ```text
   I'd like to build a Flutter app for iOS that enables me to track my electronics parts
   ```
5. Answer `agy`'s one-at-a-time clarifying questions using the **Canonical Q&A Answers** table above.
6. Let `agy` build the Flutter iOS app and verify `flutter analyze` / `flutter test`.
7. **Ask `agy` to run the app**:
   ```text
   Please run the app.
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

### 2.3 Process, Inspect & Decide: Good Capture or Redo Run 1?
Immediately after exiting the recording shell, run `process-run` to sanitize/trim `01-baseline.cast`, auto-inject chapter stop-point markers via `mark-cast.py`, and export the Antigravity session logs for `parts_tracker_baseline`:

```bash
"$REPO_ROOT/.scratch/flutter-skills-presentation/prototypes/setup-workspaces.sh" process-run 01-baseline.cast parts_tracker_baseline
```

**Verify the capture:**
1. **Check the Asciinema cast & auto-injected stop points**:
   - Confirm `recordings/final/01-baseline.cast` has non-zero stdout events, chapter markers, and expected playback duration.
   - Quick visual replay in marker-pause mode at 2x speed (`Space` to resume after each stop point, `]` to jump to next marker):
     ```bash
     asciinema play -m -s 2.0 -i 1.0 "$REPO_ROOT/recordings/final/01-baseline.cast"
     ```
2. **Check the exported Agy Markdown logs**:
   - Inspect the multi-session summary (Session 1: build & run; Session 2: codebase review) and exported Markdown traces:
     ```bash
     cat "$REPO_ROOT/artifacts/traces/parts_tracker_baseline/README.md"
     ```
   - Confirm `agy` asked its clarifying questions one at a time (per your Global Rule), built and ran the app, reviewed the codebase after `/clear`, and did **not** produce architecture artifacts (`GLOSSARY.md` or `docs/adr/`).
3. **Check the Flutter project**:
   ```bash
   cd "$EXP_DIR/parts_tracker_baseline" && flutter analyze && flutter test
   ```

> **If you need to REDO Run 1**: Clean up both the project directory **and** the aborted `parts_tracker_baseline` Agy brain sessions (so failed sessions don't pollute your exported logs), then repeat from **Step 2.1**:
> ```bash
> uv run "$REPO_ROOT/.agents/skills/agy-brain-explorer/scripts/agy-brain-explorer.py" delete --project "parts_tracker_baseline" --force
> rm -rf "$EXP_DIR/parts_tracker_baseline" "$REPO_ROOT/artifacts/traces/parts_tracker_baseline" "$REPO_ROOT/recordings/raw/01-baseline.cast" "$REPO_ROOT/recordings/final/01-baseline.cast"
> ```

---

## Step 3: Record Run 2 — Skilled Run (`02-skilled.cast`)

### 3.1 Start Recording in `$EXP_DIR`
```bash
cd "$EXP_DIR"
asciinema rec --window-size 120x32 --idle-time-limit 2.0 --overwrite "$REPO_ROOT/recordings/raw/02-skilled.cast"
```

### 3.2 Execute the Skilled Run Sequence Inside the Recording

1. **Create the Flutter project**:
   ```bash
   flutter create --platforms=ios parts_tracker_skilled
   ```
2. **Move into the project**:
   ```bash
   cd parts_tracker_skilled
   ```
3. **Install all of Matt Pocock's skills**:
   ```bash
   npx skills@latest add mattpocock/skills
   ```
4. **Start Antigravity in the Flutter project**:
   ```bash
   agy
   ```
5. **Prompt Antigravity with `/wayfinder`**:
   ```text
   /wayfinder I'd like to build a Flutter app for iOS that enables me to track my electronics parts
   ```
6. **Answer questions** using the **Canonical Q&A Answers** table above until the map and initial decision tickets are created.
7. **Issue `/clear` in `agy`** to start a new conversation:
   ```text
   /clear
   ```
8. **Issue `/wayfinder`** to start the next round of questions on the next open decision ticket:
   ```text
   /wayfinder
   ```
9. **Answer questions** to resolve the decision ticket.
10. **Issue `/clear`** to start a new conversation:
    ```text
    /clear
    ```
11. **Repeat steps 8 through 10** (`/wayfinder` → answer questions → `/clear`) until all `/wayfinder` decision tickets are complete.
12. **Complete the build of the app** (e.g., `/to-spec` → `/to-tickets` → `/implement`, with `/clear` between stages as appropriate) until the Flutter app is built and verified.
13. **Ask `agy` to run the app**:
    ```text
    Please run the app.
    ```
14. **Start a new `agy` session**:
    ```text
    /clear
    ```
15. **Review the codebase**:
    ```text
    Please review the codebase
    ```
16. **Ask `agy` to fix the issues surfaced by the review**:
    ```text
    please fix
    ```
17. Exit `agy` and press `Ctrl+D` (or `exit`) to save `02-skilled.cast`.

### 3.3 Process, Inspect & Decide: Good Capture or Redo Run 2?
Immediately after exiting the recording shell, run `process-run` to sanitize/trim `02-skilled.cast`, auto-inject chapter markers via `mark-cast.py`, and export all `/clear`-separated Antigravity session logs for `parts_tracker_skilled`:

```bash
"$REPO_ROOT/.scratch/flutter-skills-presentation/prototypes/setup-workspaces.sh" process-run 02-skilled.cast parts_tracker_skilled
```

**Verify the capture:**
1. **Check the Asciinema cast & auto-injected stop points**:
   - Confirm the output lists your `"m"` markers at the expected timestamps.
   - Replay in marker-pause mode (`-m`, press `Space` or `]` to jump between markers):
     ```bash
     asciinema play -m -s 2.0 -i 1.0 "$REPO_ROOT/recordings/final/02-skilled.cast"
     ```
2. **Check the exported Agy Markdown logs**:
   - Review the multi-session index (each `/clear` creates a distinct conversation trace in `artifacts/traces/parts_tracker_skilled/`):
     ```bash
     cat "$REPO_ROOT/artifacts/traces/parts_tracker_skilled/README.md"
     ```
3. **Check the Skilled Run artifacts & tests**:
   ```bash
   cd "$EXP_DIR/parts_tracker_skilled"
   ls -la GLOSSARY.md docs/adr/ .scratch/
   flutter analyze && flutter test
   ```

> **If you need to REDO Run 2**: Clean up both the project directory **and** the aborted `parts_tracker_skilled` Agy brain sessions, then repeat from **Step 3.1**:
> ```bash
> uv run "$REPO_ROOT/.agents/skills/agy-brain-explorer/scripts/agy-brain-explorer.py" delete --project "parts_tracker_skilled" --force
> rm -rf "$EXP_DIR/parts_tracker_skilled" "$REPO_ROOT/artifacts/traces/parts_tracker_skilled" "$REPO_ROOT/recordings/raw/02-skilled.cast" "$REPO_ROOT/recordings/final/02-skilled.cast"
> ```

---

## Step 4: Clone Skilled Run App & Record Run 3 — Comprehension Run (`03-comprehension.cast`)

### 4.1 Clone the Finished Skilled Run App
Before starting the recording, make a clean copy of the finished Skilled Run directory (`parts_tracker_skilled`) as the starting point:
```bash
cd "$EXP_DIR"
rm -rf "$EXP_DIR/parts_tracker_comprehension"
cp -R "$EXP_DIR/parts_tracker_skilled" "$EXP_DIR/parts_tracker_comprehension"
```

### 4.2 Start Recording
```bash
cd "$EXP_DIR/parts_tracker_comprehension"
asciinema rec --window-size 120x32 --idle-time-limit 2.0 --overwrite "$REPO_ROOT/recordings/raw/03-comprehension.cast"
```

### 4.3 Install Skills & Run `/teach-code` Inside the Recording

1. **Install skills live in the recording**:
   ```bash
   npx skills@latest add domesticmouse/skills
   ```
2. **Launch `agy`**:
   ```bash
   agy
   ```
3. **Paste the `/teach-code` prompt**:
   ```text
   /teach-code I know React for mobile development, and I just had AI generate this Flutter iOS electronics parts tracker app for me. Help me understand how this codebase works and teach me the Flutter and Dart concepts behind it.
   ```

Use these responses when `/teach-code` interviews you:
- **Prior Background**: `Comfortable with React mobile development (components, props, hooks/useState, JSX), brand new to Dart and Flutter widget trees and state management.`
- **Mission Goal**: `Understand and confidently modify every file in this electronics parts catalog app without relying blindly on the AI.`

**Stopping Boundary**: Stop once `/teach-code` scaffolds `./tutorial/` (`MISSION.md`, `RESOURCES.md`, `NOTES.md`, `reference/GLOSSARY.md`, `build.py`), authors and verifies **Lesson 0001** (including its companion project under `tutorial/apps/0001-...`), and compiles `tutorial/index.html` and `tutorial/lessons/0001-*.html` cleanly via `build.py --check`.

Exit `agy` and press `Ctrl+D` (or `exit`) to save `03-comprehension.cast`.

### 4.4 Process, Inspect & Decide: Good Capture or Redo Run 3?
Immediately after exiting the recording shell, run `process-run` to sanitize/trim `03-comprehension.cast`, auto-inject chapter markers via `mark-cast.py`, and export the Antigravity session logs for `parts_tracker_comprehension`:

```bash
"$REPO_ROOT/.scratch/flutter-skills-presentation/prototypes/setup-workspaces.sh" process-run 03-comprehension.cast parts_tracker_comprehension
```

**Verify the capture:**
1. **Check the Asciinema cast & auto-injected stop points**:
   ```bash
   asciinema play -m -s 2.0 -i 1.0 "$REPO_ROOT/recordings/final/03-comprehension.cast"
   ```
2. **Check the exported Agy Markdown logs**:
   ```bash
   cat "$REPO_ROOT/artifacts/traces/parts_tracker_comprehension/README.md"
   ```
3. **Check the generated `/teach-code` curriculum & HTML build**:
   ```bash
   cd "$EXP_DIR/parts_tracker_comprehension"
   python3 tutorial/build.py --check
   ls -la tutorial/index.html tutorial/lessons/ tutorial/apps/
   ```

> **If you need to REDO Run 3**: Clean up the `parts_tracker_comprehension` directory and its Agy brain sessions (leaving `parts_tracker_skilled` untouched), then repeat from **Step 4.1**:
> ```bash
> uv run "$REPO_ROOT/.agents/skills/agy-brain-explorer/scripts/agy-brain-explorer.py" delete --project "parts_tracker_comprehension" --force
> rm -rf "$EXP_DIR/parts_tracker_comprehension" "$REPO_ROOT/artifacts/traces/parts_tracker_comprehension" "$REPO_ROOT/recordings/raw/03-comprehension.cast" "$REPO_ROOT/recordings/final/03-comprehension.cast"
> ```

---

## Step 5: Copy Finished App Workspaces into `$REPO_ROOT/experiments/`

Once all three runs are verified, copy the completed project directories into this repository and check the final status:

```bash
mkdir -p "$REPO_ROOT/experiments"
cp -R "$EXP_DIR/parts_tracker_baseline" "$REPO_ROOT/experiments/"
cp -R "$EXP_DIR/parts_tracker_skilled" "$REPO_ROOT/experiments/"
cp -R "$EXP_DIR/parts_tracker_comprehension" "$REPO_ROOT/experiments/"

"$REPO_ROOT/.scratch/flutter-skills-presentation/prototypes/setup-workspaces.sh" status
```

---

## Step 6: Adding or Adjusting Stop Points Post-Recording (`mark-cast.py`)

`process-run` automatically injects canonical chapter markers for each run (`--auto-baseline`, `--auto-skilled`, `--auto-comprehension`). You can also inspect, search, or add custom stop points to any `.cast` file at any time using `mark-cast.py`:

```bash
MARK_SCRIPT="$REPO_ROOT/.agents/skills/asciinema/scripts/mark-cast.py"

# 1. List existing stop points and timestamps (raw, 1x, and time-warped)
python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" --list

# 2. Search terminal output with surrounding context events (to distinguish command-entry vs execution)
python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" --search "flutter create" --context 2

# 3. Inspect a range of events (#idx, dt, raw, 1x, code, stripped text)
python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" --inspect 0:25

# 4. Move/replace an existing stop point by number or label (and optionally sync the HTML test-drive player)
python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" \
  --set 1 "flutter create --platforms=ios parts_tracker_baseline" "1. Create Project" \
  --in-place --embed-html "$REPO_ROOT/.scratch/flutter-skills-presentation/prototypes/06-baseline-cast-player.html"

# 5. Add a stop point right after a terminal output pattern (use pattern#2 for 2nd occurrence)
python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" \
  --add-pattern "Please review the codebase" "Codebase Review" --in-place

# 6. Clear all markers, re-apply canonical markers, tighten paste/submit delays, and sync HTML player
python3 "$MARK_SCRIPT" "$REPO_ROOT/recordings/final/01-baseline.cast" \
  --clear --auto-baseline --tighten-pastes --in-place \
  --embed-html "$REPO_ROOT/.scratch/flutter-skills-presentation/prototypes/06-baseline-cast-player.html"
```

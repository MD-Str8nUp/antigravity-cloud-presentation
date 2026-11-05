---
name: cast-player-html
description: >-
  Edit, scaffold, and manage standalone HTML asciinema cast-player files
  (*-cast-player.html) and their embedded gzip+base64 recordings
  (EMBEDDED_CAST_GZ_B64). Use whenever reviewing or editing a cast-player HTML
  file, adding, moving, removing, or relabeling stop points ("m" chapter
  markers) viewed in an HTML player, shortening cut-and-paste or prompt delays
  (dt), tuning adaptive segment speeds, updating chapter pills or HUD controls,
  or scaffolding a new cast-player HTML prototype for a terminal recording.
---

# Cast-Player HTML (`*-cast-player.html`)

Standalone HTML test-drive harnesses wrap `asciinema-player@3.9.0` inside a 16:9 Slidev `cast-full` stage with two-pass adaptive segment speed compilation (`>= 2x`, `<= 30s` per segment), interactive chapter stepper pills, a live speed HUD, and stage hotkeys (`p`, `[`, `]`).

## File Mapping & Single Source of Truth

Each HTML player embeds a gzipped, base64-encoded copy (`const EMBEDDED_CAST_GZ_B64 = "..."`) of its backing `.cast` file (e.g. in `recordings/final/<name>.cast`) so `file://` URLs load in any browser without CORS errors.

### Core Rules
1. **Treat the backing `.cast` file (e.g. `recordings/final/<name>.cast`) as the single source of truth** for terminal events and `"m"` stop points. Always mutate the `.cast` file first, re-embed `EMBEDDED_CAST_GZ_B64` using `.agents/skills/asciinema/scripts/mark-cast.py --embed-html <html-path>`, and (if working with a Slidev deck) sync `slides/public/casts/` via `npm --prefix slides run sync-casts` so both the standalone HTML player and the Slidev deck stay in sync.
2. **Skip the `EMBEDDED_CAST_GZ_B64` line when reading populated HTML players**: Line ~454 is a single-line base64 string that can exceed the 46 KB `read_file` byte cap when a large recording is embedded. Read the HTML in two ranges around it:
   - Header, CSS, and DOM layout: `StartLine: 1, EndLine: 453`
   - JS compiler, chapter pills, and event handlers: `StartLine: 455`

---

## Workflows

1. [Add, Move, Remove, or Rename Stop Points](#1-add-move-remove-or-rename-stop-points-and-tighten-paste-delays)
2. [Edit HTML/CSS/JS Player Controls or Adaptive Speed Logic](#2-edit-htmlcssjs-player-controls-or-adaptive-speed-logic)
3. [Scaffold a New Cast-Player HTML File](#3-scaffold-a-new-cast-player-html-file)

---

### 1. Add, Move, Remove, or Rename Stop Points (and Tighten Paste Delays)

When the user reviews a `*-cast-player.html` file and asks to add, move, delete, or rename a stop point, or shorten cut-and-paste / input pauses:

#### Step 1: Inspect Current Markers and Target Events
List current markers and search the backing `.cast` file:

```bash
# List existing markers, event indices, 1x times, and adaptive segment speeds
python3 .agents/skills/asciinema/scripts/mark-cast.py recordings/final/session.cast --list

# Search stdout events for the target text with surrounding context
python3 .agents/skills/asciinema/scripts/mark-cast.py recordings/final/session.cast \
  --search "target command or prompt" --context 8

# Inspect exact event boundaries around a candidate index
python3 .agents/skills/asciinema/scripts/mark-cast.py recordings/final/session.cast --inspect 70:85
```

#### Step 2: Apply the Matching Stop-Point Placement Rule
Place every `"m"` marker **immediately after** the `"o"` stdout event that completes the target visual frame:

- **A. Shell Command / TUI Launcher Entry (`npx skills@latest add ...`, `agy`)**:
  Recordings start inside the target workspace directory, so the first command is either `agy` (baseline runs) or `npx skills@latest add ...` followed by `agy` (skill-enabled runs). Match the command text itself. In `zsh` with bracketed paste, `mark-cast.py` automatically advances from `[REV]` (`\x1b[7m` highlight) to `[UNREV]` (`\x1b[27m` Enter redraw) and splits any trailing `\x1b[1B\r\x1b[K` cursor-down into a post-marker `"o"` event (`dt=0.02s`) so the cursor stays on the command line while paused.
- **B. Interactive Questions (`ask_user` / `ask_question`) and Initial Prompts**:
  - **Multi-Select or Write-In Questions & Initial Prompt Entry (Two-Stop Pattern)**: Use two paired stop points when the user toggles checkboxes (`[x]`), types into a `Write-in...` input, or enters a multi-word initial prompt or slash command with arguments (`/code-review`, `/deploy`):
    1. **Before Typing (`Q: ...` / `Ready for ...`)**: Place immediately after the stdout frame that finishes rendering the question or empty input box, right before the first keystroke/paste event of the user's answer.
    2. **After Typing, Before `Enter` (`Answer: ...` / `/command`)**: Place immediately after the stdout frame that finishes echoing the typed/pasted answer or slash command in the input box, right before the `Enter` submission event (`dt` pause followed by input-box clear `\x1b[\d+D\x1b[K`, `Thinking...`, `Generating...`, or autocomplete dismissal).
    - **Audit for Unpaired Stops**: Verify every "after typing" stop point has a matching "before typing" stop point immediately before the typing sequence begins.
  - **Single-Select Questions Accepting the Pre-Selected Default (Single-Stop Pattern)**: When an `ask_user` / `ask_question` prompt is single-select and the user simply presses `Enter` on the already-highlighted first option (`❯ 1. ...`), use a **single stop point** when the question renders (`Q1: ...`)—no second "after typing" stop is needed since the selection does not change before `Enter`.
- **C. Pre-Summary Stop Point for Long Build / Tool Phases (`Build Execution Complete` -> `Build Summary`)**:
  When a build or tool-execution phase runs for several minutes (causing adaptive segment compilation to fast-forward at `15x–30x` turbo speed), place a stop point **immediately before the final prose summary begins streaming** (e.g., right before `"I have built and launched..."`) in addition to the `Build Summary` stop at the end of the response (`? for shortcuts`). This separates the fast-forwarded build execution (`15x–30x`) from the final summary so the summary streams in at a readable `2x` speed (`~8s`).
- **D. Short Transition Commands (`/clear`, `please fix`) — Single-Stop Transition**:
  Short transition commands like `/clear` or `please fix` do **not** need a stop point after typing before `Enter`. Stop at the completed output before typing (`Build Summary` before `/clear`, `Review Findings` before `please fix`), and place the next stop point after the transition completes (`Session Cleared` after `/clear` redraws the fresh CLI banner and empty prompt box, or `Fixes Verified` after `please fix` finishes). This keeps `/clear` (`~2s @ 2x`) and `please fix` transitions brisk during live presentation stepping.
- **E. Synchronized TUI Frame Flushes & Keystroke Redraws**:
  - When the TUI writes a synchronized frame (`\x1b[?2026h`…`\x1b[?2026l`) across multiple `dt = 0.0` PTY chunks, `mark-cast.py` automatically advances past trailing `dt = 0.0` `"o"` chunks so the marker sits after the complete frame flush.
  - In `agy`'s TUI, character-by-character typing and write-in inputs emit backspace-redrawn or cursor-positioned chunks (e.g. `\x08p\x08r\x08o\x08j\x08e\x08c\x08t` or `p\n\n Glease fix`), and typing `/clear` redraws the slash-command autocomplete menu on each keystroke (`/c`, `/cl`, `/cle`, `/clea`, `/clear`). When writing regexes for `--auto-spec` (`scripts/markers/*.json`), match the full final keystroke frame rather than a prefix that also appears in the `/c` autocomplete description.

#### Step 3: Update the `.cast` File, Relabel Sequentially, and Sync the HTML Player & Slidev Deck
Keep chapter labels numbered sequentially (`1. ...`, `2. ...`, ..., `N. ...`), sync `EMBEDDED_CAST_GZ_B64` into the HTML file, and sync `slides/public/casts/` (if applicable):

```bash
# 1. Insert, move, or remove the stop point in the backing .cast file
python3 .agents/skills/asciinema/scripts/mark-cast.py recordings/final/session.cast \
  --add-pattern "initial prompt text" "2. Initial Prompt" \
  --in-place

# 2. Relabel markers sequentially (1..N) and re-embed gzip+base64 into the HTML player
python3 .agents/skills/asciinema/scripts/mark-cast.py recordings/final/session.cast \
  --relabel \
    "1. Launch agy" \
    "2. Initial Prompt" \
    "3. Q1: Clarifying Question" \
    "4. Q1: Answer" \
    "5. Build Summary" \
  --in-place \
  --embed-html prototypes/session-cast-player.html

# 3. Sync recordings/final/*.cast into slides/public/casts/ (when paired with Slidev)
npm --prefix slides run sync-casts
```

If you maintain a JSON marker specification file for `--auto-spec <json_file>`, update it when the canonical marker list changes.

#### Step 4: Tighten Cut-and-Paste or Specific Event Delays (`dt`)
Because `asciicast-v3` uses relative event deltas (`dt`) and `prepareCastForPlayer()` dynamically compiles cumulative `asciicast-v2` timestamps and segment speeds, shortening `dt` on individual events never desynchronizes markers.

Default target delays (`--tighten-pastes`):
- **`1.0s` (`0.5s` at `2x` playback)**: Pre-paste wait before `[REV]` shell pastes, `agy` startup banner settle, `agy` pasted/typed prompt starts, and `/clear` execution.
- **`0.15s`**: `[REV]` → `[UNREV]` reverse-video highlight redraw when pressing `Enter` on a pasted shell command.
- **`0.20s`**: Post-marker input-box clear/submit pause (`\x1b[\d+D\x1b[K`) when resuming from an `agy` prompt stop point.

```bash
# Automatically tighten all [REV] shell paste delays (1.0s), [UNREV] Enter redraws (0.15s), and agy prompt paste/submit pauses (0.20s)
python3 .agents/skills/asciinema/scripts/mark-cast.py recordings/final/session.cast \
  --tighten-pastes \
  --in-place \
  --embed-html prototypes/session-cast-player.html

# Or set the relative dt of specific event indices directly
python3 .agents/skills/asciinema/scripts/mark-cast.py recordings/final/session.cast \
  --set-dt 7 1.0 \
  --set-dt 8 0.15 \
  --in-place \
  --embed-html prototypes/session-cast-player.html
```

- **Completion Criterion**: Run `mark-cast.py <cast> --list` and `--inspect <idx-3>:<idx+3>` around the modified marker or event. Confirm markers are numbered `1..N` in chronological order, the marker sits immediately after the target stdout frame and before any newline/clear/spinner event, target event `dt` values reflect the tightened delays, and `mark-cast.py` printed `Updated EMBEDDED_CAST_GZ_B64 in <html-path>`.

---

### 2. Edit HTML/CSS/JS Player Controls or Adaptive Speed Logic

1. Read the HTML file while skipping the `EMBEDDED_CAST_GZ_B64` line (~line 454):
   - Lines `1..453`: CSS variables, `.slidev-frame` (16:9 container), `.slide-header`, `.chapter-pills`, `#player-container`, `.speed-hud-pill`, and `.slide-footer`.
   - Lines `455..end`:
     - `prepareCastForPlayer(castText, idleTimeLimitOverride, minSpeed, maxSegmentSec)`: Two-pass compiler that splits events at `"m"` markers, omits trailing `"x"` exit-hold events (so playback reaches `ended` immediately when the final `"o"` frame renders rather than stalling for 3s of dead air), calculates `segSpeed = max(minSpeed, ceil(segCappedDur / maxSegmentSec))`, and emits warped `asciicast-v2` timestamps. Always snap `endTime = Number(warpedAbs.toFixed(6))`, `warpedAbs = endTime`, `v2Header.duration = warpedAbs`, and `ch.playTime = markerTime` so `player.seek(ch.playTime)` and `player.seek(tailSeg.endTime)` never fall short of 6-decimal rounded `"o"` event timestamps when `.toFixed(6)` rounds up.
     - `renderChapterPills(hitIdx)`, `hasTrailingSegment()`, and `getEndStopIndex()`: Renders `↺ 0. Start` (`idx = -1`), numbered chapter pills (`idx = 0..chapters.length - 1`) with speed badges, and (only when `hasTrailingSegment()` is true, i.e., `segments.length > chapters.length && tailSeg.warpedDuration > 0.05`) an interactive `→ End` stop-point button (`idx = chapters.length`). When the final `"m"` marker sits at the last stdout `"o"` event before `"x"` (`hasTrailingSegment() === false`), `getEndStopIndex()` returns `chapters.length - 1` so reaching the end of playback highlights the final stop-point pill (`chapters.length - 1`) instead of an out-of-bounds index `chapters.length`. **High Stop-Point Count Layout**: When a cast has dozens of stop points (e.g. 50+ markers), you can omit `<div class="chapter-pills" id="chapter-pills"></div>` from the DOM; `renderChapterPills()` checks `if (!container) return;` so the pill grid does not push `#player-container` off-screen inside the 16:9 `.slidev-frame`.
     - `jumpToStart()`, `jumpToChapter(idx)`, `jumpToEnd()`, `applyEndState()`, `stepPrevChapter()`, `stepNextChapter()`, `togglePlay()`, and `syncLiveSpeedFromPlayer()`: Synchronize all five stop-point indicators (`#status-badge`, `#chapter-pills`, `#live-speed-badge`, `#speed-hud-pill`, and `#marker-detail`) across keyboard jumps (`[` / `]`), pill clicks, natural playback completion (`ended`, `pause` at end, `marker` on the final chapter when `!hasTrailingSegment()`, and `t >= tailSeg.endTime - 0.02` in `syncLiveSpeedFromPlayer()`), and paused timeline-scrubber seeks (`lastPolledTime`). All `await player.seek(...)` calls are wrapped in an `isSeeking` guard (`try { ... } finally { isSeeking = false; }`), and `syncLiveSpeedFromPlayer()` both returns early when `isSeeking` or `!isPlaying && !timeChanged` and skips the `pause` listener's end-check while `isSeeking` is true—preventing the 100ms poll from resetting `activeChapterIdx` back to `endStopIdx` when pressing `[` (`stepPrevChapter()`) at the end of playback. `stepPrevChapter()` steps from `getEndStopIndex()` back to `getEndStopIndex() - 1` and from Stop Point 1 (`idx = 0`) to `0. Start` (`idx = -1`); `togglePlay()` replays from `0:00` (`↺ Replay`) when `activeChapterIdx >= getEndStopIndex()`.
     - `initPlayer()`: Mounts `AsciinemaPlayer.create({ data: v2Text }, container, { speed: 1, idleTimeLimit: 999999, ... })`, wires `play`, `pause`, `marker`, and `ended` events, and registers the `window` `keydown` listener in the **capture phase** (`true`) so `[` / `]` / `p` run before `asciinema-player`'s `.ap-wrapper` handler can swallow them when the terminal has focus.
2. Apply edits with `edit_file` targeting lines outside line 454.
3. If the change modifies shared UI/JS behavior, apply the same update to [`resources/cast-player-template.html`](resources/cast-player-template.html) and `.agents/skills/slidev-deck/resources/components/AsciinemaPlayer.vue` to keep all players consistent.

- **Completion Criterion**: Read back the edited line range to verify clean syntax, run `node --check` on the extracted `<script>` block, and confirm `const EMBEDDED_CAST_GZ_B64 = "H4sI...` remains intact.

---

### 3. Scaffold a New Cast-Player HTML File

To scaffold a new standalone HTML player for a `.cast` recording:

1. Copy the bundled template [`resources/cast-player-template.html`](resources/cast-player-template.html) to your target HTML path:
   ```bash
   cp .agents/skills/cast-player-html/resources/cast-player-template.html prototypes/my-session-cast-player.html
   ```
2. Edit the metadata strings in the new HTML file:
   - `<title>`: e.g. `Test Drive — my-session.cast (Slidev cast-full + Stop Points)`
   - `.top-title`: e.g. `recordings/final/my-session.cast · Adaptive Breakpoint Speeds`
   - `.slide-title`: e.g. `Session Title — Interactive Stop Points`
   - Comment above `EMBEDDED_CAST_GZ_B64`: reference the corresponding `.cast` file.
3. Embed the target `.cast` recording into the new HTML file:
   ```bash
   python3 .agents/skills/asciinema/scripts/mark-cast.py recordings/final/my-session.cast \
     --list \
     --embed-html prototypes/my-session-cast-player.html
   ```

- **Completion Criterion**: `mark-cast.py --list --embed-html` outputs `Updated EMBEDDED_CAST_GZ_B64 in <new-html-path>`, and reading lines `1..453` of the new HTML file confirms the updated titles and headers match the target recording.

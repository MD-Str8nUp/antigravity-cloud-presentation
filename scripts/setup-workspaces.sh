#!/usr/bin/env bash
# setup-workspaces.sh — Cast post-processor, trace extractor, and workspace sync helper
# for the Antigravity Cloud Website Experiment (3-run audio electronics inventory system).

set -euo pipefail

PRESENTATION_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -z "${DEMOS_ROOT:-}" ]]; then
  if [[ -d "$HOME/Demos/audio-inventory-experiment" ]]; then
    DEMOS_ROOT="$HOME/Demos/audio-inventory-experiment"
  elif [[ -d "$PRESENTATION_ROOT/experiments" ]]; then
    DEMOS_ROOT="$PRESENTATION_ROOT/experiments"
  else
    DEMOS_ROOT="$HOME/Demos/audio-inventory-experiment"
  fi
fi

BASELINE_DIR="$DEMOS_ROOT/audio_inventory_baseline"
SKILLED_DIR="$DEMOS_ROOT/audio_inventory_skilled"
PRODUCTION_DIR="$DEMOS_ROOT/audio_inventory_production"
RECORDINGS_RAW="$PRESENTATION_ROOT/recordings/raw"
RECORDINGS_FINAL="$PRESENTATION_ROOT/recordings/final"
TRACES_DIR="$PRESENTATION_ROOT/artifacts/traces"
PROTOTYPES_DIR="$PRESENTATION_ROOT/prototypes"
EXPERIMENTS_DIR="$PRESENTATION_ROOT/experiments"

usage() {
  cat <<EOF
Usage: $(basename "$0") <subcommand>

Subcommands:
  process-run <cast> <proj>  Sanitize/trim a single cast, inject chapter markers, update HTML player, export Agy Session Traces, and sync workspace into experiments/
  postprocess-casts          Run sanitize-cast.py, mark-cast.py, and trim-cast.py across all captured .cast files
  extract-traces             Export Session Traces from ~/.gemini/**/brain/ via agy-brain-explorer
  sync-experiments [proj]    Copy completed experiment workspace(s) from \$DEMOS_ROOT into \$PRESENTATION_ROOT/experiments/ (excluding target/ and .venv/)
  status                     Print current state of all 3 workspaces, .cast recordings, Session Traces, and HTML cast players
EOF
}

process_single_cast() {
  local cast_name="$1"
  mkdir -p "$RECORDINGS_RAW" "$RECORDINGS_FINAL"
  local mark_script="$PRESENTATION_ROOT/.agents/skills/asciinema/scripts/mark-cast.py"
  local sanitize_script="$PRESENTATION_ROOT/.agents/skills/asciinema/scripts/sanitize-cast.py"
  local trim_script="$PRESENTATION_ROOT/.agents/skills/asciinema/scripts/trim-cast.py"

  # If raw cast was recorded into $DEMOS_ROOT/recordings/raw, sync it into $RECORDINGS_RAW
  if [[ -f "$DEMOS_ROOT/recordings/raw/$cast_name" ]] && [[ "$DEMOS_ROOT/recordings/raw/$cast_name" != "$RECORDINGS_RAW/$cast_name" ]]; then
    cp "$DEMOS_ROOT/recordings/raw/$cast_name" "$RECORDINGS_RAW/$cast_name"
  fi

  local raw_file="$RECORDINGS_RAW/$cast_name"
  local final_file="$RECORDINGS_FINAL/$cast_name"
  local html_file=""
  if [[ ! -f "$raw_file" ]]; then
    echo "[SKIP] $raw_file not recorded yet"
    return 1
  fi
  echo "=== Processing $cast_name ==="
  python3 "$sanitize_script" "$raw_file" "$final_file"
  if [[ "$cast_name" == "01-baseline.cast" ]]; then
    html_file="$PROTOTYPES_DIR/01-baseline-cast-player.html"
    python3 "$mark_script" "$final_file" --auto-baseline --tighten-pastes --in-place
  elif [[ "$cast_name" == "02-skilled.cast" ]]; then
    html_file="$PROTOTYPES_DIR/02-skilled-cast-player.html"
    python3 "$mark_script" "$final_file" --auto-skilled --tighten-pastes --in-place
  elif [[ "$cast_name" == "03-production.cast" ]]; then
    html_file="$PROTOTYPES_DIR/03-production-cast-player.html"
    python3 "$mark_script" "$final_file" --auto-production --tighten-pastes --in-place
  fi
  python3 "$trim_script" "$final_file" --in-place --hold 3.0 || true
  if [[ -n "$html_file" ]] && [[ -f "$html_file" ]]; then
    python3 "$mark_script" "$final_file" --list --embed-html "$html_file"
  else
    python3 "$mark_script" "$final_file" --list
  fi
  python3 "$trim_script" "$final_file" --info
}

sync_single_experiment() {
  local proj="$1"
  mkdir -p "$EXPERIMENTS_DIR"
  local src="$DEMOS_ROOT/$proj"
  local dst="$EXPERIMENTS_DIR/$proj"
  if [[ -d "$src" ]]; then
    echo "=== Syncing $proj -> $dst (excluding target/, .venv/, node_modules/, __pycache__/, .git/) ==="
    mkdir -p "$dst"
    rsync -a --delete --delete-excluded \
      --exclude 'target/' \
      --exclude '.venv/' \
      --exclude 'node_modules/' \
      --exclude '__pycache__/' \
      --exclude '.git/' \
      "$src/" "$dst/"
  else
    echo "[SKIP] $src does not exist yet"
  fi
}

process_run() {
  local cast_name="${1:?Usage: setup-workspaces.sh process-run <cast-name.cast> <project-name>}"
  local proj="${2:?Usage: setup-workspaces.sh process-run <cast-name.cast> <project-name>}"
  local explorer="$PRESENTATION_ROOT/.agents/skills/agy-brain-explorer/scripts/agy-brain-explorer.py"

  process_single_cast "$cast_name"

  echo ""
  echo "=== Discovering & Exporting Agy Session Traces: $proj ==="
  uv run "$explorer" --project "$proj"
  rm -rf "$TRACES_DIR/$proj"
  mkdir -p "$TRACES_DIR/$proj"
  uv run "$explorer" export --project "$proj" -o "$TRACES_DIR/$proj"
  echo ""
  echo "Exported Markdown Session Traces in $TRACES_DIR/$proj:"
  ls -la "$TRACES_DIR/$proj"

  echo ""
  sync_single_experiment "$proj"
}

postprocess_casts() {
  for cast_name in 01-baseline.cast 02-skilled.cast 03-production.cast; do
    process_single_cast "$cast_name" || true
  done
}

extract_traces() {
  mkdir -p "$TRACES_DIR"
  local explorer="$PRESENTATION_ROOT/.agents/skills/agy-brain-explorer/scripts/agy-brain-explorer.py"
  echo "=== Discovering Sessions Across Experiment Projects ==="
  uv run "$explorer" --by-project
  for proj in audio_inventory_baseline audio_inventory_skilled audio_inventory_production; do
    rm -rf "$TRACES_DIR/$proj"
    mkdir -p "$TRACES_DIR/$proj"
    uv run "$explorer" export --project "$proj" -o "$TRACES_DIR/$proj" || true
  done
  echo "[OK] Exported Markdown Session Traces to $TRACES_DIR"
}

sync_experiments() {
  local target_proj="${1:-}"
  if [[ -n "$target_proj" ]]; then
    sync_single_experiment "$target_proj"
  else
    for proj in audio_inventory_baseline audio_inventory_skilled audio_inventory_production; do
      sync_single_experiment "$proj"
    done
  fi
}

show_status() {
  echo "=== Live Experiment Workspaces ($DEMOS_ROOT) ==="
  for dir in "$BASELINE_DIR" "$SKILLED_DIR" "$PRODUCTION_DIR"; do
    if [[ -d "$dir" ]]; then
      local skill_count=0
      [[ -d "$dir/.agents/skills" ]] && skill_count=$(ls -1 "$dir/.agents/skills" | wc -l | tr -d ' ')
      echo "  [READY] $(basename "$dir") (skills: $skill_count)"
    else
      echo "  [MISSING] $(basename "$dir")"
    fi
  done
  echo "=== Synced Repository Workspaces ($EXPERIMENTS_DIR) ==="
  for proj in audio_inventory_baseline audio_inventory_skilled audio_inventory_production; do
    local dir="$EXPERIMENTS_DIR/$proj"
    if [[ -d "$dir" ]]; then
      local skill_count=0
      [[ -d "$dir/.agents/skills" ]] && skill_count=$(ls -1 "$dir/.agents/skills" | wc -l | tr -d ' ')
      echo "  [READY] $proj (skills: $skill_count)"
    else
      echo "  [MISSING] $proj"
    fi
  done
  echo "=== Terminal Casts ($RECORDINGS_RAW -> $RECORDINGS_FINAL) ==="
  for cast_name in 01-baseline.cast 02-skilled.cast 03-production.cast; do
    local raw_state="missing"
    local final_state="missing"
    [[ -f "$RECORDINGS_RAW/$cast_name" ]] && raw_state="present"
    if [[ -f "$RECORDINGS_FINAL/$cast_name" ]]; then
      local marker_count
      marker_count=$(grep -c '"m"' "$RECORDINGS_FINAL/$cast_name" || true)
      final_state="present (markers: $marker_count)"
    fi
    echo "  $cast_name: raw=$raw_state | final=$final_state"
  done
  echo "=== Session Traces ($TRACES_DIR) ==="
  for proj in audio_inventory_baseline audio_inventory_skilled audio_inventory_production; do
    if [[ -d "$TRACES_DIR/$proj" ]]; then
      local trace_count
      trace_count=$(find "$TRACES_DIR/$proj" -maxdepth 1 -name "[0-9][0-9]-*.md" | wc -l | tr -d ' ')
      echo "  [READY] $proj (sessions: $trace_count)"
    else
      echo "  [MISSING] $proj"
    fi
  done
  echo "=== Cast-Player HTML Harnesses ($PROTOTYPES_DIR) ==="
  for html_name in 01-baseline-cast-player.html 02-skilled-cast-player.html 03-production-cast-player.html; do
    local html_path="$PROTOTYPES_DIR/$html_name"
    if [[ -f "$html_path" ]]; then
      echo "  [READY] $html_name"
    else
      echo "  [MISSING] $html_name"
    fi
  done
}

cmd="${1:-status}"
case "$cmd" in
  process-run)
    shift
    process_run "$@"
    ;;
  postprocess-casts)
    postprocess_casts
    ;;
  extract-traces)
    extract_traces
    ;;
  sync-experiments)
    shift
    sync_experiments "$@"
    ;;
  status)
    show_status
    ;;
  *)
    usage
    exit 1
    ;;
esac

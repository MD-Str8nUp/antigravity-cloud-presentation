# 08: Unified Data Directory `.zip` Backup & Restore

**What to build:** End-to-end backup and restore of the unified local data directory (`inventory.db` and `attachments/`), allowing the builder to download a consistent `.zip` archive snapshot in one click from the web UI (`GET /settings/backup.zip`) and restore an existing `.zip` backup archive into the data directory via the UI or CLI.

**Blocked by:** 05: Active Bench Checklist, Build Attachments & Bench Log Entries

**Status:** resolved

- [x] Clicking **Download Backup (`.zip`)** (`GET /settings/backup.zip`) streams a valid `.zip` archive containing a consistent snapshot of the SQLite database (`inventory.db`) and all uploaded build attachment files under `attachments/<kit_build_id>/`.
- [x] Restoring from a `.zip` backup archive (via the settings restore upload or CLI restore command) validates the archive structure and restores both `inventory.db` and `attachments/` into the configured data directory (`--data-dir` / `AUDIO_INVENTORY_DATA_DIR`).
- [x] Integration and smoke tests (`tower::ServiceExt::oneshot` + temporary data directories) verify that creating **Components**, **Storage Locations**, **Stock Lots**, **Kit Builds**, **BOM Lines**, **Bench Log Entries**, and attachment files, exporting `GET /settings/backup.zip`, and restoring into a fresh data directory preserves all SQLite records and attachment files byte-for-byte.

use crate::db::{Db, init_data_dir};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read, Write};
use std::path::{Component as PathComponent, Path, PathBuf};
use turso::{Value, params_from_iter};
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

struct ScopedTempDir {
    path: PathBuf,
}

impl ScopedTempDir {
    fn new(prefix: &str) -> anyhow::Result<Self> {
        let path = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ScopedTempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

const ALL_TABLES_IN_DEPENDENCY_ORDER: &[&str] = &[
    "_schema_migrations",
    "storage_locations",
    "components",
    "stock_lots",
    "kit_builds",
    "bom_lines",
    "bench_log_entries",
    "attachments",
    "import_presets",
    "import_batches",
    "import_rows",
    "shopping_list_overrides",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreSummary {
    pub database_bytes: usize,
    pub attachment_files_restored: usize,
}

use crate::attachments::resolve_effective_data_dir_path as resolve_effective_data_dir;

pub async fn checkpoint_database(db: &Db) -> anyhow::Result<()> {
    let conn = db.connect().await?;
    if let Ok(mut rows) = conn.query("PRAGMA wal_checkpoint(TRUNCATE);", ()).await {
        while rows.next().await?.is_some() {}
    } else {
        let _ = conn.execute("PRAGMA wal_checkpoint(TRUNCATE);", ()).await;
    }
    Ok(())
}

pub async fn copy_all_tables(src_db: &Db, dst_db: &Db) -> anyhow::Result<()> {
    let src_conn = src_db.connect().await?;
    let mut dst_conn = dst_db.connect().await?;
    dst_conn.execute("PRAGMA foreign_keys = OFF;", ()).await?;

    let tx = dst_conn.transaction().await?;
    for &table in ALL_TABLES_IN_DEPENDENCY_ORDER.iter().rev() {
        if table == "_schema_migrations" {
            continue;
        }
        tx.execute(&format!("DELETE FROM {table}"), ()).await?;
    }

    for &table in ALL_TABLES_IN_DEPENDENCY_ORDER {
        if table == "_schema_migrations" {
            continue;
        }
        let mut col_rows = src_conn
            .query(&format!("PRAGMA table_info({table});"), ())
            .await?;
        let mut col_names = Vec::new();
        while let Some(r) = col_rows.next().await? {
            col_names.push(r.get::<String>(1)?);
        }
        if col_names.is_empty() {
            continue;
        }

        let cols_csv = col_names.join(", ");
        let placeholders: Vec<String> = (1..=col_names.len()).map(|i| format!("?{i}")).collect();
        let insert_sql = format!(
            "INSERT INTO {table} ({cols_csv}) VALUES ({})",
            placeholders.join(", ")
        );

        let select_sql = format!("SELECT {cols_csv} FROM {table}");
        let mut rows = src_conn.query(&select_sql, ()).await?;
        while let Some(row) = rows.next().await? {
            let mut vals: Vec<Value> = Vec::with_capacity(col_names.len());
            for idx in 0..col_names.len() {
                vals.push(row.get_value(idx)?);
            }
            tx.execute(&insert_sql, params_from_iter(vals)).await?;
        }
    }

    tx.commit().await?;
    dst_conn.execute("PRAGMA foreign_keys = ON;", ()).await?;
    Ok(())
}

async fn snapshot_database_bytes(db: &Db, data_dir: &Path) -> anyhow::Result<Vec<u8>> {
    let db_file = data_dir.join("inventory.db");
    let wal_file = data_dir.join("inventory.db-wal");

    if data_dir != Path::new(":memory:") && db_file.is_file() {
        let _ = checkpoint_database(db).await;
        let wal_len = std::fs::metadata(&wal_file).map(|m| m.len()).unwrap_or(0);
        let db_len = std::fs::metadata(&db_file).map(|m| m.len()).unwrap_or(0);
        if wal_len == 0 && db_len > 0 {
            return Ok(std::fs::read(&db_file)?);
        }
    }

    // Either :memory: or WAL still has uncheckpointed frames: materialize a clean standalone snapshot
    let tmp = ScopedTempDir::new("audio-inv-snap")?;
    let snap_dir = tmp.path().join("snapshot");
    let snap_db = init_data_dir(&snap_dir).await?;
    copy_all_tables(db, &snap_db).await?;
    let _ = checkpoint_database(&snap_db).await;
    drop(snap_db);

    let snap_db_file = snap_dir.join("inventory.db");
    Ok(std::fs::read(&snap_db_file)?)
}

fn collect_attachment_files(
    base_dir: &Path,
    current_dir: &Path,
    out: &mut Vec<(String, PathBuf)>,
) -> anyhow::Result<()> {
    if !current_dir.exists() {
        return Ok(());
    }
    let mut entries: Vec<_> = std::fs::read_dir(current_dir)?
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_attachment_files(base_dir, &path, out)?;
        } else if file_type.is_file() {
            let rel = path
                .strip_prefix(base_dir)
                .map_err(|_| anyhow::anyhow!("Attachment path outside base_dir"))?;
            let rel_slash = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            out.push((rel_slash, path));
        }
    }
    Ok(())
}

pub async fn create_backup_zip_bytes(db: &Db, data_dir: &Path) -> anyhow::Result<Vec<u8>> {
    let db_bytes = snapshot_database_bytes(db, data_dir).await?;
    let effective_dir = resolve_effective_data_dir(data_dir);
    let attachments_dir = effective_dir.join("attachments");

    let mut attachment_files = Vec::new();
    collect_attachment_files(&effective_dir, &attachments_dir, &mut attachment_files)?;

    // Also include any attachment files referenced in the `attachments` table if stored under db.data_dir()
    if effective_dir != resolve_effective_data_dir(db.data_dir()) {
        let alt_base = resolve_effective_data_dir(db.data_dir());
        let alt_atts = alt_base.join("attachments");
        let mut alt_files = Vec::new();
        collect_attachment_files(&alt_base, &alt_atts, &mut alt_files)?;
        for (rel, path) in alt_files {
            if !attachment_files.iter().any(|(r, _)| r == &rel) {
                attachment_files.push((rel, path));
            }
        }
    }
    attachment_files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buf);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644);

        zip.start_file("inventory.db", options)?;
        zip.write_all(&db_bytes)?;

        for (rel_path, fs_path) in attachment_files {
            let bytes = std::fs::read(&fs_path)?;
            zip.start_file(rel_path, options)?;
            zip.write_all(&bytes)?;
        }

        zip.finish()?;
    }

    Ok(buf.into_inner())
}

pub async fn create_backup_zip(db: &Db) -> anyhow::Result<Vec<u8>> {
    create_backup_zip_bytes(db, db.data_dir()).await
}

fn validate_archive_entry_path(name: &str) -> anyhow::Result<Option<PathBuf>> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        anyhow::bail!("Backup archive contains an empty entry name");
    }
    if trimmed.starts_with('/') || trimmed.starts_with('\\') || trimmed.contains('\\') {
        anyhow::bail!("Invalid path '{}' in backup archive: absolute or backslash paths are not allowed", name);
    }

    let path = Path::new(trimmed);
    let mut normalized = PathBuf::new();
    for comp in path.components() {
        match comp {
            PathComponent::Normal(part) => {
                let s = part
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("Non-UTF-8 path in backup archive"))?;
                if s.is_empty() || s == "." || s == ".." {
                    anyhow::bail!("Path traversal component in backup archive entry '{}'", name);
                }
                normalized.push(s);
            }
            _ => {
                anyhow::bail!(
                    "Invalid path component in backup archive entry '{}': path traversal is not allowed",
                    name
                );
            }
        }
    }

    if trimmed.ends_with('/') {
        // Directory entry: allow `attachments/` or subdirectories under `attachments/`
        let comps: Vec<_> = normalized
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        if comps.first().map(String::as_str) == Some("attachments") {
            return Ok(None);
        }
        anyhow::bail!(
            "Unexpected directory entry '{}' in backup archive",
            name
        );
    }

    let comps: Vec<_> = normalized
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();

    if comps.len() == 1 && comps[0] == "inventory.db" {
        return Ok(Some(normalized));
    }

    if comps.len() >= 2 && comps[0] == "attachments" {
        return Ok(Some(normalized));
    }

    anyhow::bail!(
        "Unexpected file '{}' in backup archive (only 'inventory.db' and 'attachments/...' are allowed)",
        name
    );
}

pub async fn restore_backup_zip_bytes(
    zip_bytes: &[u8],
    target_data_dir: &Path,
) -> anyhow::Result<RestoreSummary> {
    if zip_bytes.is_empty() {
        anyhow::bail!("Backup archive is empty");
    }

    let mut archive = ZipArchive::new(Cursor::new(zip_bytes))
        .map_err(|e| anyhow::anyhow!("Invalid .zip backup archive: {e}"))?;

    let mut db_bytes: Option<Vec<u8>> = None;
    let mut extracted_attachments: Vec<(PathBuf, Vec<u8>)> = Vec::new();

    for idx in 0..archive.len() {
        let mut entry = archive
            .by_index(idx)
            .map_err(|e| anyhow::anyhow!("Failed to read zip entry #{idx}: {e}"))?;
        let raw_name = entry.name().to_string();

        if entry.is_dir() || raw_name.ends_with('/') {
            let _ = validate_archive_entry_path(&raw_name)?;
            continue;
        }

        let Some(rel_path) = validate_archive_entry_path(&raw_name)? else {
            continue;
        };

        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;

        if rel_path == Path::new("inventory.db") {
            if db_bytes.is_some() {
                anyhow::bail!("Backup archive contains duplicate 'inventory.db' entries");
            }
            db_bytes = Some(buf);
        } else {
            extracted_attachments.push((rel_path, buf));
        }
    }

    let db_bytes = db_bytes
        .ok_or_else(|| anyhow::anyhow!("Backup archive is missing required 'inventory.db'"))?;

    if db_bytes.len() < 16 || &db_bytes[0..16] != b"SQLite format 3\0" {
        anyhow::bail!("'inventory.db' in backup archive is not a valid SQLite database file");
    }

    // Validate the restored SQLite database in a temporary staging directory before touching target_data_dir
    let staging_tmp = ScopedTempDir::new("audio-inv-stage")?;
    let staging_dir = staging_tmp.path().join("staging");
    std::fs::create_dir_all(&staging_dir)?;
    let staging_db_path = staging_dir.join("inventory.db");
    std::fs::write(&staging_db_path, &db_bytes)?;

    let staging_db = init_data_dir(&staging_dir)
        .await
        .map_err(|e| anyhow::anyhow!("Backup 'inventory.db' failed schema validation: {e}"))?;
    {
        let conn = staging_db.connect().await?;
        for required_table in ["components", "storage_locations", "stock_lots", "kit_builds", "bom_lines"] {
            let mut rows = conn
                .query(
                    "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    (required_table,),
                )
                .await?;
            if rows.next().await?.is_none() {
                anyhow::bail!(
                    "Backup 'inventory.db' is missing required table '{required_table}'"
                );
            }
        }
    }
    let _ = checkpoint_database(&staging_db).await;
    drop(staging_db);

    let effective_target = resolve_effective_data_dir(target_data_dir);
    let target_attachments_dir = effective_target.join("attachments");
    std::fs::create_dir_all(&effective_target)?;

    // Remove stale WAL/SHM files before writing restored inventory.db
    let wal_path = effective_target.join("inventory.db-wal");
    let shm_path = effective_target.join("inventory.db-shm");
    if wal_path.exists() {
        let _ = std::fs::remove_file(&wal_path);
    }
    if shm_path.exists() {
        let _ = std::fs::remove_file(&shm_path);
    }

    let final_db_bytes = std::fs::read(&staging_db_path)?;
    let target_db_path = effective_target.join("inventory.db");
    std::fs::write(&target_db_path, &final_db_bytes)?;

    // Replace attachments directory cleanly
    if target_attachments_dir.exists() {
        std::fs::remove_dir_all(&target_attachments_dir)?;
    }
    std::fs::create_dir_all(&target_attachments_dir)?;

    let mut attachment_files_restored = 0usize;
    for (rel_path, bytes) in &extracted_attachments {
        let dest_path = effective_target.join(rel_path);
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dest_path, bytes)?;
        attachment_files_restored += 1;
    }

    Ok(RestoreSummary {
        database_bytes: final_db_bytes.len(),
        attachment_files_restored,
    })
}

pub async fn restore_backup_zip_into_db(
    db: &Db,
    zip_bytes: &[u8],
) -> anyhow::Result<RestoreSummary> {
    let staging_tmp = ScopedTempDir::new("audio-inv-restore")?;
    let staging_dir = staging_tmp.path().join("restore_staging");
    let summary = restore_backup_zip_bytes(zip_bytes, &staging_dir).await?;

    let staging_db = init_data_dir(&staging_dir).await?;
    copy_all_tables(&staging_db, db).await?;
    let _ = checkpoint_database(db).await;
    drop(staging_db);

    // Replace attachments directory in the active db's effective data_dir
    let effective_target = resolve_effective_data_dir(db.data_dir());
    let target_attachments_dir = effective_target.join("attachments");
    if target_attachments_dir.exists() {
        std::fs::remove_dir_all(&target_attachments_dir)?;
    }
    std::fs::create_dir_all(&target_attachments_dir)?;

    let staging_attachments_dir = staging_dir.join("attachments");
    let mut attachment_files = Vec::new();
    collect_attachment_files(&staging_dir, &staging_attachments_dir, &mut attachment_files)?;
    for (rel_str, src_path) in attachment_files {
        let dest_path = effective_target.join(&rel_str);
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&src_path, &dest_path)?;
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builds::{
        CreateBenchLogEntry, CreateBomLine, CreateKitBuild, add_bom_line,
        add_build_attachment_file, add_build_external_link, create_bench_log_entry,
        create_kit_build, get_kit_build, list_bench_log_entries, list_build_attachments,
        read_build_attachment_bytes, toggle_bom_line_designator, transition_bom_line_status,
    };
    use crate::catalog::{
        CreateComponent, CreateInitialStockLot, CreateStorageLocation, create_component,
        create_storage_location, list_components, list_storage_locations,
    };
    use crate::db::init_data_dir;
    use std::io::Cursor;
    use zip::ZipArchive;

    #[tokio::test]
    async fn backup_and_restore_preserves_all_sqlite_records_and_attachments_byte_for_byte() {
        let src_tmp = tempfile::tempdir().expect("src tempdir");
        let src_dir = src_tmp.path().join("src_data");
        let src_db = init_data_dir(&src_dir).await.expect("init src db");

        // 1. Storage Locations
        let crate_a = create_storage_location(
            &src_db,
            CreateStorageLocation {
                name: "Crate A".into(),
                kind: "Crate".into(),
                parent_id: None,
                notes: "Main passive storage".into(),
            },
        )
        .await
        .unwrap();
        let binder = create_storage_location(
            &src_db,
            CreateStorageLocation {
                name: "Resistor Binder".into(),
                kind: "Binder".into(),
                parent_id: Some(crate_a.id.clone()),
                notes: "1% metal film pages".into(),
            },
        )
        .await
        .unwrap();

        // 2. Components & Stock Lots
        let comp_r = create_component(
            &src_db,
            CreateComponent {
                name: "4.7kΩ 1% 1/4W Metal Film Resistor".into(),
                category: "Resistors".into(),
                subtype: "Fixed".into(),
                package_or_mounting: "Through-Hole".into(),
                raw_spec_text: "4k7 1% 1/4W Metal Film".into(),
                canonical_value_display: None,
                resistance_ohms: None,
                capacitance_pf: None,
                inductance_uh: None,
                voltage_rating_volts: None,
                low_stock_threshold: 10,
                datasheet_url: "https://example.com/4k7.pdf".into(),
                notes: "Yageo MFR".into(),
                specs_json: None,
                initial_lot: Some(CreateInitialStockLot {
                    location_id: Some(binder.id.clone()),
                    quantity: 25,
                    supplier: "Tayda".into(),
                    supplier_part_no: "A-2214".into(),
                    lot_notes: "Bag #12".into(),
                }),
            },
        )
        .await
        .unwrap();

        // 3. Kit Build, BOM Lines, Fulfillment transitions & Designator checklist
        let build = create_kit_build(
            &src_db,
            CreateKitBuild {
                name: "Klon Centaur Overdrive".into(),
                project_type: "Guitar Pedal".into(),
                status: Some("On the Bench (In Progress)".into()),
                location_id: Some(crate_a.id.clone()),
                description: "Transparent overdrive build".into(),
            },
        )
        .await
        .unwrap();

        let bom_line = add_bom_line(
            &src_db,
            CreateBomLine {
                kit_build_id: build.id.clone(),
                raw_spec_text: "4.7k".into(),
                category: Some("Resistors".into()),
                package_or_mounting: Some("Through-Hole".into()),
                quantity_required: 2,
                reference_designators: "R1, R2".into(),
                included_in_kit: false,
                component_id: Some(comp_r.id.clone()),
                substitution_note: "".into(),
                notes: "Input biasing".into(),
            },
        )
        .await
        .unwrap();

        transition_bom_line_status(
            &src_db,
            &bom_line.id,
            "Picked",
            Some(&comp_r.lots[0].id),
            Some(2),
        )
        .await
        .unwrap();
        toggle_bom_line_designator(&src_db, &bom_line.id, "R1")
            .await
            .unwrap();

        // 4. Bench Log Entry & Build Attachments
        let log_entry = create_bench_log_entry(
            &src_db,
            CreateBenchLogEntry {
                kit_build_id: build.id.clone(),
                entry_type: "Measurement / Voltage Reading".into(),
                body: "IC1 Pin 8 = +9.14V, Charge Pump = +17.8V".into(),
            },
        )
        .await
        .unwrap();

        let schematic_bytes = b"%PDF-1.4\n%Klon Centaur Schematic Binary Payload \x00\xFF\xFE\n";
        let att_pdf = add_build_attachment_file(
            &src_db,
            &build.id,
            None,
            "Schematic PDF",
            "klon_schematic.pdf",
            "application/pdf",
            schematic_bytes,
        )
        .await
        .unwrap();

        let photo_bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR-bench-photo-bytes";
        let att_photo = add_build_attachment_file(
            &src_db,
            &build.id,
            Some(&log_entry.id),
            "Photos",
            "bench_wiring.png",
            "image/png",
            photo_bytes,
        )
        .await
        .unwrap();

        add_build_external_link(
            &src_db,
            &build.id,
            Some("External URL"),
            "Aion FX Refractor Build Guide",
            "https://aionfx.com/project/refractor/",
        )
        .await
        .unwrap();

        // 5. Create Backup (.zip)
        let zip_bytes = create_backup_zip_bytes(&src_db, src_db.data_dir())
            .await
            .expect("create backup zip");
        assert!(!zip_bytes.is_empty());

        // Inspect archive structure
        let mut archive = ZipArchive::new(Cursor::new(&zip_bytes)).expect("valid zip archive");
        let mut entry_names = Vec::new();
        for i in 0..archive.len() {
            let file = archive.by_index(i).unwrap();
            entry_names.push(file.name().to_string());
        }
        assert!(
            entry_names.contains(&"inventory.db".to_string()),
            "zip must contain inventory.db, got: {entry_names:?}"
        );
        assert!(
            entry_names.contains(&att_pdf.relative_path),
            "zip must contain schematic attachment {}, got: {entry_names:?}",
            att_pdf.relative_path
        );
        assert!(
            entry_names.contains(&att_photo.relative_path),
            "zip must contain photo attachment {}, got: {entry_names:?}",
            att_photo.relative_path
        );

        // 6. Restore into a fresh data directory
        let dst_tmp = tempfile::tempdir().expect("dst tempdir");
        let dst_dir = dst_tmp.path().join("restored_data");
        let summary = restore_backup_zip_bytes(&zip_bytes, &dst_dir)
            .await
            .expect("restore backup zip");
        assert!(summary.database_bytes > 0);
        assert_eq!(summary.attachment_files_restored, 2);

        let dst_db = init_data_dir(&dst_dir).await.expect("open restored db");

        // Verify Storage Locations
        let restored_locs = list_storage_locations(&dst_db).await.unwrap();
        assert_eq!(restored_locs.len(), 2);
        assert!(
            restored_locs
                .iter()
                .any(|l| l.breadcrumb_path == "Crate A > Resistor Binder")
        );

        // Verify Components & Stock Lots (25 initial - 2 picked = 23 on hand)
        let restored_comps = list_components(&dst_db).await.unwrap();
        assert_eq!(restored_comps.len(), 1);
        assert_eq!(restored_comps[0].id, comp_r.id);
        assert_eq!(restored_comps[0].total_on_hand, 23);
        assert_eq!(restored_comps[0].lots.len(), 1);
        assert_eq!(restored_comps[0].lots[0].quantity, 23);
        assert_eq!(restored_comps[0].lots[0].supplier_part_no, "A-2214");

        // Verify Kit Build & BOM Lines
        let restored_build = get_kit_build(&dst_db, &build.id)
            .await
            .unwrap()
            .expect("restored build exists");
        assert_eq!(restored_build.name, "Klon Centaur Overdrive");
        assert_eq!(restored_build.bom_lines.len(), 1);
        assert_eq!(restored_build.bom_lines[0].fulfillment_status, "Picked");
        assert_eq!(restored_build.bom_lines[0].quantity_allocated, 2);
        assert_eq!(
            restored_build.bom_lines[0].installed_designators,
            vec!["R1".to_string()]
        );

        // Verify Bench Log Entries
        let restored_logs = list_bench_log_entries(&dst_db, &build.id).await.unwrap();
        assert_eq!(restored_logs.len(), 1);
        assert_eq!(
            restored_logs[0].body,
            "IC1 Pin 8 = +9.14V, Charge Pump = +17.8V"
        );
        assert_eq!(restored_logs[0].attachments.len(), 1);
        assert_eq!(restored_logs[0].attachments[0].id, att_photo.id);

        // Verify Attachments metadata & byte-for-byte file contents
        let restored_atts = list_build_attachments(&dst_db, &build.id).await.unwrap();
        assert_eq!(restored_atts.len(), 3);
        let restored_pdf = restored_atts.iter().find(|a| a.id == att_pdf.id).unwrap();
        let restored_pdf_bytes = read_build_attachment_bytes(&dst_db, restored_pdf).unwrap();
        assert_eq!(restored_pdf_bytes, schematic_bytes);

        let restored_photo = restored_atts.iter().find(|a| a.id == att_photo.id).unwrap();
        let restored_photo_bytes = read_build_attachment_bytes(&dst_db, restored_photo).unwrap();
        assert_eq!(restored_photo_bytes, photo_bytes);
    }

    #[tokio::test]
    async fn restore_validates_archive_structure_and_rejects_invalid_or_unsafe_zips_without_clobbering_existing_data()
     {
        let tmp = tempfile::tempdir().expect("tempdir");
        let target_dir = tmp.path().join("existing_data");
        let existing_db = init_data_dir(&target_dir).await.expect("init existing db");

        let loc = create_storage_location(
            &existing_db,
            CreateStorageLocation {
                name: "Safe Drawer".into(),
                kind: "Drawer".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        // 1. Corrupt non-zip bytes
        let err_non_zip = restore_backup_zip_bytes(b"not a valid zip file", &target_dir)
            .await
            .expect_err("should reject non-zip");
        assert!(err_non_zip.to_string().contains("Invalid .zip"));

        // 2. Zip missing inventory.db
        let mut no_db_buf = Cursor::new(Vec::new());
        {
            let mut z = ZipWriter::new(&mut no_db_buf);
            let opts = SimpleFileOptions::default();
            z.start_file("attachments/kb-1/readme.txt", opts).unwrap();
            z.write_all(b"hello").unwrap();
            z.finish().unwrap();
        }
        let err_no_db = restore_backup_zip_bytes(&no_db_buf.into_inner(), &target_dir)
            .await
            .expect_err("should reject zip without inventory.db");
        assert!(err_no_db.to_string().contains("missing required 'inventory.db'"));

        // 3. Zip with invalid SQLite file in inventory.db
        let mut bad_db_buf = Cursor::new(Vec::new());
        {
            let mut z = ZipWriter::new(&mut bad_db_buf);
            let opts = SimpleFileOptions::default();
            z.start_file("inventory.db", opts).unwrap();
            z.write_all(b"corrupt database payload").unwrap();
            z.finish().unwrap();
        }
        let err_bad_db = restore_backup_zip_bytes(&bad_db_buf.into_inner(), &target_dir)
            .await
            .expect_err("should reject invalid sqlite header");
        assert!(err_bad_db.to_string().contains("not a valid SQLite database"));

        // 4. Zip with path traversal entry
        let valid_zip = create_backup_zip(&existing_db).await.unwrap();
        let mut valid_archive = ZipArchive::new(Cursor::new(&valid_zip)).unwrap();
        let mut valid_db_bytes = Vec::new();
        valid_archive
            .by_name("inventory.db")
            .unwrap()
            .read_to_end(&mut valid_db_bytes)
            .unwrap();

        let mut traversal_buf = Cursor::new(Vec::new());
        {
            let mut z = ZipWriter::new(&mut traversal_buf);
            let opts = SimpleFileOptions::default();
            z.start_file("inventory.db", opts).unwrap();
            z.write_all(&valid_db_bytes).unwrap();
            z.start_file("attachments/../../escape.txt", opts).unwrap();
            z.write_all(b"pwn").unwrap();
            z.finish().unwrap();
        }
        let err_traversal = restore_backup_zip_bytes(&traversal_buf.into_inner(), &target_dir)
            .await
            .expect_err("should reject path traversal");
        assert!(err_traversal.to_string().contains("path traversal"));

        // Verify existing data in target_dir is completely intact
        let locs_after = list_storage_locations(&existing_db).await.unwrap();
        assert_eq!(locs_after.len(), 1);
        assert_eq!(locs_after[0].id, loc.id);
    }
}

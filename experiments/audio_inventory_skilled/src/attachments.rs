use crate::db::Db;
use anyhow::Context;
use std::path::{Path, PathBuf};

pub use crate::backup::{
    RestoreSummary, create_backup_zip, create_backup_zip_bytes, restore_backup_zip_bytes,
    restore_backup_zip_into_db,
};
pub use crate::builds::{
    AttachmentKind, BuildAttachment, add_build_attachment_file, add_build_attachment_file_in_dir,
    add_build_external_link, get_build_attachment, list_build_attachments,
    read_build_attachment_bytes, read_build_attachment_bytes_in_dir,
};

/// Resolves the effective on-disk data directory for a path, falling back
/// to a process-scoped temporary directory when `data_dir` is `:memory:`.
pub fn resolve_effective_data_dir_path(data_dir: &Path) -> PathBuf {
    if data_dir == Path::new(":memory:") {
        std::env::temp_dir().join("audio-inventory-memory-data")
    } else {
        data_dir.to_path_buf()
    }
}

/// Resolves the effective on-disk data directory for a database handle, falling back
/// to a process-scoped temporary directory when `db` is an in-memory test database (`:memory:`).
pub fn resolve_effective_data_dir(db: &Db) -> PathBuf {
    resolve_effective_data_dir_path(db.data_dir())
}

/// Sanitizes an uploaded filename so it is safe to store inside `attachments/<kit_build_id>/`.
pub fn sanitize_attachment_filename(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "attachment.bin".to_string();
    }
    let mut out = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
            out.push(ch);
        } else if ch.is_whitespace() {
            out.push('_');
        }
    }
    let cleaned = out.trim_matches('.').to_string();
    if cleaned.is_empty() {
        "attachment.bin".to_string()
    } else {
        cleaned
    }
}

/// Writes attachment bytes to `<data_dir>/attachments/<kit_build_id>/<attachment_id>-<sanitized_filename>`
/// and returns the normalized relative path (`attachments/<kit_build_id>/<stored_filename>`).
pub fn write_attachment_file_bytes(
    data_dir: &Path,
    kit_build_id: &str,
    attachment_id: &str,
    original_filename: &str,
    bytes: &[u8],
) -> anyhow::Result<String> {
    let sanitized = sanitize_attachment_filename(original_filename);
    let stored_filename = format!("{attachment_id}-{sanitized}");
    let rel_dir = Path::new("attachments").join(kit_build_id);
    let rel_path = rel_dir.join(&stored_filename);
    let relative_path_str = format!("attachments/{kit_build_id}/{stored_filename}");

    let full_dir = data_dir.join(&rel_dir);
    std::fs::create_dir_all(&full_dir).with_context(|| {
        format!(
            "Failed to create attachments directory {}",
            full_dir.display()
        )
    })?;
    let full_path = data_dir.join(&rel_path);
    std::fs::write(&full_path, bytes)
        .with_context(|| format!("Failed to write attachment file {}", full_path.display()))?;

    Ok(relative_path_str)
}

/// Reads attachment bytes from `<data_dir>/<relative_path>`.
pub fn read_attachment_file_bytes_in_dir(
    data_dir: &Path,
    relative_path: &str,
) -> anyhow::Result<Vec<u8>> {
    if relative_path.trim().is_empty() {
        anyhow::bail!("Attachment does not have a local file path");
    }
    let full_path = data_dir.join(relative_path);
    std::fs::read(&full_path)
        .with_context(|| format!("Failed to read attachment file {}", full_path.display()))
}

use audio_inventory::db::{
    init_data_dir, resolve_data_dir_from_env, resolve_restore_path_from_env,
    restore_backup_file_to_data_dir,
};
use audio_inventory::web::app_router;
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let data_dir = resolve_data_dir_from_env();
    if let Some(restore_zip_path) = resolve_restore_path_from_env() {
        let summary = restore_backup_file_to_data_dir(&restore_zip_path, &data_dir).await?;
        tracing::info!(
            data_dir = %data_dir.display(),
            backup_zip = %restore_zip_path.display(),
            database_bytes = summary.database_bytes,
            attachment_files_restored = summary.attachment_files_restored,
            "Restored unified data directory from backup .zip archive"
        );
        return Ok(());
    }

    let db = init_data_dir(&data_dir).await?;

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tracing::info!(
        data_dir = %db.data_dir().display(),
        addr = %addr,
        "Starting Audio Electronics Inventory & Kit Builds server"
    );

    let app = app_router(db);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

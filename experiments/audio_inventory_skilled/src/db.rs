use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use turso::{Builder, Connection, Database};

pub use crate::backup::{
    RestoreSummary, create_backup_zip, create_backup_zip_bytes, restore_backup_zip_bytes,
    restore_backup_zip_into_db,
};

const MIGRATIONS: &[(i64, &str, &str)] = &[(
    1,
    "001_initial_schema",
    include_str!("../migrations/001_initial_schema.sql"),
)];

#[derive(Clone)]
pub struct Db {
    db: Arc<RwLock<Database>>,
    data_dir: PathBuf,
}

impl Db {
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("inventory.db")
    }

    pub fn attachments_dir(&self) -> PathBuf {
        self.data_dir.join("attachments")
    }

    pub async fn connect(&self) -> anyhow::Result<Connection> {
        let conn = {
            let guard = self
                .db
                .read()
                .map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
            guard.connect()?
        };
        conn.execute("PRAGMA foreign_keys = ON;", ()).await?;
        Ok(conn)
    }

    pub async fn reload_from_disk(&self) -> anyhow::Result<()> {
        if self.data_dir == Path::new(":memory:") {
            return Ok(());
        }
        let db_path = self.db_path();
        let db_path_str = db_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 database path"))?;
        let new_database = Builder::new_local(db_path_str).build().await?;
        {
            let mut guard = self
                .db
                .write()
                .map_err(|_| anyhow::anyhow!("Database lock poisoned"))?;
            *guard = new_database;
        }
        run_migrations(self).await?;
        Ok(())
    }
}

pub fn resolve_data_dir(args: &[String], env_var: Option<&str>) -> PathBuf {
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        if arg == "--data-dir" {
            if let Some(val) = iter.next() {
                if !val.trim().is_empty() {
                    return PathBuf::from(val.trim());
                }
            }
        } else if let Some(val) = arg.strip_prefix("--data-dir=") {
            if !val.trim().is_empty() {
                return PathBuf::from(val.trim());
            }
        }
    }

    if let Some(env_val) = env_var {
        if !env_val.trim().is_empty() {
            return PathBuf::from(env_val.trim());
        }
    }

    PathBuf::from("./data")
}

pub fn resolve_data_dir_from_env() -> PathBuf {
    let args: Vec<String> = std::env::args().collect();
    let env_val = std::env::var("AUDIO_INVENTORY_DATA_DIR").ok();
    resolve_data_dir(&args, env_val.as_deref())
}

pub fn resolve_restore_path(args: &[String]) -> Option<PathBuf> {
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        if arg == "--restore" || arg == "--restore-zip" {
            if let Some(val) = iter.next() {
                if !val.trim().is_empty() {
                    return Some(PathBuf::from(val.trim()));
                }
            }
        } else if let Some(val) = arg
            .strip_prefix("--restore=")
            .or_else(|| arg.strip_prefix("--restore-zip="))
        {
            if !val.trim().is_empty() {
                return Some(PathBuf::from(val.trim()));
            }
        }
    }
    None
}

pub fn resolve_restore_path_from_env() -> Option<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    resolve_restore_path(&args)
}

pub async fn restore_backup_file_to_data_dir(
    zip_path: impl AsRef<Path>,
    target_data_dir: impl AsRef<Path>,
) -> anyhow::Result<RestoreSummary> {
    let zip_bytes = std::fs::read(zip_path.as_ref())?;
    restore_backup_zip_bytes(&zip_bytes, target_data_dir.as_ref()).await
}

pub async fn init_data_dir(data_dir: impl AsRef<Path>) -> anyhow::Result<Db> {
    let data_dir = data_dir.as_ref().to_path_buf();
    let attachments_dir = data_dir.join("attachments");
    std::fs::create_dir_all(&attachments_dir)?;

    let db_path = data_dir.join("inventory.db");
    let db_path_str = db_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 database path"))?;

    let database = Builder::new_local(db_path_str).build().await?;
    let db = Db {
        db: Arc::new(RwLock::new(database)),
        data_dir,
    };
    run_migrations(&db).await?;
    Ok(db)
}

pub async fn open_data_dir(data_dir: impl AsRef<Path>) -> anyhow::Result<Db> {
    init_data_dir(data_dir).await
}

pub async fn init_memory_db() -> anyhow::Result<Db> {
    let database = Builder::new_local(":memory:").build().await?;
    let db = Db {
        db: Arc::new(RwLock::new(database)),
        data_dir: PathBuf::from(":memory:"),
    };
    run_migrations(&db).await?;
    Ok(db)
}

pub async fn run_migrations(db: &Db) -> anyhow::Result<()> {
    let mut conn = db.connect().await?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
        (),
    )
    .await?;

    for &(version, name, sql) in MIGRATIONS {
        let mut rows = conn
            .query(
                "SELECT version FROM _schema_migrations WHERE version = ?1",
                (version,),
            )
            .await?;
        if rows.next().await?.is_some() {
            continue;
        }

        let tx = conn.transaction().await?;
        tx.execute_batch(sql).await?;
        tx.execute(
            "INSERT INTO _schema_migrations (version, name) VALUES (?1, ?2)",
            (version, name),
        )
        .await?;
        tx.commit().await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_data_dir_from_cli_flag_or_env_or_default() {
        let from_cli = resolve_data_dir(
            &[
                "audio_inventory".into(),
                "--data-dir".into(),
                "/tmp/custom-cli".into(),
            ],
            Some("/tmp/custom-env"),
        );
        assert_eq!(from_cli, PathBuf::from("/tmp/custom-cli"));

        let from_cli_eq = resolve_data_dir(
            &["audio_inventory".into(), "--data-dir=/tmp/custom-eq".into()],
            Some("/tmp/custom-env"),
        );
        assert_eq!(from_cli_eq, PathBuf::from("/tmp/custom-eq"));

        let from_env = resolve_data_dir(&["audio_inventory".into()], Some("/tmp/custom-env"));
        assert_eq!(from_env, PathBuf::from("/tmp/custom-env"));

        let default_dir = resolve_data_dir(&["audio_inventory".into()], None);
        assert_eq!(default_dir, PathBuf::from("./data"));
    }

    #[tokio::test]
    async fn resolves_restore_cli_flag_and_restores_backup_file_into_configured_data_dir() {
        let from_cli = resolve_restore_path(&[
            "audio_inventory".into(),
            "--data-dir".into(),
            "/tmp/custom-cli".into(),
            "--restore".into(),
            "/tmp/my-backup.zip".into(),
        ]);
        assert_eq!(from_cli, Some(PathBuf::from("/tmp/my-backup.zip")));

        let from_cli_eq = resolve_restore_path(&[
            "audio_inventory".into(),
            "--restore=/tmp/eq-backup.zip".into(),
        ]);
        assert_eq!(from_cli_eq, Some(PathBuf::from("/tmp/eq-backup.zip")));

        let none_flag = resolve_restore_path(&["audio_inventory".into()]);
        assert_eq!(none_flag, None);

        let tmp = tempfile::tempdir().expect("tempdir");
        let src_dir = tmp.path().join("src_data");
        let src_db = init_data_dir(&src_dir).await.unwrap();
        crate::catalog::create_storage_location(
            &src_db,
            crate::catalog::CreateStorageLocation {
                name: "CLI Restore Shelf".into(),
                kind: "Crate".into(),
                parent_id: None,
                notes: "".into(),
            },
        )
        .await
        .unwrap();

        let zip_bytes = create_backup_zip(&src_db).await.unwrap();
        let zip_file_path = tmp.path().join("workshop-backup.zip");
        std::fs::write(&zip_file_path, &zip_bytes).unwrap();

        let dst_dir = tmp.path().join("cli_restored_data");
        let summary = restore_backup_file_to_data_dir(&zip_file_path, &dst_dir)
            .await
            .expect("restore from file");
        assert!(summary.database_bytes > 0);

        let dst_db = init_data_dir(&dst_dir).await.unwrap();
        let locs = crate::catalog::list_storage_locations(&dst_db).await.unwrap();
        assert_eq!(locs.len(), 1);
        assert_eq!(locs[0].name, "CLI Restore Shelf");
    }

    #[tokio::test]
    async fn initializes_data_directory_and_applies_migrations_idempotently() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let data_dir = tmp.path().join("workshop_data");

        let db = init_data_dir(&data_dir).await.expect("first init");
        assert!(data_dir.exists(), "data dir should be created");
        assert!(
            db.attachments_dir().exists(),
            "attachments dir should be created"
        );
        assert!(
            data_dir.join("inventory.db").exists(),
            "inventory.db file should exist"
        );

        let conn = db.connect().await.expect("connect");

        // Verify foreign_keys pragma is enabled on connections
        let mut fk_rows = conn.query("PRAGMA foreign_keys;", ()).await.expect("pragma");
        let fk_row = fk_rows.next().await.expect("row").expect("some row");
        assert_eq!(fk_row.get::<i64>(0).expect("fk val"), 1);

        // Verify _schema_migrations recorded 001_initial_schema
        let mut mig_rows = conn
            .query("SELECT version, name FROM _schema_migrations", ())
            .await
            .expect("query migrations");
        let mig_row = mig_rows.next().await.expect("row").expect("some migration");
        assert_eq!(mig_row.get::<i64>(0).unwrap(), 1);
        assert_eq!(mig_row.get::<String>(1).unwrap(), "001_initial_schema");

        // Re-running init_data_dir on the same directory should be idempotent
        let db2 = init_data_dir(&data_dir).await.expect("second init");
        let conn2 = db2.connect().await.expect("connect 2");
        let mut count_rows = conn2
            .query("SELECT COUNT(*) FROM _schema_migrations", ())
            .await
            .expect("count");
        let count_row = count_rows.next().await.unwrap().unwrap();
        assert_eq!(count_row.get::<i64>(0).unwrap(), 1);
    }
}

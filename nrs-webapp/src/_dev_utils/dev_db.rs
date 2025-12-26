use std::time::Duration;

use include_dir::{File, include_dir};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::_dev_utils::{Db, PG_DEV_APP_URL, PG_DEV_POSTGRES_URL};

fn is_init_file(file: &File) -> bool {
    file.path().file_name().unwrap_or_default() == "00-recreate-db.sql"
}

pub async fn init_dev_db() -> Db {
    tracing::info!("{:<12} -- init_dev_db()", "FOR-DEV-ONLY");

    let migration_dir = include_dir!("$CARGO_MANIFEST_DIR/sql/dev_initial");
    let sql_files = migration_dir.files();

    let (mut init_sqls, mut app_sqls) = sql_files.partition::<Vec<_>, _>(|f| is_init_file(f));

    {
        let initial_db = new_db_pool(PG_DEV_POSTGRES_URL).await;
        init_sqls.sort_by_key(|f| f.path());
        for init_file in init_sqls {
            execute_sql(
                &initial_db,
                init_file
                    .contents_utf8()
                    .expect("Invalid UTF-8 in SQL file"),
                init_file.path().to_str().unwrap_or_default(),
            )
            .await;
        }
    }

    let app_db = new_db_pool(PG_DEV_APP_URL).await;
    app_sqls.sort_by_key(|f| f.path());
    for app_file in app_sqls {
        execute_sql(
            &app_db,
            app_file.contents_utf8().expect("Invalid UTF-8 in SQL file"),
            app_file.path().to_str().unwrap_or_default(),
        )
        .await;
    }

    app_db
}

async fn new_db_pool(url: &str) -> Db {
    tracing::info!(
        "{:<12} -- Creating DB pool for URL: {}",
        "FOR-DEV-ONLY",
        url
    );
    PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(500))
        .connect(url)
        .await
        .expect("Failed to create DB pool")
}

async fn execute_sql(pool: &Db, sql: &str, file_path: &str) {
    tracing::info!(
        "{:<12} -- Executing SQL from file: {}",
        "FOR-DEV-ONLY",
        file_path
    );

    // FIXME: avoid splitting by ';' naively, handle edge cases
    for cmd in sql.split(';') {
        let trimmed = cmd.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed)
                .execute(pool)
                .await
                .unwrap_or_else(|_| panic!("Failed to execute SQL command: {}", trimmed));
        }
    }
}

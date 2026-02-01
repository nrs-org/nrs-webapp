use sqlx::{Pool, Postgres};

use crate::_dev_utils::{dev_db::init_dev_db, seed::seed_dev_db};

mod dev_db;
mod seed;

type Db = Pool<Postgres>;
// NOTE: We use a non-standard port for the pg database in dev
// to avoid conflicts with any local pg instances.
const PG_DEV_POSTGRES_URL: &str = "postgres://postgres:password@localhost:3622/postgres";
const PG_DEV_APP_URL: &str =
    "postgres://nrs_webapp_user:dev_only_pwd@localhost:3622/nrs_webapp_dev";

pub async fn init_dev() {
    tracing::info!("{:<12} -- init_dev()", "FOR-DEV-ONLY");

    let _ = init_dev_db().await;
    seed_dev_db().await;
}

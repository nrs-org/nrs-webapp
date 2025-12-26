use crate::_dev_utils::{dev_db::init_dev_db, seed::seed_dev_db};

mod dev_db;
mod seed;

pub async fn init_dev() {
    tracing::info!("{:<12} -- init_dev()", "FOR-DEV-ONLY");

    init_dev_db().await;
    seed_dev_db().await;
}

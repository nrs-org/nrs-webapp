use std::sync::Mutex;

use nrs_webapp_core::{data::entry::types::idtype::EntryType, legacy_json::Bulk};
use uuid::Uuid;

use crate::{
    _dev_utils::Db,
    crypt::password_hash::PasswordHasher,
    model::{
        ModelManager,
        entry::{EntryBmc, EntryForCreate},
        user::{UserBmc, UserForCreate},
    },
};

pub async fn seed_dev_db(_db: &Db) {
    tracing::info!("{:<12} -- seed_dev_db()", "FOR-DEV-ONLY");

    let mut mm = ModelManager::new()
        .await
        .expect("Failed to create ModelManager");

    let _ = create_test_user(&mut mm).await;
    seed_entries(&mut mm).await;
}

static TEST_USER_ID: Mutex<Option<Uuid>> = Mutex::new(None);

/// Creates a deterministic test user in the database and returns its user ID.
///
/// The function creates a user with fixed credentials, hashes the password using the configured
/// PasswordHasher, persists the user via `UserBmc::create_user`, marks the user's email as
/// verified, stores the resulting ID in the global `TEST_USER_ID`, and returns the created ID.
///
/// `mm` — mutable reference to the ModelManager used for database operations.
///
/// # Returns
///
/// The newly created user's ID.
///
/// # Examples
///
/// ```rust,no_run
/// # async fn run_example() {
/// # // obtain or construct a ModelManager suitable for tests/dev
/// # let mut mm = /* ModelManager::new_for_tests().await */ unimplemented!();
/// let id = create_test_user(&mut mm).await;
/// println!("created test user id: {}", id);
/// # }
/// ```
async fn create_test_user(mm: &mut ModelManager) -> Uuid {
    tracing::info!("{:<12} -- create_test_user()", "FOR-DEV-ONLY");

    let username = "testuser".into();
    let email = "testuser@nrs.dev".into();
    let password_clear = "password123";

    let password_hash = PasswordHasher::get_from_config()
        .encrypt_password(password_clear)
        .expect("Unable to hash password");

    let id = UserBmc::create_user(
        mm,
        UserForCreate {
            username,
            email,
            password_hash,
        },
    )
    .await
    .expect("Unable to create test user");

    UserBmc::mark_email_verified(mm, id)
        .await
        .expect("Unable to verify test user email");

    tracing::info!(
        "{:<12} -- Created test user with ID: {}",
        "FOR-DEV-ONLY",
        id
    );

    TEST_USER_ID.lock().unwrap().replace(id);

    id
}

pub fn test_user_id() -> Uuid {
    TEST_USER_ID.lock().unwrap().clone().unwrap()
}

async fn seed_entries(mm: &mut ModelManager) {
    tracing::info!("{:<12} -- seed_entries()", "FOR-DEV-ONLY");

    let entries = include_str!("latest-pj-escape-bulk.json");
    let Bulk {
        entries, scores, ..
    } = serde_json::from_str::<Bulk>(entries).expect("Unable to decode JSON");
    let num_entries = entries.len();

    let create_reqs = entries.into_iter().map(|(id, e)| EntryForCreate {
        title: e
            .meta
            .get("DAH_entry_title")
            .and_then(|v| v.as_str())
            .unwrap_or("No title")
            .into(),
        entry_type: e
            .meta
            .get("DAH_entry_type")
            .and_then(|v| v.as_str())
            .and_then(EntryType::from_enum_string)
            .unwrap_or_default(),
        added_by: test_user_id(),
        overall_score: scores
            .get(&id)
            .and_then(|r| {
                r.meta
                    .as_object()
                    .and_then(|meta| meta.get("DAH_overall_score").and_then(|v| v.as_f64()))
            })
            .unwrap_or_default(),
        id,
    });

    // inserting via transaction for performance (on my setup: 5-6s to sub-1)
    let mut tx = mm.tx().await.expect("Unable to start transaction");
    for create_req in create_reqs {
        EntryBmc::create_entry(&mut tx, create_req)
            .await
            .expect("Unable to create entry");
    }
    tx.commit().await.expect("Unable to commit transaction");

    tracing::info!(
        "{:<12} -- Seeded {} entries into the database",
        "FOR-DEV-ONLY",
        num_entries
    );
}

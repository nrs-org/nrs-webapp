use std::{path::PathBuf, pin::Pin, sync::Arc};

use crate::{
    auth::external::AuthProviderRegistry,
    config::AppConfig,
    model::store::{Db, new_db_pool, primary_store::PrimaryStore},
};

pub mod entity;
pub mod entry;
mod error;
pub mod oauth_links;
mod store;
pub mod token;
pub mod user;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
pub use error::{Error, Result};
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use oauth2::{AsyncHttpClient, HttpClientError};
use sqlx::{Database, Transaction};

type SqlxDatabase = sqlx::Postgres;
type SqlxRow = sqlx::postgres::PgRow;

#[derive(Clone)]
pub struct ModelManager {
    db: Db,
    auth_providers: AuthProviderRegistry,
    http_client: HttpClientWrapper,
    _cache_dir_handle: Option<Arc<tempfile::TempDir>>,
}

#[derive(Clone)]
pub struct HttpClientWrapper(reqwest_middleware::ClientWithMiddleware);

pub type OAuth2HttpClientError = HttpClientError<reqwest_middleware::Error>;

impl<'a> AsyncHttpClient<'a> for HttpClientWrapper {
    type Error = OAuth2HttpClientError;
    type Future = Pin<
        Box<
            dyn Future<Output = core::result::Result<oauth2::HttpResponse, Self::Error>>
                + Send
                + 'a,
        >,
    >;

    fn call(&'a self, request: oauth2::HttpRequest) -> Self::Future {
        Box::pin(async move {
            let response = self
                .0
                .execute(
                    request
                        .try_into()
                        .map_err(reqwest_middleware::Error::Reqwest)
                        .map_err(Box::new)?,
                )
                .await
                .map_err(Box::new)?;

            let mut builder = axum::http::Response::builder().status(response.status());

            for (name, value) in response.headers().iter() {
                builder = builder.header(name, value);
            }

            builder
                .body(
                    response
                        .bytes()
                        .await
                        .map_err(reqwest_middleware::Error::Reqwest)
                        .map_err(Box::new)?
                        .to_vec(),
                )
                .map_err(HttpClientError::Http)
        })
    }
}

impl ModelManager {
    pub async fn new() -> Result<Self> {
        let (cache_dir, cache_dir_handle) = match AppConfig::get().SERVICE_CACHE_DIR.as_deref() {
            Some(dir) => {
                tokio::fs::create_dir_all(dir).await?;
                (PathBuf::from(dir), None)
            }
            None => {
                let cache_dir = tempfile::tempdir()?;
                (cache_dir.path().to_owned(), Some(Arc::new(cache_dir)))
            }
        };

        let http_cache_dir = cache_dir.join("http");
        tokio::fs::create_dir_all(&http_cache_dir).await?;

        let db = new_db_pool().await?;
        let auth_providers = AuthProviderRegistry::from_config();
        let http_cache_manager = CACacheManager::new(http_cache_dir, false);
        let http_client = reqwest_middleware::ClientBuilder::new(
            reqwest::ClientBuilder::new()
                .redirect(reqwest::redirect::Policy::none())
                // TODO: add a configurable timeout value
                .timeout(std::time::Duration::from_secs(10))
                .build()?,
        )
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: http_cache_manager,
            options: HttpCacheOptions::default(),
        }))
        .build();

        Ok(Self {
            db,
            auth_providers,
            http_client: HttpClientWrapper(http_client),
            _cache_dir_handle: cache_dir_handle,
        })
    }

    /// Get a reference to the manager's database pool.
    ///
    /// Returns a reference to the internal `Db`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let manager: crate::model::ModelManager = todo!();
    /// let db_ref = manager.db();
    /// let _ = db_ref; // use the Db reference
    /// ```
    pub(in crate::model) fn db(&self) -> &Db {
        &self.db
    }

    /// Begins a new database transaction from the manager's connection pool.
    ///
    /// # Returns
    ///
    /// A `Transaction<'static, SqlxDatabase>` wrapped in `Result` on success.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example(mgr: &crate::model::ModelManager) -> Result<(), crate::model::Error> {
    /// let mut tx = mgr.tx().await?;
    /// // use `tx`...
    /// Ok(())
    /// # }
    /// ```
    pub async fn tx(&self) -> Result<Transaction<'static, SqlxDatabase>> {
        let tx = self.db.begin().await?;
        Ok(tx)
    }

    pub fn auth_providers(&self) -> &AuthProviderRegistry {
        &self.auth_providers
    }

    pub fn http_client(&self) -> &reqwest_middleware::ClientWithMiddleware {
        &self.http_client.0
    }

    pub fn http_client_wrapper(&self) -> &HttpClientWrapper {
        &self.http_client
    }
}

impl PrimaryStore for ModelManager {
    type Executor<'a> = &'a Db;

    fn executor(&mut self) -> Self::Executor<'_> {
        self.db()
    }
}

impl<'t> PrimaryStore for Transaction<'t, SqlxDatabase> {
    fn executor(&mut self) -> Self::Executor<'_> {
        &mut *self
    }

    type Executor<'a>
        = &'a mut <SqlxDatabase as Database>::Connection
    where
        Self: 'a;
}

impl FromRef<ModelManager> for Key {
    fn from_ref(_: &ModelManager) -> Self {
        Key::from(&AppConfig::get().SERVICE_COOKIE_KEY)
    }
}

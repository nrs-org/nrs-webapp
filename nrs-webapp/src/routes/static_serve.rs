use tower_http::services::ServeDir;

use crate::config::AppConfig;

pub fn service() -> ServeDir {
    ServeDir::new(&AppConfig::get().STATIC_SERVE_DIR)
}

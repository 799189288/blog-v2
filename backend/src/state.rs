use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::dict::DictItemPublic;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: Arc<String>,
    /// Per-type cache of enabled dict items. Cleared on any admin write.
    pub dict_cache: Arc<RwLock<HashMap<String, Arc<Vec<DictItemPublic>>>>>,
}

impl AppState {
    pub fn new(db: PgPool, jwt_secret: String) -> Self {
        Self {
            db,
            jwt_secret: Arc::new(jwt_secret),
            dict_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn clear_dict_cache(&self) {
        self.dict_cache.write().await.clear();
    }

    pub async fn invalidate_dict_type(&self, type_code: &str) {
        self.dict_cache.write().await.remove(type_code);
    }
}

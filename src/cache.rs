use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use crate::models::UserPublic;

pub type UserCache = Arc<Cache<i32, UserPublic>>;
pub type TokenBlacklist = Arc<Cache<String, ()>>;

#[derive(Clone)]
pub struct CacheManager {
    pub user_cache: UserCache,
    pub token_blacklist: TokenBlacklist,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            // User cache with 10,000 entries, 1 hour TTL
            user_cache: Arc::new(
                Cache::builder()
                    .max_capacity(10_000)
                    .time_to_live(Duration::from_secs(3600))
                    .build()
            ),
            // Token blacklist with 50,000 entries, 24 hour TTL
            token_blacklist: Arc::new(
                Cache::builder()
                    .max_capacity(50_000)
                    .time_to_live(Duration::from_secs(86400))
                    .build()
            ),
        }
    }

    pub async fn get_user(&self, user_id: i32) -> Option<UserPublic> {
        self.user_cache.get(&user_id).await
    }

    pub async fn cache_user(&self, user: UserPublic) {
        self.user_cache.insert(user.id, user).await;
    }

    #[allow(dead_code)]
    pub async fn invalidate_user(&self, user_id: i32) {
        self.user_cache.invalidate(&user_id).await;
    }

    pub async fn is_token_blacklisted(&self, token: &str) -> bool {
        self.token_blacklist.get(token).await.is_some()
    }

    pub async fn blacklist_token(&self, token: String) {
        self.token_blacklist.insert(token, ()).await;
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
} 
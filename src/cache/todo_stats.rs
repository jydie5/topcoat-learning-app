use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::RwLock;

/// 一覧上部に表示する集計値。
#[derive(Clone, Copy, Debug)]
pub struct TodoStats {
    pub total: usize,
    pub completed: usize,
}

#[derive(Clone, Copy)]
struct CachedStats {
    value: TodoStats,
    stored_at: Instant,
}

/// プロセス内の短時間キャッシュ。
///
/// DBそのものではなく、再計算できる集計結果だけを5秒間保持する。
/// 書き込み時には明示的に破棄するため、責務と無効化条件を追いやすい。
#[derive(Clone)]
pub struct TodoStatsCache {
    entries: Arc<RwLock<HashMap<u64, CachedStats>>>,
    ttl: Duration,
}

impl TodoStatsCache {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(5),
        }
    }

    pub async fn get(&self, user_id: u64) -> Option<TodoStats> {
        self.entries
            .read()
            .await
            .get(&user_id)
            .copied()
            .filter(|cached| cached.stored_at.elapsed() < self.ttl)
            .map(|cached| cached.value)
    }

    pub async fn put(&self, user_id: u64, value: TodoStats) {
        self.entries.write().await.insert(
            user_id,
            CachedStats {
                value,
                stored_at: Instant::now(),
            },
        );
    }

    pub async fn invalidate(&self, user_id: u64) {
        self.entries.write().await.remove(&user_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn put_and_invalidate_are_visible_to_readers() {
        let cache = TodoStatsCache::new();
        let stats = TodoStats {
            total: 3,
            completed: 1,
        };

        cache.put(7, stats).await;
        let cached = cache.get(7).await.expect("保存した値を取得できる");
        assert_eq!(cached.total, 3);
        assert_eq!(cached.completed, 1);

        cache.invalidate(7).await;
        assert!(cache.get(7).await.is_none());
    }
}

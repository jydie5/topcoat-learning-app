use crate::{
    cache::todo_stats::{TodoStats, TodoStatsCache},
    domain::todo::Todo,
    repository::todo_repository::TodoRepository,
};

/// 画面が必要とするユースケースをまとめるサービス層。
///
/// DBアクセスとキャッシュ無効化をここで組み合わせることで、
/// ルートハンドラをHTTP入出力に集中させる。
#[derive(Clone)]
pub struct TodoService {
    repository: TodoRepository,
    stats_cache: TodoStatsCache,
}

impl TodoService {
    pub fn new(repository: TodoRepository, stats_cache: TodoStatsCache) -> Self {
        Self {
            repository,
            stats_cache,
        }
    }

    pub async fn search(&self, user_id: u64, query: &str) -> Result<Vec<Todo>, toasty::Error> {
        let needle = query.trim().to_lowercase();
        let todos = self.repository.list(user_id).await?;

        if needle.is_empty() {
            return Ok(todos);
        }

        Ok(todos
            .into_iter()
            .filter(|todo| {
                todo.title.to_lowercase().contains(&needle)
                    || todo.body.to_lowercase().contains(&needle)
            })
            .collect())
    }

    pub async fn find(&self, user_id: u64, id: u64) -> Result<Option<Todo>, toasty::Error> {
        self.repository.find(user_id, id).await
    }

    pub async fn stats(&self, user_id: u64) -> Result<TodoStats, toasty::Error> {
        if let Some(stats) = self.stats_cache.get(user_id).await {
            return Ok(stats);
        }

        let todos = self.repository.list(user_id).await?;
        let stats = TodoStats {
            total: todos.len(),
            completed: todos.iter().filter(|todo| todo.done).count(),
        };
        self.stats_cache.put(user_id, stats).await;
        Ok(stats)
    }

    pub async fn create(&self, user_id: u64, title: &str, body: &str) -> Result<(), toasty::Error> {
        self.repository.create(user_id, title, body).await?;
        self.stats_cache.invalidate(user_id).await;
        Ok(())
    }

    pub async fn update(
        &self,
        user_id: u64,
        id: u64,
        title: &str,
        body: &str,
    ) -> Result<bool, toasty::Error> {
        let updated = self.repository.update(user_id, id, title, body).await?;
        if updated {
            self.stats_cache.invalidate(user_id).await;
        }
        Ok(updated)
    }

    pub async fn toggle(&self, user_id: u64, id: u64) -> Result<bool, toasty::Error> {
        let updated = self.repository.toggle(user_id, id).await?;
        if updated {
            self.stats_cache.invalidate(user_id).await;
        }
        Ok(updated)
    }

    pub async fn delete(&self, user_id: u64, id: u64) -> Result<bool, toasty::Error> {
        let deleted = self.repository.delete(user_id, id).await?;
        if deleted {
            self.stats_cache.invalidate(user_id).await;
        }
        Ok(deleted)
    }
}

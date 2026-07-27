use toasty::Db;

use crate::domain::todo::Todo;

/// DB固有の操作を画面やユースケースから隔離する層。
#[derive(Clone)]
pub struct TodoRepository {
    db: Db,
}

impl TodoRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn list(&self, user_id: u64) -> Result<Vec<Todo>, toasty::Error> {
        Todo::filter(Todo::fields().user_id().eq(user_id))
            .order_by(Todo::fields().id().desc())
            .exec(&mut self.db.clone())
            .await
    }

    pub async fn find(&self, user_id: u64, id: u64) -> Result<Option<Todo>, toasty::Error> {
        Todo::filter(
            Todo::fields()
                .id()
                .eq(id)
                .and(Todo::fields().user_id().eq(user_id)),
        )
        .first()
        .exec(&mut self.db.clone())
        .await
    }

    pub async fn create(&self, user_id: u64, title: &str, body: &str) -> Result<(), toasty::Error> {
        toasty::create!(Todo {
            title,
            body,
            done: false,
            user_id
        })
        .exec(&mut self.db.clone())
        .await?;

        Ok(())
    }

    pub async fn toggle(&self, user_id: u64, id: u64) -> Result<bool, toasty::Error> {
        let mut db = self.db.clone();
        let Some(mut todo) = self.find(user_id, id).await? else {
            return Ok(false);
        };
        let done = !todo.done;
        toasty::update!(todo { done }).exec(&mut db).await?;
        Ok(true)
    }

    pub async fn update(
        &self,
        user_id: u64,
        id: u64,
        title: &str,
        body: &str,
    ) -> Result<bool, toasty::Error> {
        let mut db = self.db.clone();
        let Some(mut todo) = self.find(user_id, id).await? else {
            return Ok(false);
        };
        toasty::update!(todo { title, body }).exec(&mut db).await?;
        Ok(true)
    }

    pub async fn delete(&self, user_id: u64, id: u64) -> Result<bool, toasty::Error> {
        let Some(todo) = self.find(user_id, id).await? else {
            return Ok(false);
        };
        Todo::delete_by_id(&mut self.db.clone(), todo.id).await?;
        Ok(true)
    }
}

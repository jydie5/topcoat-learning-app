use toasty::Db;

use crate::domain::{session_record::SessionRecord, user::User};

#[derive(Clone)]
pub struct AuthRepository {
    db: Db,
}

impl AuthRepository {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub async fn create_user(
        &self,
        display_name: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, toasty::Error> {
        toasty::create!(User {
            display_name,
            email,
            password_hash
        })
        .exec(&mut self.db.clone())
        .await
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, toasty::Error> {
        User::filter(User::fields().email().eq(email))
            .first()
            .exec(&mut self.db.clone())
            .await
    }

    pub async fn find_user_by_id(&self, id: u64) -> Result<Option<User>, toasty::Error> {
        User::filter(User::fields().id().eq(id))
            .first()
            .exec(&mut self.db.clone())
            .await
    }

    pub async fn save_session(
        &self,
        user_id: u64,
        token_hash: &str,
        expires_at_unix: u64,
    ) -> Result<(), toasty::Error> {
        toasty::create!(SessionRecord {
            user_id,
            token_hash,
            expires_at_unix
        })
        .exec(&mut self.db.clone())
        .await?;
        Ok(())
    }

    pub async fn find_user_id_by_session(
        &self,
        token_hash: &str,
        now_unix: u64,
    ) -> Result<Option<u64>, toasty::Error> {
        let record = SessionRecord::filter(
            SessionRecord::fields()
                .token_hash()
                .eq(token_hash)
                .and(SessionRecord::fields().expires_at_unix().gt(now_unix)),
        )
        .first()
        .exec(&mut self.db.clone())
        .await?;
        Ok(record.map(|record| record.user_id))
    }

    pub async fn delete_session(&self, token_hash: &str) -> Result<(), toasty::Error> {
        SessionRecord::filter(SessionRecord::fields().token_hash().eq(token_hash))
            .delete()
            .exec(&mut self.db.clone())
            .await?;
        Ok(())
    }
}

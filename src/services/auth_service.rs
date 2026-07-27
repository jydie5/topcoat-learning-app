use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use topcoat::session::{Session, TokenHash};

use crate::repository::auth_repository::AuthRepository;

#[derive(Clone, Debug)]
pub struct UserIdentity {
    pub id: u64,
    pub display_name: String,
}

#[derive(Clone)]
pub struct AuthService {
    repository: AuthRepository,
}

impl AuthService {
    pub fn new(repository: AuthRepository) -> Self {
        Self { repository }
    }

    pub async fn register(
        &self,
        display_name: &str,
        email: &str,
        password: &str,
    ) -> Result<UserIdentity, String> {
        if self
            .repository
            .find_user_by_email(email)
            .await
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return Err("このメールアドレスは既に登録されています。".to_owned());
        }

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|error| error.to_string())?
            .to_string();
        let user = self
            .repository
            .create_user(display_name, email, &password_hash)
            .await
            .map_err(|error| error.to_string())?;
        Ok(UserIdentity {
            id: user.id,
            display_name: user.display_name,
        })
    }

    pub async fn authenticate(
        &self,
        email: &str,
        password: &str,
    ) -> Result<Option<UserIdentity>, String> {
        let Some(user) = self
            .repository
            .find_user_by_email(email)
            .await
            .map_err(|error| error.to_string())?
        else {
            return Ok(None);
        };
        let Ok(parsed_hash) = PasswordHash::new(&user.password_hash) else {
            return Ok(None);
        };
        if Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_err()
        {
            return Ok(None);
        }
        Ok(Some(UserIdentity {
            id: user.id,
            display_name: user.display_name,
        }))
    }

    pub async fn save_session(&self, user_id: u64, session: &Session) -> Result<(), String> {
        let expires_at_unix = session
            .expires_at
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_secs();
        self.repository
            .save_session(user_id, &hex::encode(*session.token_hash), expires_at_unix)
            .await
            .map_err(|error| error.to_string())
    }

    pub async fn resolve_session(&self, token_hash: &TokenHash) -> Option<UserIdentity> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
        let user_id = self
            .repository
            .find_user_id_by_session(&hex::encode(**token_hash), now)
            .await
            .ok()??;
        let user = self.repository.find_user_by_id(user_id).await.ok()??;
        Some(UserIdentity {
            id: user.id,
            display_name: user.display_name,
        })
    }

    pub async fn delete_session(&self, token_hash: &TokenHash) -> Result<(), String> {
        self.repository
            .delete_session(&hex::encode(**token_hash))
            .await
            .map_err(|error| error.to_string())
    }
}

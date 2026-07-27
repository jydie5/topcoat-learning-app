mod app;
mod auth;
mod cache;
mod components;
mod cookies;
mod domain;
mod repository;
mod services;
mod sessions;
mod validation;

use std::{env, fs, path::Path};

use cache::todo_stats::TodoStatsCache;
use repository::{auth_repository::AuthRepository, todo_repository::TodoRepository};
use services::{auth_service::AuthService, todo_service::TodoService};
use sessions::LocalCookieTokenStore;
use toasty::Db;
use topcoat::cookie::Key;
use topcoat::session::Config;

#[tokio::main]
async fn main() {
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:topcoat-todos.db".to_owned());
    let initialize_schema = database_needs_schema(&database_url);

    let db = Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&database_url)
        .await
        .expect("SQLiteへの接続に失敗しました");

    // Toasty 0.7のpush_schemaは既存テーブルに対して冪等ではないため、
    // 新しいSQLiteファイルに限って初期スキーマを作る。
    if initialize_schema {
        db.push_schema()
            .await
            .expect("データベーススキーマの準備に失敗しました");
    }

    let todo_service = TodoService::new(TodoRepository::new(db.clone()), TodoStatsCache::new());
    let auth_service = AuthService::new(AuthRepository::new(db));

    topcoat::start(app::router(
        todo_service,
        auth_service,
        cookie_key(),
        session_config(),
    ))
    .await
    .expect("Topcoatサーバーが異常終了しました");
}

fn database_needs_schema(database_url: &str) -> bool {
    if database_url.contains(":memory:") || database_url.contains("//memory") {
        return true;
    }

    let Some(path) = database_url.strip_prefix("sqlite:") else {
        return false;
    };
    let path = path.strip_prefix("//").unwrap_or(path);

    fs::metadata(Path::new(path))
        .map(|metadata| metadata.len() == 0)
        .unwrap_or(true)
}

fn cookie_key() -> Key {
    match env::var("COOKIE_SECRET") {
        Ok(secret) if secret.len() >= 32 => Key::derive_from(secret.as_bytes()),
        Ok(_) => panic!("COOKIE_SECRETは32バイト以上にしてください"),
        Err(_) if is_production() => {
            panic!("productionではCOOKIE_SECRETの設定が必須です")
        }
        Err(_) => {
            eprintln!(
                "COOKIE_SECRETが未設定のため、一時キーを使います。再起動するとCookieは無効になります。"
            );
            Key::generate()
        }
    }
}

fn session_config() -> Config {
    if is_production() {
        Config::default()
    } else {
        Config::builder().token_store(LocalCookieTokenStore).build()
    }
}

fn is_production() -> bool {
    env::var("APP_ENV").is_ok_and(|value| value.eq_ignore_ascii_case("production"))
}

#[cfg(test)]
mod tests {
    use super::database_needs_schema;

    #[test]
    fn memory_databases_always_need_a_schema() {
        assert!(database_needs_schema("sqlite::memory:"));
    }
}

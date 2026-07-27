/// ログインできるアプリ利用者。
///
/// パスワードはArgon2のハッシュだけを保存する。
#[derive(Debug, toasty::Model)]
pub struct User {
    #[key]
    #[auto]
    pub id: u64,

    pub display_name: String,

    #[unique]
    pub email: String,

    pub password_hash: String,
}

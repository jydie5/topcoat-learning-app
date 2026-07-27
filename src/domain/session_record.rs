/// Topcoat Sessionが発行したトークンのハッシュとユーザーの対応。
///
/// ブラウザへ渡る生トークンはDBに保存しない。
#[derive(Debug, toasty::Model)]
pub struct SessionRecord {
    #[key]
    #[auto]
    pub id: u64,

    #[index]
    pub user_id: u64,

    #[unique]
    pub token_hash: String,

    pub expires_at_unix: u64,
}

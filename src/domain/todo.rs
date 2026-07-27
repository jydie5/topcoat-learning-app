/// アプリの中心となるドメインモデル。
///
/// `toasty::Model` により、同じ型からSQLiteのテーブル定義と
/// 型安全なクエリAPIが生成される。
#[derive(Debug, toasty::Model)]
pub struct Todo {
    #[key]
    #[auto]
    pub id: u64,

    pub title: String,

    pub body: String,

    pub done: bool,

    /// 所有者を必ずクエリ条件に含め、別ユーザーのメモを見せない。
    #[index]
    pub user_id: u64,
}

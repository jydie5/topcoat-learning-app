use topcoat::{
    context::{Cx, app_context, memoize},
    router::RouterErrorExt,
};

use crate::services::auth_service::{AuthService, UserIdentity};

pub fn service(cx: &Cx) -> AuthService {
    app_context::<AuthService>(cx).clone()
}

/// 同じリクエスト中にレイアウト・ページ・集計から何度呼ばれても、
/// Session→DB→Userの解決は一度だけになる。
#[memoize]
pub async fn current_user(cx: &Cx) -> Option<UserIdentity> {
    let token_hash = topcoat::session::token_hash(cx).await.ok()??;
    service(cx).resolve_session(&token_hash).await
}

pub async fn require_user(cx: &Cx) -> topcoat::Result<&UserIdentity> {
    current_user(cx)
        .await
        .ok_or_redirect("/login")
        .map_err(Into::into)
}

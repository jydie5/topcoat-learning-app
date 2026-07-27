use std::time::Duration;

use topcoat::{
    context::Cx,
    cookie::{Cookie, Cookies, SameSite},
    session::{Token, TokenStore, TokenStoreFuture},
};

/// 公式CookieTokenStoreと同じHttpOnly/SameSite設定で、開発HTTPだけSecureを外す。
/// APP_ENV=productionではmain.rsがTopcoat標準のToken Storeを自動選択する。
pub struct LocalCookieTokenStore;

fn jar(cx: &Cx) -> impl Cookies {
    topcoat::cookie::cookies(cx)
        .override_same_site(SameSite::Lax)
        .override_http_only(true)
        .override_secure(false)
        .override_path("/")
}

impl TokenStore for LocalCookieTokenStore {
    fn read<'a>(&'a self, cx: &'a Cx) -> TokenStoreFuture<'a, Option<Token>> {
        Box::pin(async move {
            let Some(cookie) = jar(cx).get("session") else {
                return Ok(None);
            };
            Ok(Token::decode(cookie.value_trimmed()).ok())
        })
    }

    fn write<'a>(
        &'a self,
        cx: &'a Cx,
        token: Token,
        max_age: Duration,
    ) -> TokenStoreFuture<'a, ()> {
        Box::pin(async move {
            let max_age = topcoat::cookie::time::Duration::try_from(max_age)?;
            jar(cx)
                .override_max_age(max_age)
                .add(Cookie::new("session", token.encode()));
            Ok(())
        })
    }

    fn delete<'a>(&'a self, cx: &'a Cx) -> TokenStoreFuture<'a, ()> {
        Box::pin(async move {
            jar(cx).remove(Cookie::new("session", ""));
            Ok(())
        })
    }
}

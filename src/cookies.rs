use serde::{Deserialize, Serialize};
use topcoat::{
    context::Cx,
    cookie::{CookieStore, Cookies, cookie_store, signed_cookies},
};

/// ブラウザごとの訪問回数。署名付きCookieに保存する。
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Visits(pub u32);

impl Visits {
    fn increment(&mut self) {
        self.0 += 1;
    }
}

fn app_cookies(cx: &Cx) -> impl Cookies {
    signed_cookies(cx).default_path("/").default_http_only(true)
}

fn visits(cx: &Cx) -> CookieStore<Visits, impl Cookies> {
    cookie_store(app_cookies(cx), "topcoat_visits").parse_or_default()
}

pub fn increment_visits(cx: &Cx) -> topcoat::Result<Visits> {
    Ok(visits(cx).update(Visits::increment).commit()?)
}

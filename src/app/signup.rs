use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    router::{Form, SeeOther, internal_server_error, page, query_params, route, see_other},
    view::view,
};

use crate::{auth, validation};

#[topcoat::router::query_params(error = redirect("/signup"))]
struct SignupQuery {
    error: Option<String>,
}

#[page]
async fn index(cx: &Cx) -> Result {
    if auth::current_user(cx).await.is_some() {
        return Err(topcoat::router::redirect("/todos").into());
    }
    let error = &query_params::<SignupQuery>(cx)?.error;

    view! {
        <section class="mx-auto max-w-md">
            <div class="rounded-2xl border border-border bg-background p-7 shadow-sm">
                <p class="text-sm font-medium text-primary">"新規アカウント"</p>
                <h1 class="mt-2 text-2xl font-bold">"自分のメモ領域を作る"</h1>
                <p class="mt-2 text-sm leading-6 text-muted-foreground">
                    "パスワードはArgon2でハッシュ化し、平文では保存しません。"
                </p>
                if error.is_some() {
                    <div role="alert" class="mt-5 rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                        if error.as_deref() == Some("exists") {
                            "このメールアドレスは既に登録されています。"
                        } else {
                            "表示名・メールアドレス・8文字以上のパスワードを確認してください。"
                        }
                    </div>
                }
                <form method="post" action="/signup" class="mt-6 grid gap-4">
                    <label class="grid gap-2 text-sm font-medium">
                        "表示名"
                        <input class="h-11 rounded-lg border border-border bg-background px-3 outline-none focus:ring-2 focus:ring-ring" name="display_name" maxlength="60" autocomplete="name" required="">
                    </label>
                    <label class="grid gap-2 text-sm font-medium">
                        "メールアドレス"
                        <input class="h-11 rounded-lg border border-border bg-background px-3 outline-none focus:ring-2 focus:ring-ring" type="email" name="email" maxlength="254" autocomplete="email" required="">
                    </label>
                    <label class="grid gap-2 text-sm font-medium">
                        "パスワード（8文字以上）"
                        <input class="h-11 rounded-lg border border-border bg-background px-3 outline-none focus:ring-2 focus:ring-ring" type="password" name="password" minlength="8" autocomplete="new-password" required="">
                    </label>
                    <button class="h-11 rounded-lg bg-primary px-4 font-medium text-primary-foreground">
                        "作成してログイン"
                    </button>
                </form>
                <p class="mt-5 text-center text-sm text-muted-foreground">
                    <a href="/login" class="font-medium text-foreground underline underline-offset-4">
                        "ログインへ戻る"
                    </a>
                </p>
            </div>
        </section>
    }
}

#[derive(Deserialize)]
struct SignupForm {
    display_name: String,
    email: String,
    password: String,
}

#[route(POST)]
async fn signup(cx: &Cx, Form(form): Form<SignupForm>) -> Result<SeeOther> {
    let Ok((display_name, email)) =
        validation::signup(&form.display_name, &form.email, &form.password)
    else {
        return Ok(see_other("/signup?error=validation"));
    };
    let user = match auth::service(cx)
        .register(&display_name, &email, &form.password)
        .await
    {
        Ok(user) => user,
        Err(error) if error.contains("既に登録") => {
            return Ok(see_other("/signup?error=exists"));
        }
        Err(error) => {
            return Err(internal_server_error(std::io::Error::other(error)).into());
        }
    };
    let session = topcoat::session::start(cx).await?;
    auth::service(cx)
        .save_session(user.id, &session)
        .await
        .map_err(|error| internal_server_error(std::io::Error::other(error)))?;
    Ok(see_other("/todos"))
}

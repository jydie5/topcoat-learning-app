use serde::Deserialize;
use topcoat::{
    Result,
    context::Cx,
    router::{Form, SeeOther, internal_server_error, page, query_params, route, see_other},
    view::view,
};

use crate::auth;

#[topcoat::router::query_params(error = redirect("/login"))]
struct LoginQuery {
    error: Option<String>,
}

#[page]
async fn index(cx: &Cx) -> Result {
    if auth::current_user(cx).await.is_some() {
        return Err(topcoat::router::redirect("/todos").into());
    }
    let error = query_params::<LoginQuery>(cx)?
        .error
        .as_deref()
        .map(login_error);

    view! {
        <section class="mx-auto max-w-md">
            <div class="rounded-2xl border border-border bg-background p-7 shadow-sm">
                <p class="text-sm font-medium text-primary">"Sessionログイン"</p>
                <h1 class="mt-2 text-2xl font-bold">"おかえりなさい"</h1>
                <p class="mt-2 text-sm leading-6 text-muted-foreground">
                    "ユーザーごとに分離されたメモへログインします。"
                </p>
                if let Some(error) = error {
                    <div role="alert" class="mt-5 rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                        (error)
                    </div>
                }
                <form method="post" action="/login" class="mt-6 grid gap-4">
                    <label class="grid gap-2 text-sm font-medium">
                        "メールアドレス"
                        <input class="h-11 rounded-lg border border-border bg-background px-3 outline-none focus:ring-2 focus:ring-ring" type="email" name="email" autocomplete="email" required="">
                    </label>
                    <label class="grid gap-2 text-sm font-medium">
                        "パスワード"
                        <input class="h-11 rounded-lg border border-border bg-background px-3 outline-none focus:ring-2 focus:ring-ring" type="password" name="password" autocomplete="current-password" minlength="8" required="">
                    </label>
                    <button class="h-11 rounded-lg bg-primary px-4 font-medium text-primary-foreground">
                        "ログイン"
                    </button>
                </form>
                <p class="mt-5 text-center text-sm text-muted-foreground">
                    "初めてですか？ "
                    <a href="/signup" class="font-medium text-foreground underline underline-offset-4">
                        "アカウントを作成"
                    </a>
                </p>
            </div>
        </section>
    }
}

#[derive(Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}

#[route(POST)]
async fn login(cx: &Cx, Form(form): Form<LoginForm>) -> Result<SeeOther> {
    let email = form.email.trim().to_lowercase();
    if !email.contains('@') || form.password.chars().count() < 8 {
        return Ok(see_other("/login?error=invalid"));
    }
    let user = auth::service(cx)
        .authenticate(&email, &form.password)
        .await
        .map_err(|error| internal_server_error(std::io::Error::other(error)))?;
    let Some(user) = user else {
        return Ok(see_other("/login?error=credentials"));
    };
    let session = topcoat::session::start(cx).await?;
    auth::service(cx)
        .save_session(user.id, &session)
        .await
        .map_err(|error| internal_server_error(std::io::Error::other(error)))?;
    Ok(see_other("/todos"))
}

fn login_error(code: &str) -> &'static str {
    match code {
        "credentials" => "メールアドレスまたはパスワードが違います。",
        _ => "入力内容を確認してください。",
    }
}

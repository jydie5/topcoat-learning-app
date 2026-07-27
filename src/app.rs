mod htmx_search;
mod login;
mod logout;
mod signup;
mod todos;

use crate::{
    auth::current_user,
    cookies::increment_visits,
    services::{auth_service::AuthService, todo_service::TodoService},
};
use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::Cx,
    cookie::{Key, RouterBuilderCookieExt},
    font::{RouterBuilderFontExt, fontsource::fontsource_font},
    router::{
        BadRequestError, NotFoundError, SeeOther, Slot, StatusCode, layout, route, see_other,
    },
    runtime::{RouterBuilderProcedureExt, RouterBuilderShardExt},
    session::{Config, RouterBuilderSessionExt},
    tailwind,
    view::{component, view},
};

/// Topcoatのモジュールルーターに、アプリ共通の依存関係を組み立てる。
pub fn router(
    todo_service: TodoService,
    auth_service: AuthService,
    cookie_key: Key,
    session_config: Config,
) -> topcoat::router::Router {
    topcoat::router::module_router!()
        .assets(AssetBundle::load().expect("アセットの読み込みに失敗しました"))
        .cookies()
        .sessions(session_config)
        .app_context(todo_service)
        .app_context(auth_service)
        .app_context(cookie_key)
        .discover_fonts()
        .discover_procedures()
        .discover_shards()
        .build()
}

/// すべてのページを包む共通レイアウト。
///
/// CSS、ブラウザ側runtime、ナビゲーション、Cookieのデモを一か所に置く。
#[layout]
async fn root_layout(cx: &Cx, slot: Slot<'_>) -> Result {
    let visits = increment_visits(cx)?;
    let user = current_user(cx).await;
    let content = match slot.await {
        Err(error) if error.downcast_ref::<NotFoundError>().is_some() => view! {
            (StatusCode::NOT_FOUND)
            error_panel(
                code: "404",
                title: "ページが見つかりません",
                message: "URLが違うか、メモが削除された可能性があります。",
            )
        },
        Err(error) if error.downcast_ref::<BadRequestError>().is_some() => view! {
            (StatusCode::BAD_REQUEST)
            error_panel(
                code: "400",
                title: "入力を読み取れませんでした",
                message: "URLや入力内容を確認して、もう一度お試しください。",
            )
        },
        result => result,
    }?;

    view! {
        <!DOCTYPE html>
        <html lang="ja">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>"Topcoat Notes"</title>
                <meta
                    name="description"
                    content="Topcoatを学ぶためのメモ／ToDoアプリ"
                >
                topcoat::dev::script()
                topcoat::runtime::script()
                <script
                    src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.10/dist/htmx.min.js"
                    integrity="sha384-H5SrcfygHmAuTDZphMHqBJLc3FhssKjG7w/CeCpFReSfwBWDTKpkzPP8c+cLsK+V"
                    crossorigin="anonymous"
                    defer=""
                ></script>
                topcoat::font::link(font: fontsource_font!(GEIST, host: Asset))
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
            </head>
            <body class="min-h-screen bg-background transition-colors">
                <header class="border-b border-border bg-background/95">
                    <div
                        class="mx-auto flex max-w-5xl items-center justify-between px-5 py-4"
                    >
                        <a href="/todos" class="font-semibold tracking-tight">
                            "Topcoat Notes"
                        </a>
                        <div class="flex items-center gap-2 text-xs text-muted-foreground">
                            <button
                                type="button"
                                class="rounded-lg border border-border px-2.5 py-1.5 text-foreground"
                                onclick="document.documentElement.classList.toggle('dark'); this.textContent = document.documentElement.classList.contains('dark') ? 'ライト' : 'ダーク';"
                            >
                                "ダーク"
                            </button>
                            if let Some(user) = user {
                                <span class="hidden sm:inline">(&user.display_name)</span>
                                <form method="post" action="/logout">
                                    <button class="rounded-lg border border-border px-2.5 py-1.5 text-foreground">
                                        "ログアウト"
                                    </button>
                                </form>
                            }
                            <span>"signed Cookie"</span>
                            <span
                                class="rounded-full border border-border px-2.5 py-1 text-foreground"
                            >
                                "訪問 " (visits.0) " 回"
                            </span>
                        </div>
                    </div>
                </header>
                <main class="mx-auto max-w-5xl px-5 py-10">
                    (content)
                </main>
                <footer class="mx-auto max-w-5xl px-5 pb-10 text-xs text-muted-foreground">
                    "Rust + Topcoat 0.4 · SSR / signal / shard / procedure / Session / htmx"
                </footer>
            </body>
        </html>
    }
}

#[component]
async fn error_panel(code: &'static str, title: &'static str, message: &'static str) -> Result {
    view! {
        <section class="mx-auto max-w-xl rounded-2xl border border-border bg-background p-8 text-center shadow-sm">
            <p class="text-sm font-semibold text-primary">(code)</p>
            <h1 class="mt-2 text-2xl font-bold">(title)</h1>
            <p class="mt-3 text-sm leading-6 text-muted-foreground">(message)</p>
            <a href="/todos" class="mt-6 inline-flex rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground">
                "メモ一覧へ"
            </a>
        </section>
    }
}

/// ルートURLは一覧へ誘導する。
#[route(GET)]
async fn home() -> Result<SeeOther> {
    Ok(see_other("/todos"))
}

mod id;

use serde::Deserialize;
use topcoat::{
    Result,
    context::{Cx, app_context, memoize},
    router::{Form, SeeOther, page, query_params, route, see_other},
    runtime::{Event, shard},
    view::{attributes, component, view},
};

use crate::{
    auth::require_user,
    cache::todo_stats::TodoStats,
    components::{
        button::{ButtonSize, ButtonVariant, button, button_variants},
        card::{card, card_content, card_description, card_header, card_title},
        input::input,
        label::label,
        textarea::textarea,
    },
    services::todo_service::TodoService,
    validation,
};

pub(super) fn service(cx: &Cx) -> TodoService {
    app_context::<TodoService>(cx).clone()
}

#[memoize]
async fn user_stats(cx: &Cx, user_id: u64) -> Option<TodoStats> {
    service(cx).stats(user_id).await.ok()
}

#[topcoat::router::query_params(error = redirect("/todos"))]
struct TodosQuery {
    error: Option<String>,
}

#[page]
async fn index(cx: &Cx) -> Result {
    let user = require_user(cx).await?;
    let stats = user_stats(cx, user.id).await;
    let _same_stats = user_stats(cx, user.id).await;
    let validation_error = query_params::<TodosQuery>(cx)?.error.is_some();

    view! {
        <section class="mb-8 grid gap-5 lg:grid-cols-[1fr_auto] lg:items-end">
            <div>
                <p class="mb-2 text-sm font-medium text-muted-foreground">
                    "学習用フルスタック・サンプル"
                </p>
                <h1 class="text-3xl font-bold tracking-tight sm:text-4xl">
                    (&user.display_name) "さんのメモ"
                </h1>
                <p class="mt-3 max-w-2xl text-sm leading-6 text-muted-foreground">
                    "画面はSSR、検索は shard、開閉は signal、保存はSQLite。\
                    ブラウザとサーバーの境界を追いながら試せます。"
                </p>
            </div>
            <dl class="grid grid-cols-2 gap-3">
                <div class="min-w-28 rounded-xl border border-border p-4 shadow-xs">
                    <dt class="text-xs text-muted-foreground">"全件"</dt>
                    <dd class="mt-1 text-2xl font-semibold">
                        (stats.map_or(0, |stats| stats.total))
                    </dd>
                </div>
                <div class="min-w-28 rounded-xl border border-border p-4 shadow-xs">
                    <dt class="text-xs text-muted-foreground">"完了"</dt>
                    <dd class="mt-1 text-2xl font-semibold">
                        (stats.map_or(0, |stats| stats.completed))
                    </dd>
                </div>
            </dl>
        </section>

        create_panel(validation_error: validation_error)

        <section class="mt-8">
            <div class="mb-4 flex items-end justify-between gap-4">
                <div>
                    <h2 class="text-xl font-semibold">"メモ一覧"</h2>
                    <p class="mt-1 text-sm text-muted-foreground">
                        "同じ検索を #[shard] と htmx の両方で比較できます。"
                    </p>
                </div>
            </div>
            <div class="grid gap-6 lg:grid-cols-2">
                <div class="rounded-xl border border-border p-5">
                    <div class="mb-4">
                        <span class="rounded-md bg-primary/10 px-2 py-1 text-xs font-semibold text-primary">
                            "#[shard]"
                        </span>
                        <p class="mt-2 text-xs leading-5 text-muted-foreground">
                            "Rust関数がsignal引数の変更に合わせて部分再描画。"
                        </p>
                    </div>
                    search_panel()
                </div>
                <div class="rounded-xl border border-border p-5">
                    <div class="mb-4">
                        <span class="rounded-md bg-foreground/10 px-2 py-1 text-xs font-semibold">
                            "htmx"
                        </span>
                        <p class="mt-2 text-xs leading-5 text-muted-foreground">
                            "HTML属性でGETし、返ったHTML断片を差し替え。"
                        </p>
                    </div>
                    <label class="mb-5 block">
                        <span class="sr-only">"htmxでメモを検索"</span>
                        <input
                            type="search"
                            name="q"
                            placeholder="htmxで検索…"
                            class="h-11 w-full rounded-xl border border-border bg-background px-4 text-sm shadow-xs outline-none focus-visible:ring-2 focus-visible:ring-ring"
                            hx-get="/htmx-search"
                            hx-trigger="input changed delay:250ms"
                            hx-target="#htmx-results"
                            hx-swap="innerHTML"
                        >
                    </label>
                    <div id="htmx-results" hx-get="/htmx-search" hx-trigger="load">
                        <p class="text-sm text-muted-foreground">"読み込み中…"</p>
                    </div>
                </div>
            </div>
        </section>
    }
}

/// `signal` による開閉はブラウザ内だけで完結し、通信を発生させない。
#[component]
async fn create_panel(validation_error: bool) -> Result {
    view! {
        signal open = validation_error;

        <div class="rounded-xl border border-border bg-foreground/[0.02] p-5">
            <div class="flex items-center justify-between gap-4">
                <div>
                    <h2 class="font-semibold">"新しいメモ"</h2>
                    <p class="mt-1 text-sm text-muted-foreground">
                        "この開閉はクライアント側 signal のデモです。"
                    </p>
                </div>
                <button
                    type="button"
                    class=(button_variants(ButtonVariant::Primary, ButtonSize::Md))
                    @click=$(|_event| open.set(!open.get()))
                >
                    $(if open.get() { "閉じる" } else { "メモを追加" })
                </button>
            </div>

            <div :hidden=$(!open.get()) class="pt-5">
                if validation_error {
                    <div role="alert" class="mb-4 rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
                        "タイトルは必須・120文字以内、本文は2000文字以内です。"
                    </div>
                }
                card(
                    card_header(
                        card_title("内容を入力")
                        card_description("送信後、TopcoatのrouteがSQLiteへ保存します。")
                    )
                    card_content(
                        <form method="post" action="/todos" class="grid gap-4">
                            <div class="grid gap-2">
                                label(attrs: attributes! { for="title" }, "タイトル")
                                input(attrs: attributes! {
                                    id="title"
                                    name="title"
                                    type="text"
                                    maxlength="120"
                                    placeholder="例: Topcoatのルーティングを読む"
                                    required=""
                                })
                            </div>
                            <div class="grid gap-2">
                                label(attrs: attributes! { for="body" }, "メモ")
                                textarea(attrs: attributes! {
                                    id="body"
                                    name="body"
                                    maxlength="2000"
                                    placeholder="補足や気づきを残す（任意）"
                                })
                            </div>
                            <div class="flex justify-end">
                                button(
                                    attrs: attributes! { type="submit" },
                                    "保存する"
                                )
                            </div>
                        </form>
                    )
                )
            </div>
        </div>
    }
}

#[component]
async fn search_panel() -> Result {
    view! {
        signal query = String::new();

        <label class="mb-5 block">
            <span class="sr-only">"メモを検索"</span>
            <input
                type="search"
                placeholder="タイトルまたは本文を検索…"
                class="h-11 w-full rounded-xl border border-border bg-background px-4 \
                    text-sm shadow-xs outline-none placeholder:text-muted-foreground \
                    focus-visible:ring-2 focus-visible:ring-ring"
                :value=$(query.get())
                @input=$(|event: Event| query.set(event.target.value))
            >
        </label>

        search_results(query: $(query.get()))
    }
}

/// 引数が変わるたびサーバーで再描画される部分。
/// クライアントから渡るqueryは信用せず、長さを制限してから扱う。
#[shard]
async fn search_results(cx: &Cx, query: String) -> Result {
    let user = require_user(cx).await?;
    let safe_query: String = query.chars().take(120).collect();
    let todos = service(cx).search(user.id, &safe_query).await?;

    view! {
        <div class="mb-3 flex items-center justify-between text-xs text-muted-foreground">
            if safe_query.trim().is_empty() {
                <span>"新しい順に表示"</span>
            } else {
                <span>"検索: " (&safe_query)</span>
            }
            <span>(todos.len()) " 件"</span>
        </div>

        if todos.is_empty() {
            <div class="rounded-xl border border-dashed border-border px-6 py-14 text-center">
                <p class="font-medium">"該当するメモはありません"</p>
                <p class="mt-2 text-sm text-muted-foreground">
                    "検索語を変えるか、新しいメモを追加してください。"
                </p>
            </div>
        } else {
            <div class="grid gap-3">
                for todo in todos {
                    <article
                        class="group rounded-xl border border-border bg-background p-5 shadow-xs \
                            transition hover:-translate-y-0.5 hover:shadow-sm"
                    >
                        <div class="flex items-start justify-between gap-5">
                            <div class="min-w-0">
                                <div class="mb-2 flex items-center gap-2">
                                    if todo.done {
                                        <span
                                            class="rounded-md bg-foreground/5 px-2 py-0.5 \
                                                text-xs font-medium"
                                        >
                                            "完了"
                                        </span>
                                    } else {
                                        <span
                                            class="rounded-md bg-primary px-2 py-0.5 text-xs \
                                                font-medium text-primary-foreground"
                                        >
                                            "進行中"
                                        </span>
                                    }
                                    <span class="text-xs text-muted-foreground">
                                        "#" (todo.id)
                                    </span>
                                </div>
                                <h3 class="font-semibold">
                                    if todo.done {
                                        <s class="text-muted-foreground">(&todo.title)</s>
                                    } else {
                                        (&todo.title)
                                    }
                                </h3>
                                if !todo.body.is_empty() {
                                    <p class="mt-2 line-clamp-2 text-sm leading-6 text-muted-foreground">
                                        (&todo.body)
                                    </p>
                                }
                            </div>
                            <a
                                href=(("/todos/", todo.id))
                                class=(button_variants(
                                    ButtonVariant::Outline,
                                    ButtonSize::Sm,
                                ))
                            >
                                "詳細"
                            </a>
                        </div>
                    </article>
                }
            </div>
        }
    }
}

#[derive(Deserialize)]
struct NewTodo {
    title: String,
    #[serde(default)]
    body: String,
}

/// POST /todos — HTMLフォームを受け取り、サービス層経由で永続化する。
#[route(POST)]
async fn create(cx: &Cx, Form(form): Form<NewTodo>) -> Result<SeeOther> {
    let user = require_user(cx).await?;
    let Ok((title, body)) = validation::todo(&form.title, &form.body) else {
        return Ok(see_other("/todos?error=validation"));
    };
    service(cx).create(user.id, &title, &body).await?;

    Ok(see_other("/todos"))
}

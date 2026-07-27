mod delete;
mod toggle;

use topcoat::{
    Result,
    context::Cx,
    router::{RouterErrorExt, bad_request, page, path_param},
    runtime::procedure,
    view::view,
};

use crate::{
    auth::require_user,
    components::{
        badge::{BadgeVariant, badge},
        button::{ButtonSize, ButtonVariant, button_variants},
    },
    validation,
};

use super::service;

/// `id.rs` に置かれたpath paramは、親の `todos` から `{todo_id}` と推論される。
#[path_param(error = bad_request)]
pub(super) struct TodoId(pub u64);

#[page]
async fn detail(cx: &Cx) -> Result {
    let user = require_user(cx).await?;
    let id = *path_param::<TodoId>(cx)?;
    let todo = service(cx).find(user.id, id).await?.ok_or_not_found()?;
    let note_id = todo.id as f64;
    let todo_title = todo.title.clone();
    let todo_body = todo.body.clone();

    view! {
        signal editing = false;
        signal todo_id = note_id;
        signal title = todo_title;
        signal body = todo_body;

        <div class="mb-6">
            <a
                href="/todos"
                class="text-sm text-muted-foreground hover:text-foreground"
            >
                "← 一覧へ戻る"
            </a>
        </div>

        <article
            class="rounded-xl border border-border bg-background p-6 shadow-sm sm:p-8"
        >
            <div class="mb-6 flex flex-wrap items-start justify-between gap-4">
                <div>
                    <div class="mb-3 flex items-center gap-2">
                        if todo.done {
                            badge(variant: BadgeVariant::Secondary, "完了")
                        } else {
                            badge("進行中")
                        }
                        <span class="text-xs text-muted-foreground">
                            "Todo #"
                            (todo.id)
                        </span>
                    </div>
                    <h1 :hidden=$(editing.get()) class="text-2xl font-bold tracking-tight sm:text-3xl">
                        (&todo.title)
                    </h1>
                </div>
                <button
                    type="button"
                    class=(button_variants(ButtonVariant::Outline, ButtonSize::Sm))
                    @click=$(|_event| editing.set(!editing.get()))
                >
                    $(if editing.get() { "編集を閉じる" } else { "インライン編集" })
                </button>
            </div>

            <div :hidden=$(editing.get())>
                if todo.body.is_empty() {
                    <p class="rounded-lg bg-foreground/[0.03] p-5 text-sm text-muted-foreground">
                        "本文はありません。"
                    </p>
                } else {
                    <p class="whitespace-pre-wrap text-sm leading-7">
                        (&todo.body)
                    </p>
                }
            </div>

            <div :hidden=$(!editing.get()) class="grid gap-4 rounded-xl border border-primary/20 bg-primary/5 p-5">
                <label class="grid gap-2 text-sm font-medium">
                    "タイトル"
                    <input
                        class="h-11 rounded-lg border border-border bg-background px-3 outline-none focus:ring-2 focus:ring-ring"
                        maxlength="120"
                        value=(&todo.title)
                        @input=$(|event: topcoat::runtime::Event| title.set(event.target.value))
                    >
                </label>
                <label class="grid gap-2 text-sm font-medium">
                    "本文"
                    <textarea
                        class="min-h-32 rounded-lg border border-border bg-background p-3 outline-none focus:ring-2 focus:ring-ring"
                        maxlength="2000"
                        @input=$(|event: topcoat::runtime::Event| body.set(event.target.value))
                    >(&todo.body)</textarea>
                </label>
                <div class="flex items-center justify-between gap-3">
                    <p class="text-xs text-muted-foreground">
                        "#[procedure] がJSONでRust関数を呼び出します。"
                    </p>
                    <button
                        type="button"
                        class=(button_variants(ButtonVariant::Primary, ButtonSize::Md))
                        @click=$(async |_event| {
                            update_todo(todo_id.get(), title.get(), body.get()).await;
                        })
                    >
                        "変更を保存"
                    </button>
                </div>
            </div>
            <p class="mt-3 text-xs text-muted-foreground">
                "保存後は「一覧へ戻る」で永続化結果を確認できます。"
            </p>

            <div
                class="mt-8 flex flex-wrap items-center gap-3 border-t border-border pt-6"
            >
                <form method="post" action=(("/todos/", todo.id, "/toggle"))>
                    <button
                        type="submit"
                        class=(button_variants(
                            ButtonVariant::Primary,
                            ButtonSize::Md,
                        ))
                    >
                        if todo.done {
                            "未完了に戻す"
                        } else {
                            "完了にする"
                        }
                    </button>
                </form>
                <form method="post" action=(("/todos/", todo.id, "/delete"))>
                    <button
                        type="submit"
                        class=(button_variants(
                            ButtonVariant::Destructive,
                            ButtonSize::Md,
                        ))
                    >
                        "削除"
                    </button>
                </form>
            </div>
        </article>
    }
}

#[procedure]
async fn update_todo(cx: &Cx, id: f64, title: String, body: String) -> Result<String> {
    let user = require_user(cx).await?;
    if !id.is_finite() || id < 0.0 || id.fract() != 0.0 || id > u64::MAX as f64 {
        return Err(bad_request("無効なメモIDです。").into());
    }
    let id = id as u64;
    let (title, body) = validation::todo(&title, &body).map_err(bad_request)?;
    service(cx)
        .update(user.id, id, &title, &body)
        .await?
        .then_some(())
        .ok_or_not_found()?;
    Ok(title)
}

use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{query_params, route},
    view::view,
};

use crate::{auth::require_user, services::todo_service::TodoService};

#[topcoat::router::query_params(error = bad_request)]
struct SearchQuery {
    q: Option<String>,
}

#[route(GET)]
async fn search(cx: &Cx) -> Result {
    let user = require_user(cx).await?;
    let query = query_params::<SearchQuery>(cx)?
        .q
        .as_deref()
        .unwrap_or_default();
    let safe_query: String = query.chars().take(120).collect();
    let todos = app_context::<TodoService>(cx)
        .search(user.id, &safe_query)
        .await?;

    view! {
        <div class="mb-3 flex items-center justify-between text-xs text-muted-foreground">
            <span>"HTML断片で更新"</span>
            <span>(todos.len()) " 件"</span>
        </div>
        if todos.is_empty() {
            <p class="rounded-lg border border-dashed border-border p-6 text-center text-sm text-muted-foreground">
                "該当するメモはありません"
            </p>
        } else {
            <div class="grid gap-2">
                for todo in todos {
                    <a href=(("/todos/", todo.id)) class="block rounded-lg border border-border p-3 transition hover:bg-foreground/[0.03]">
                        <div class="flex items-center justify-between gap-3">
                            <span class="truncate text-sm font-medium">(&todo.title)</span>
                            <span class="text-xs text-muted-foreground">
                                if todo.done { "完了" } else { "進行中" }
                            </span>
                        </div>
                    </a>
                }
            </div>
        }
    }
}

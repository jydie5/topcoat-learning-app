use topcoat::{
    Result,
    context::Cx,
    router::{SeeOther, path_param, route, see_other},
};

use super::{TodoId, service};
use crate::auth::require_user;

#[route(POST)]
async fn toggle(cx: &Cx) -> Result<SeeOther> {
    let user = require_user(cx).await?;
    let id = *path_param::<TodoId>(cx)?;
    service(cx)
        .toggle(user.id, id)
        .await?
        .then_some(())
        .ok_or_else(topcoat::router::not_found)?;
    Ok(see_other(&format!("/todos/{id}")))
}

use topcoat::{
    Result,
    context::Cx,
    router::{SeeOther, internal_server_error, route, see_other},
};

use crate::auth;

#[route(POST)]
async fn logout(cx: &Cx) -> Result<SeeOther> {
    if let Some(token_hash) = topcoat::session::stop(cx).await? {
        auth::service(cx)
            .delete_session(&token_hash)
            .await
            .map_err(|error| internal_server_error(std::io::Error::other(error)))?;
    }
    Ok(see_other("/login"))
}

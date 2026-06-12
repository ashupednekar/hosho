use tower_service::Service;
use worker::*;

mod handlers;
mod internal;
mod router;

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    _env: Env,
    _ctx: Context,
) -> Result<axum::http::Response<axum::body::Body>> {
    Ok(router::build_routes().call(req).await?)
}
